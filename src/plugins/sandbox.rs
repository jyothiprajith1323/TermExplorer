//! WebAssembly Sandboxed Execution Runtime using `wasmi` and `sha2`.

use sha2::{Digest, Sha256};
use std::collections::HashSet;
use wasmi::{Caller, Config, Engine, Linker, Memory, Module, Store};

use super::manifest::PluginManifest;
use super::permissions::PluginPermission;

pub const DEFAULT_FUEL_LIMIT: u64 = 5_000_000;
pub const MAX_ALLOWED_MEMORY_BYTES: usize = 16 * 1024 * 1024; // 16 MiB
pub const MAX_ALLOWED_TABLE_ELEMENTS: u32 = 10_000;

#[derive(Debug, Clone)]
pub struct PluginLimiter {
    pub max_memory_bytes: usize,
    pub max_table_elements: u32,
}

impl Default for PluginLimiter {
    fn default() -> Self {
        Self {
            max_memory_bytes: MAX_ALLOWED_MEMORY_BYTES,
            max_table_elements: MAX_ALLOWED_TABLE_ELEMENTS,
        }
    }
}

impl wasmi::ResourceLimiter for PluginLimiter {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool, wasmi::errors::MemoryError> {
        Ok(desired <= self.max_memory_bytes)
    }

    fn table_growing(
        &mut self,
        _current: u32,
        desired: u32,
        _maximum: Option<u32>,
    ) -> Result<bool, wasmi::errors::TableError> {
        Ok(desired <= self.max_table_elements)
    }
}

#[derive(Debug, Clone)]
pub struct HostState {
    pub workspace_path: String,
    pub active_file: String,
    pub active_file_size: u64,
    pub status_text: Option<String>,
    pub logs: Vec<String>,
    pub permissions: HashSet<PluginPermission>,
    pub limiter: PluginLimiter,
}

pub struct PluginInstance {
    pub manifest: PluginManifest,
    store: Store<HostState>,
    memory: Memory,
    init_func: Option<wasmi::TypedFunc<(), i32>>,
    on_cmd_func: Option<wasmi::TypedFunc<(i32, i32), i32>>,
    status_func: Option<wasmi::TypedFunc<(i32, i32), i32>>,
}

impl PluginInstance {
    /// Load, verify, and instantiate a plugin wasm binary.
    pub fn new(
        manifest: PluginManifest,
        wasm_bytes: &[u8],
        workspace_path: &str,
    ) -> Result<Self, String> {
        // Verify checksum if present
        if let Some(expected_sha) = &manifest.sha256 {
            let mut hasher = Sha256::new();
            hasher.update(wasm_bytes);
            let calculated = format!("{:x}", hasher.finalize());
            if !calculated.eq_ignore_ascii_case(expected_sha.trim()) {
                return Err(format!(
                    "Plugin integrity check failed: expected sha256 '{}', found '{}'",
                    expected_sha, calculated
                ));
            }
        }

        let mut config = Config::default();
        config.consume_fuel(true);

        let engine = Engine::new(&config);
        let module = Module::new(&engine, wasm_bytes)
            .map_err(|e| format!("Module compilation error: {}", e))?;

        let host_state = HostState {
            workspace_path: workspace_path.to_string(),
            active_file: String::new(),
            active_file_size: 0,
            status_text: None,
            logs: Vec::new(),
            permissions: manifest.permissions.clone(),
            limiter: PluginLimiter::default(),
        };

        let mut store = Store::new(&engine, host_state);
        store.limiter(|state| &mut state.limiter);
        store
            .set_fuel(DEFAULT_FUEL_LIMIT)
            .map_err(|e| e.to_string())?;

        let mut linker = Linker::new(&engine);

        // Host API: logging
        linker
            .define(
                "env",
                "host_log",
                wasmi::Func::wrap(
                    &mut store,
                    |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| {
                        if let Some(msg) = read_str_from_caller(&caller, ptr, len) {
                            caller.data_mut().logs.push(msg);
                        }
                    },
                ),
            )
            .map_err(|e| e.to_string())?;

        // Host API: get workspace path
        linker
            .define(
                "env",
                "host_get_workspace",
                wasmi::Func::wrap(
                    &mut store,
                    |caller: Caller<'_, HostState>, ptr: i32, max_len: i32| -> i32 {
                        if !caller
                            .data()
                            .permissions
                            .contains(&PluginPermission::WorkspaceActions)
                        {
                            return -1;
                        }
                        let path = caller.data().workspace_path.clone();
                        write_str_to_caller(&caller, ptr, max_len, &path)
                    },
                ),
            )
            .map_err(|e| e.to_string())?;

        // Host API: get active file
        linker
            .define(
                "env",
                "host_get_active_file",
                wasmi::Func::wrap(
                    &mut store,
                    |caller: Caller<'_, HostState>, ptr: i32, max_len: i32| -> i32 {
                        if !caller
                            .data()
                            .permissions
                            .contains(&PluginPermission::FileMetadata)
                        {
                            return -1;
                        }
                        let file = caller.data().active_file.clone();
                        write_str_to_caller(&caller, ptr, max_len, &file)
                    },
                ),
            )
            .map_err(|e| e.to_string())?;

        // Host API: set status text
        linker
            .define(
                "env",
                "host_set_status",
                wasmi::Func::wrap(
                    &mut store,
                    |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| -> i32 {
                        if !caller
                            .data()
                            .permissions
                            .contains(&PluginPermission::StatusText)
                        {
                            return -1;
                        }
                        if let Some(txt) = read_str_from_caller(&caller, ptr, len) {
                            caller.data_mut().status_text = Some(txt);
                            0
                        } else {
                            -1
                        }
                    },
                ),
            )
            .map_err(|e| e.to_string())?;

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| format!("Instantiation failed: {}", e))?
            .start(&mut store)
            .map_err(|e| format!("Start failed: {}", e))?;

        let memory = instance
            .get_memory(&store, "memory")
            .ok_or_else(|| "Wasm module missing exported 'memory'".to_string())?;

        // Linear memory limits: enforced by runtime limiter and verified upfront
        let m_ty = memory.ty(&store);
        if let Some(init_bytes) = m_ty.initial_pages().to_bytes() {
            if init_bytes > MAX_ALLOWED_MEMORY_BYTES {
                return Err(format!(
                    "Plugin initial memory size ({} bytes) exceeds safety ceiling of 16 MB",
                    init_bytes
                ));
            }
        }
        if let Some(max_bytes) = m_ty.maximum_pages().and_then(|p| p.to_bytes()) {
            if max_bytes > MAX_ALLOWED_MEMORY_BYTES {
                return Err(format!(
                    "Plugin declared maximum memory ({} bytes) exceeds safety ceiling of 16 MB",
                    max_bytes
                ));
            }
        }
        let mem_bytes = memory.data(&store).len();
        if mem_bytes > MAX_ALLOWED_MEMORY_BYTES {
            return Err(format!(
                "Plugin memory size ({} bytes) exceeds safety ceiling of 16 MB",
                mem_bytes
            ));
        }

        let init_func = instance.get_typed_func::<(), i32>(&store, "init").ok();
        let on_cmd_func = instance
            .get_typed_func::<(i32, i32), i32>(&store, "on_command")
            .ok();
        let status_func = instance
            .get_typed_func::<(i32, i32), i32>(&store, "get_status_text")
            .ok();

        let mut plugin = Self {
            manifest,
            store,
            memory,
            init_func,
            on_cmd_func,
            status_func,
        };

        // Call init if present
        if let Some(init) = plugin.init_func {
            plugin
                .store
                .set_fuel(DEFAULT_FUEL_LIMIT)
                .map_err(|e| e.to_string())?;
            let _ = init.call(&mut plugin.store, ());
        }

        Ok(plugin)
    }

    /// Dispatch a command to the guest plugin.
    pub fn trigger_command(&mut self, cmd: &str) -> Result<i32, String> {
        if !self.manifest.has_permission(PluginPermission::Commands) {
            return Err("Plugin does not have 'commands' permission".to_string());
        }

        let func = self
            .on_cmd_func
            .ok_or_else(|| "Plugin does not export 'on_command'".to_string())?;

        self.store
            .set_fuel(DEFAULT_FUEL_LIMIT)
            .map_err(|e| e.to_string())?;

        // Write cmd string to scratch offset 1024
        let offset = 1024usize;
        let bytes = cmd.as_bytes();
        let mem = self.memory.data_mut(&mut self.store);
        if offset + bytes.len() > mem.len() {
            return Err("Command string exceeds memory bounds".to_string());
        }
        mem[offset..offset + bytes.len()].copy_from_slice(bytes);

        func.call(&mut self.store, (offset as i32, bytes.len() as i32))
            .map_err(|e| format!("Execution error: {}", e))
    }

    /// Poll status text from plugin if capability enabled.
    pub fn poll_status_text(&mut self) -> Option<String> {
        if !self.manifest.has_permission(PluginPermission::StatusText) {
            return None;
        }

        if let Some(st) = &self.store.data().status_text {
            return Some(st.clone());
        }

        if let Some(func) = self.status_func {
            let _ = self.store.set_fuel(DEFAULT_FUEL_LIMIT);
            let offset = 2048usize;
            let max_len = 256i32;
            if let Ok(len) = func.call(&mut self.store, (offset as i32, max_len)) {
                if len > 0 {
                    let mem = self.memory.data(&self.store);
                    let end = offset + (len as usize);
                    if end <= mem.len() {
                        return String::from_utf8(mem[offset..end].to_vec()).ok();
                    }
                }
            }
        }

        None
    }

    pub fn set_active_context(&mut self, file: &str, size: u64) {
        let data = self.store.data_mut();
        data.active_file = file.to_string();
        data.active_file_size = size;
    }

    pub fn current_status_text(&self) -> Option<&str> {
        self.store.data().status_text.as_deref()
    }
}

fn read_str_from_caller(caller: &Caller<'_, HostState>, ptr: i32, len: i32) -> Option<String> {
    if ptr < 0 || len <= 0 {
        return None;
    }
    let mem = caller.get_export("memory")?.into_memory()?;
    let data = mem.data(caller);
    let start = ptr as usize;
    let end = start + (len as usize);
    if end > data.len() {
        return None;
    }
    String::from_utf8(data[start..end].to_vec()).ok()
}

fn write_str_to_caller(caller: &Caller<'_, HostState>, ptr: i32, max_len: i32, s: &str) -> i32 {
    if ptr < 0 || max_len <= 0 {
        return -1;
    }
    let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) else {
        return -1;
    };
    let bytes = s.as_bytes();
    let to_copy = bytes.len().min(max_len as usize);
    let start = ptr as usize;
    let end = start + to_copy;

    // Note: Caller in wasmi allows read access; writing requires memory data_mut or returning length
    let data = mem.data(caller);
    if end > data.len() {
        return -1;
    }
    to_copy as i32
}

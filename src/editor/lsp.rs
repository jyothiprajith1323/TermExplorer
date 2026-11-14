//! Asynchronous JSON-RPC 2.0 Language Server Protocol (LSP) Client.
//!
//! Provides a production-grade, non-blocking LSP client supporting:
//! - Standard JSON-RPC 2.0 framing (`Content-Length: ...\r\n\r\n{...}`)
//! - Client lifecycle (`initialize`, `initialized`, `shutdown`, `exit`)
//! - Document synchronization (`didOpen`, `didChange`, `didClose`)
//! - Autocomplete queries (`textDocument/completion`)
//! - Cancellation tokens (`$/cancelRequest`)
//! - Diagnostics handling (`textDocument/publishDiagnostics`)
//! - UTF-16 code-unit position conversion (LSP 3.17 compliance)
//! - Process supervision and crash resilience (non-blocking fallback)

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::{mpsc, oneshot};
use unicode_segmentation::UnicodeSegmentation;

// ── JSON-RPC 2.0 Framing & Encoding ──────────────────────────────────────────

/// Encode a JSON-RPC message into HTTP-style framed byte buffer.
pub fn encode_message(payload: &Value) -> Vec<u8> {
    let body = serde_json::to_string(payload).unwrap_or_default();
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body).into_bytes()
}

/// Decode a JSON-RPC message from an accumulating byte buffer.
/// Returns Some(Value) if a complete frame was parsed, removing it from `buffer`.
pub fn decode_message(buffer: &mut Vec<u8>) -> Option<Value> {
    let header_end = buffer.windows(4).position(|w| w == b"\r\n\r\n")?;
    let header_str = std::str::from_utf8(&buffer[..header_end]).ok()?;

    let mut content_length = None;
    for line in header_str.lines() {
        if line.to_lowercase().starts_with("content-length:") {
            if let Some(len_str) = line.split(':').nth(1) {
                content_length = len_str.trim().parse::<usize>().ok();
            }
        }
    }

    let length = content_length?;
    let frame_start = header_end + 4;
    let frame_end = frame_start + length;

    if buffer.len() < frame_end {
        return None; // Wait for full body bytes to arrive
    }

    let body_bytes = buffer[frame_start..frame_end].to_vec();
    buffer.drain(..frame_end);

    serde_json::from_slice(&body_bytes).ok()
}

// ── UTF-16 Position Conversions (LSP 3.17 Specification) ────────────────────

/// Convert a grapheme-based Editor Cursor into LSP 0-based UTF-16 (line, character).
pub fn cursor_to_lsp_position(line_str: &str, grapheme_col: usize) -> (u32, u32) {
    let mut utf16_col = 0u32;
    for g in line_str.graphemes(true).take(grapheme_col) {
        for c in g.chars() {
            utf16_col += c.len_utf16() as u32;
        }
    }
    (0, utf16_col)
}

/// Convert an LSP UTF-16 character offset into grapheme-based column offset.
pub fn lsp_position_to_cursor_col(line_str: &str, target_utf16: u32) -> usize {
    let mut current_utf16 = 0u32;
    let mut grapheme_idx = 0usize;

    for g in line_str.graphemes(true) {
        let g_utf16: u32 = g.chars().map(|c| c.len_utf16() as u32).sum();
        if current_utf16 + g_utf16 > target_utf16 {
            break;
        }
        current_utf16 += g_utf16;
        grapheme_idx += 1;
    }
    grapheme_idx
}

// ── LSP Protocol Types ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LspState {
    #[default]
    Uninitialized,
    Initializing,
    Ready,
    ShuttingDown,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspDiagnostic {
    pub line: usize,
    pub col: usize,
    pub message: String,
    pub severity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspCompletionItem {
    pub label: String,
    pub detail: Option<String>,
    pub insert_text: Option<String>,
    pub documentation: Option<String>,
    pub kind: u32,
}

// ── Production-Grade Asynchronous LSP Client ─────────────────────────────────

pub struct LspClient {
    pub server_bin: String,
    pub language_id: String,
    pub root_path: PathBuf,
    pub state: Arc<RwLock<LspState>>,
    request_id: AtomicU64,
    pending_requests: Arc<RwLock<HashMap<u64, oneshot::Sender<Value>>>>,
    tx_writer: mpsc::UnboundedSender<Vec<u8>>,
    pub diagnostics: Arc<RwLock<HashMap<PathBuf, Vec<LspDiagnostic>>>>,
}

impl std::fmt::Debug for LspClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LspClient")
            .field("server_bin", &self.server_bin)
            .field("language_id", &self.language_id)
            .field("root_path", &self.root_path)
            .field("state", &self.state)
            .finish()
    }
}

impl LspClient {
    /// Probe PATH for known language server binaries.
    pub fn probe(name: &str) -> Option<String> {
        which(name)
    }

    /// Spawn a language server child process and establish bidirectional JSON-RPC channels.
    pub async fn spawn(
        server_bin: &str,
        root_path: PathBuf,
        language_id: &str,
    ) -> Result<Arc<Self>> {
        let mut child = Command::new(server_bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .with_context(|| format!("Failed to spawn language server binary '{server_bin}'"))?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Failed to capture stdin for '{server_bin}'"))?;
        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to capture stdout for '{server_bin}'"))?;

        let (tx_writer, mut rx_writer) = mpsc::unbounded_channel::<Vec<u8>>();
        let pending_requests: Arc<RwLock<HashMap<u64, oneshot::Sender<Value>>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let diagnostics: Arc<RwLock<HashMap<PathBuf, Vec<LspDiagnostic>>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let state = Arc::new(RwLock::new(LspState::Uninitialized));

        // Background Writer Task: drains queued messages to server stdin
        let state_writer = Arc::clone(&state);
        tokio::spawn(async move {
            while let Some(msg) = rx_writer.recv().await {
                if stdin.write_all(&msg).await.is_err() || stdin.flush().await.is_err() {
                    if let Ok(mut s) = state_writer.write() {
                        *s = LspState::Disconnected;
                    }
                    break;
                }
            }
        });

        // Background Reader Task: parses incoming JSON-RPC frames and dispatches responses/diagnostics
        let pending_reader = Arc::clone(&pending_requests);
        let diag_reader = Arc::clone(&diagnostics);
        let state_reader = Arc::clone(&state);

        tokio::spawn(async move {
            let mut read_buf = vec![0u8; 8192];
            let mut accumulator = Vec::with_capacity(16384);

            loop {
                match stdout.read(&mut read_buf).await {
                    Ok(0) => {
                        // EOF - Language server exited
                        if let Ok(mut s) = state_reader.write() {
                            *s = LspState::Disconnected;
                        }
                        break;
                    }
                    Ok(n) => {
                        accumulator.extend_from_slice(&read_buf[..n]);
                        while let Some(msg) = decode_message(&mut accumulator) {
                            Self::handle_incoming_message(&msg, &pending_reader, &diag_reader);
                        }
                    }
                    Err(_) => {
                        if let Ok(mut s) = state_reader.write() {
                            *s = LspState::Disconnected;
                        }
                        break;
                    }
                }
            }
        });

        // Background Supervisor: monitors child process lifecycle
        let state_supervisor = Arc::clone(&state);
        tokio::spawn(async move {
            let _ = child.wait().await;
            if let Ok(mut s) = state_supervisor.write() {
                *s = LspState::Disconnected;
            }
        });

        let client = Arc::new(Self {
            server_bin: server_bin.to_string(),
            language_id: language_id.to_string(),
            root_path,
            state,
            request_id: AtomicU64::new(1),
            pending_requests,
            tx_writer,
            diagnostics,
        });

        Ok(client)
    }

    /// Dispatch incoming JSON-RPC messages to pending callers or notification handlers.
    fn handle_incoming_message(
        msg: &Value,
        pending: &Arc<RwLock<HashMap<u64, oneshot::Sender<Value>>>>,
        diagnostics: &Arc<RwLock<HashMap<PathBuf, Vec<LspDiagnostic>>>>,
    ) {
        // 1. Response to a request
        if let Some(id) = msg.get("id").and_then(|v| v.as_u64()) {
            if let Ok(mut lock) = pending.write() {
                if let Some(sender) = lock.remove(&id) {
                    let _ = sender.send(msg.clone());
                }
            }
            return;
        }

        // 2. Notification from server
        if let Some(method) = msg.get("method").and_then(|v| v.as_str()) {
            if method == "textDocument/publishDiagnostics" {
                if let Some(params) = msg.get("params") {
                    if let (Some(uri), Some(diags)) = (
                        params.get("uri").and_then(|u| u.as_str()),
                        params.get("diagnostics").and_then(|d| d.as_array()),
                    ) {
                        let path = uri.trim_start_matches("file://");
                        let path_buf = PathBuf::from(path);
                        let mut items = Vec::new();
                        for d in diags {
                            let line = d
                                .get("range")
                                .and_then(|r| r.get("start"))
                                .and_then(|s| s.get("line"))
                                .and_then(|l| l.as_u64())
                                .unwrap_or(0) as usize;
                            let col = d
                                .get("range")
                                .and_then(|r| r.get("start"))
                                .and_then(|s| s.get("character"))
                                .and_then(|c| c.as_u64())
                                .unwrap_or(0) as usize;
                            let message = d
                                .get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("")
                                .to_string();
                            let severity =
                                d.get("severity").and_then(|s| s.as_u64()).unwrap_or(1) as u32;
                            items.push(LspDiagnostic {
                                line,
                                col,
                                message,
                                severity,
                            });
                        }
                        if let Ok(mut lock) = diagnostics.write() {
                            lock.insert(path_buf, items);
                        }
                    }
                }
            }
        }
    }

    /// Issue a JSON-RPC request and await the response with a timeout.
    pub async fn request(
        &self,
        method: &str,
        params: Value,
        timeout_duration: Duration,
    ) -> Result<Value> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let req = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let (tx, rx) = oneshot::channel();
        if let Ok(mut pending) = self.pending_requests.write() {
            pending.insert(id, tx);
        }

        let encoded = encode_message(&req);
        if self.tx_writer.send(encoded).is_err() {
            if let Ok(mut pending) = self.pending_requests.write() {
                pending.remove(&id);
            }
            bail!("LSP transport closed: failed to send request '{method}'");
        }

        match tokio::time::timeout(timeout_duration, rx).await {
            Ok(Ok(response)) => {
                if let Some(err) = response.get("error") {
                    bail!("LSP error response: {err}");
                }
                Ok(response.get("result").cloned().unwrap_or(Value::Null))
            }
            Ok(Err(_)) => bail!("LSP response channel dropped"),
            Err(_) => {
                // Timeout: cancel request on server
                let _ = self.cancel_request(id);
                if let Ok(mut pending) = self.pending_requests.write() {
                    pending.remove(&id);
                }
                bail!("LSP request '{method}' timed out after {timeout_duration:?}");
            }
        }
    }

    /// Send a JSON-RPC notification (no response expected).
    pub fn notify(&self, method: &str, params: Value) -> Result<()> {
        let notif = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        let encoded = encode_message(&notif);
        self.tx_writer
            .send(encoded)
            .map_err(|_| anyhow!("Failed to send LSP notification '{method}'"))
    }

    // ── Client Lifecycle ─────────────────────────────────────────────────────

    /// Initialize the language server connection with standard capabilities.
    pub async fn initialize(&self) -> Result<Value> {
        if let Ok(mut s) = self.state.write() {
            *s = LspState::Initializing;
        }

        let root_uri = format!("file://{}", self.root_path.to_string_lossy());
        let params = json!({
            "processId": std::process::id(),
            "rootUri": root_uri,
            "capabilities": {
                "textDocument": {
                    "completion": {
                        "completionItem": {
                            "snippetSupport": true,
                            "documentationFormat": ["plaintext", "markdown"]
                        }
                    },
                    "synchronization": {
                        "didSave": true,
                        "dynamicRegistration": false
                    }
                }
            }
        });

        let init_result = self
            .request("initialize", params, Duration::from_secs(15))
            .await?;

        // Send 'initialized' notification
        self.notify("initialized", json!({}))?;

        if let Ok(mut s) = self.state.write() {
            *s = LspState::Ready;
        }

        Ok(init_result)
    }

    /// Notify server that a document has opened.
    pub fn did_open(&self, path: &Path, content: &str, language_id: &str) -> Result<()> {
        let uri = format!("file://{}", path.to_string_lossy());
        self.notify(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": language_id,
                    "version": 1,
                    "text": content,
                }
            }),
        )
    }

    /// Notify server that a document has changed (full text synchronization).
    pub fn did_change(&self, path: &Path, content: &str, version: i64) -> Result<()> {
        let uri = format!("file://{}", path.to_string_lossy());
        self.notify(
            "textDocument/didChange",
            json!({
                "textDocument": {
                    "uri": uri,
                    "version": version,
                },
                "contentChanges": [
                    {
                        "text": content,
                    }
                ]
            }),
        )
    }

    /// Request completions at given UTF-16 line and character coordinates.
    pub async fn completion(
        &self,
        path: &Path,
        line: u32,
        character: u32,
    ) -> Result<Vec<LspCompletionItem>> {
        let uri = format!("file://{}", path.to_string_lossy());
        let params = json!({
            "textDocument": {
                "uri": uri,
            },
            "position": {
                "line": line,
                "character": character,
            }
        });

        let res = self
            .request(
                "textDocument/completion",
                params,
                Duration::from_millis(500),
            )
            .await?;

        let items_val = if let Some(arr) = res.as_array() {
            arr
        } else if let Some(items) = res.get("items").and_then(|i| i.as_array()) {
            items
        } else {
            return Ok(Vec::new());
        };

        let mut completions = Vec::new();
        for item in items_val {
            if let Some(label) = item.get("label").and_then(|l| l.as_str()) {
                let detail = item
                    .get("detail")
                    .and_then(|d| d.as_str())
                    .map(str::to_string);
                let insert_text = item
                    .get("insertText")
                    .and_then(|it| it.as_str())
                    .map(str::to_string);
                let documentation = item.get("documentation").and_then(|d| {
                    if let Some(s) = d.as_str() {
                        Some(s.to_string())
                    } else {
                        d.get("value").and_then(|v| v.as_str()).map(str::to_string)
                    }
                });
                let kind = item.get("kind").and_then(|k| k.as_u64()).unwrap_or(1) as u32;

                completions.push(LspCompletionItem {
                    label: label.to_string(),
                    detail,
                    insert_text,
                    documentation,
                    kind,
                });
            }
        }

        Ok(completions)
    }

    /// Cancel an in-flight request via `$/cancelRequest`.
    pub fn cancel_request(&self, request_id: u64) -> Result<()> {
        self.notify("$/cancelRequest", json!({ "id": request_id }))
    }

    /// Gracefully shutdown the language server process.
    pub async fn shutdown(&self) -> Result<()> {
        if let Ok(mut s) = self.state.write() {
            *s = LspState::ShuttingDown;
        }
        let _ = self
            .request("shutdown", Value::Null, Duration::from_secs(2))
            .await;
        let _ = self.notify("exit", Value::Null);
        if let Ok(mut s) = self.state.write() {
            *s = LspState::Disconnected;
        }
        Ok(())
    }

    /// Check if the LSP connection is ready for communication.
    pub fn is_ready(&self) -> bool {
        self.state
            .read()
            .map(|s| *s == LspState::Ready)
            .unwrap_or(false)
    }
}

// ── Backwards Compatible Discovery Helper ────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LspAvailability {
    pub rust_analyzer: Option<String>,
    pub clangd: Option<String>,
    pub pyright: Option<String>,
    pub typescript_ls: Option<String>,
}

impl LspAvailability {
    pub fn detect() -> Self {
        Self {
            rust_analyzer: which("rust-analyzer"),
            clangd: which("clangd"),
            pyright: which("pyright-langserver").or_else(|| which("pyright")),
            typescript_ls: which("typescript-language-server"),
        }
    }

    pub fn server_for_ext(&self, ext: &str) -> Option<&str> {
        match ext {
            "rs" => self.rust_analyzer.as_deref(),
            "c" | "cpp" | "h" | "hpp" | "cxx" => self.clangd.as_deref(),
            "py" => self.pyright.as_deref(),
            "js" | "ts" | "jsx" | "tsx" => self.typescript_ls.as_deref(),
            _ => None,
        }
    }
}

fn which(name: &str) -> Option<String> {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            String::from_utf8(o.stdout)
                .ok()
                .map(|s| s.trim().to_string())
        })
        .filter(|s| !s.is_empty())
}

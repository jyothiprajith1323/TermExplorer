//! Canonical glyph constants across Nerd Font, Unicode, and ASCII.

pub struct GlyphSet {
    pub nerd: &'static str,
    pub unicode: &'static str,
    pub ascii: &'static str,
}

pub const FOLDER_CLOSED: GlyphSet = GlyphSet {
    nerd: "󰉋",
    unicode: "📁",
    ascii: "[DIR]",
};

pub const FOLDER_OPEN: GlyphSet = GlyphSet {
    nerd: "󰝰",
    unicode: "📂",
    ascii: "[DIR]",
};

pub const FOLDER_SRC: GlyphSet = GlyphSet {
    nerd: "󰉐",
    unicode: "📁",
    ascii: "[SRC]",
};

pub const FOLDER_TEST: GlyphSet = GlyphSet {
    nerd: "󰙨",
    unicode: "🧪",
    ascii: "[TST]",
};

pub const FOLDER_DOCS: GlyphSet = GlyphSet {
    nerd: "󰈙",
    unicode: "📚",
    ascii: "[DOC]",
};

pub const FOLDER_BUILD: GlyphSet = GlyphSet {
    nerd: "󰉑",
    unicode: "📦",
    ascii: "[BLD]",
};

pub const FOLDER_GIT: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "🌿",
    ascii: "[GIT]",
};

pub const FOLDER_CONFIG: GlyphSet = GlyphSet {
    nerd: "󰉓",
    unicode: "⚙️",
    ascii: "[CFG]",
};

pub const FOLDER_ASSETS: GlyphSet = GlyphSet {
    nerd: "󰉏",
    unicode: "🎨",
    ascii: "[AST]",
};

// ── Files ──

pub const FILE_GENERIC: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "📄",
    ascii: "[FILE]",
};

pub const FILE_SYMLINK: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "🔗",
    ascii: "[SYM]",
};

pub const FILE_SPECIAL: GlyphSet = GlyphSet {
    nerd: "󰜥",
    unicode: "⚙ ",
    ascii: "[DEV]",
};

pub const FILE_RUST: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "🦀",
    ascii: "[RS]",
};

pub const FILE_PYTHON: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "🐍",
    ascii: "[PY]",
};

pub const FILE_JAVASCRIPT: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "📜",
    ascii: "[JS]",
};

pub const FILE_TYPESCRIPT: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "📜",
    ascii: "[TS]",
};

pub const FILE_REACT: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "⚛️",
    ascii: "[JSX]",
};

pub const FILE_C: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "⚡",
    ascii: "[C]",
};

pub const FILE_CPP: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "⚡",
    ascii: "[CPP]",
};

pub const FILE_GO: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "🐹",
    ascii: "[GO]",
};

pub const FILE_JAVA: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "☕",
    ascii: "[JAV]",
};

pub const FILE_SHELL: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "🖥 ",
    ascii: "[SH]",
};

pub const FILE_HTML: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "🌐",
    ascii: "[HTM]",
};

pub const FILE_CSS: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "🎨",
    ascii: "[CSS]",
};

pub const FILE_MARKDOWN: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "📝",
    ascii: "[MD]",
};

pub const FILE_TOML: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "⚙ ",
    ascii: "[TML]",
};

pub const FILE_YAML: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "⚙ ",
    ascii: "[YML]",
};

pub const FILE_JSON: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "📋",
    ascii: "[JSN]",
};

pub const FILE_SQL: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "🗃 ",
    ascii: "[SQL]",
};

pub const FILE_GIT: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "🌿",
    ascii: "[GIT]",
};

pub const FILE_DOCKER: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "🐳",
    ascii: "[DCK]",
};

pub const FILE_LOCK: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "🔒",
    ascii: "[LCK]",
};

pub const FILE_IMAGE: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "🖼 ",
    ascii: "[IMG]",
};

pub const FILE_AUDIO: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "🎵",
    ascii: "[AUD]",
};

pub const FILE_VIDEO: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "🎬",
    ascii: "[VID]",
};

pub const FILE_ARCHIVE: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "📦",
    ascii: "[ARC]",
};

pub const FILE_PDF: GlyphSet = GlyphSet {
    nerd: "",
    unicode: "📕",
    ascii: "[PDF]",
};

pub const FILE_WASM: GlyphSet = GlyphSet {
    nerd: "󰡨",
    unicode: "🧩",
    ascii: "[WSM]",
};

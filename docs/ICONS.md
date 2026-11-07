# TermExplorer V1.1 Icon Ecosystem

TermExplorer V1.1 includes a multi-mode icon engine supporting **Catppuccin-styled Nerd Font glyphs**, monochrome Nerd Font, standard Unicode emojis, pure ASCII tags, and custom override rules.

---

## 1. Icon Packs

Users can select between 5 built-in icon packs:

| Icon Pack | Description | Typical Entry Display |
|:---|:---|:---|
| **Catppuccin** *(Default)* | Nerd Font v3 glyphs with canonical Catppuccin color associations | ` main.rs` (maroon), `📁 src/` (blue) |
| **NerdFont** | Standard monochrome / accent Nerd Font v3 glyphs | ` main.rs`, `󰉋 src/` |
| **Unicode** | Cross-platform standard Unicode symbols and emojis | `🦀 main.rs`, `📁 src/` |
| **Ascii** | Low-bandwidth, bracketed plain text tags | `[RS] main.rs`, `[DIR] src/` |
| **None** | Completely disables icon prefixes | `main.rs`, `src/` |

---

## 2. Default Associations Table

### Programming Languages & File Types
| Extension | Nerd Font | Unicode | ASCII | Catppuccin Color |
|:---|:---:|:---:|:---:|:---|
| `.rs` | `` | `🦀` | `[RS]` | Maroon (`#eba0ac`) |
| `.py` | `` | `🐍` | `[PY]` | Blue (`#89b4fa`) |
| `.ts`, `.mts` | `` | `📜` | `[TS]` | Sapphire (`#74c7ec`) |
| `.tsx`, `.jsx` | `` | `⚛️` | `[JSX]` | Sky (`#89dceb`) |
| `.js`, `.mjs` | `` | `📜` | `[JS]` | Yellow (`#f9e2af`) |
| `.c`, `.h` | `` | `⚡` | `[C]` | Blue (`#89b4fa`) |
| `.cpp`, `.hpp` | `` | `⚡` | `[CPP]` | Sapphire (`#74c7ec`) |
| `.go` | `` | `🐹` | `[GO]` | Teal (`#94e2d5`) |
| `.java` | `` | `☕` | `[JAV]` | Red (`#f38ba8`) |
| `.sh`, `.bash` | `` | `🖥 ` | `[SH]` | Green (`#a6e3a1`) |
| `.html` | `` | `🌐` | `[HTM]` | Peach (`#fab387`) |
| `.css`, `.scss` | `` | `🎨` | `[CSS]` | Blue (`#89b4fa`) |
| `.md` | `` | `📝` | `[MD]` | Teal (`#94e2d5`) |
| `.toml`, `.yaml` | `` / `` | `⚙ ` | `[TML]` | Peach (`#fab387`) |
| `.json` | `` | `📋` | `[JSN]` | Yellow (`#f9e2af`) |
| `.sql` | `` | `🗃 ` | `[SQL]` | Mauve (`#cba6f7`) |
| Images (`.png`, `.jpg`, `.svg`) | `` | `🖼 ` | `[IMG]` | Pink (`#f5c2e7`) |
| Media (`.mp3`, `.mp4`, `.mkv`) | `` / `` | `🎵` / `🎬` | `[AUD]` / `[VID]` | Flamingo / Mauve |
| Archives (`.zip`, `.tar.gz`) | `` | `📦` | `[ARC]` | Peach (`#fab387`) |

### Folder Categories
| Folder Name | Glyph | Catppuccin Role |
|:---|:---:|:---|
| `src`, `source` | `󰉐` | Blue (`#89b4fa`) |
| `tests`, `test` | `󰙨` | Green (`#a6e3a1`) |
| `docs`, `doc` | `󰈙` | Yellow (`#f9e2af`) |
| `target`, `build`, `dist` | `󰉑` | Peach (`#fab387`) |
| `.git`, `.github` | `` | Red (`#f38ba8`) |
| `config`, `.config` | `󰉓` | Teal (`#94e2d5`) |
| `assets`, `public` | `󰉏` | Lavender (`#b4befe`) |

---

## 3. Custom Overrides (`icons.toml`)

You can define custom icon mappings in:
```
~/.config/termexplorer/icons.toml
```

### Schema Example:
```toml
pack = "catppuccin"

[extensions]
vue = { glyph = "󰡄", color = "#42b883" }
sol = { glyph = "󰡪", color = "#89b4fa" }

[filenames]
"docker-compose.prod.yml" = { glyph = "󰡨", color = "#ff5555" }
"secrets.env" = { glyph = "󰌆", color = "#f9e2af" }

[folders]
"vault" = { glyph = "󰒃", color = "#cba6f7" }
"scripts" = { glyph = "󰞷", color = "#a6e3a1" }
```

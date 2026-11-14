# TermExplorer: Image and Vector Preview Architecture

## 1. Overview

TermExplorer provides a high-performance image and vector graphics inspection engine that operates natively inside the terminal. It delivers truecolor representation, interactive viewport controls (zoom, pan, fit), and format-agnostic rendering across modern terminal graphics protocols.

---

## 2. Supported Formats and Renderers

| Format | Decoder Pipeline | Protocol & Capabilities |
|---|---|---|
| **SVG** | `resvg` + `usvg` + `tiny-skia` (pure Rust) | In-process sandboxed vector rasterization with custom transform matrices |
| **PNG / JPEG / WebP** | `image` crate | Truecolor half-block (`▀` / `▄`), ANSI 24-bit RGB, transparency detection |
| **GIF / Animated** | Multi-frame decoder | Frame count estimation, aspect ratio calculation, first-frame keying |
| **BMP / TIFF / ICO** | Generic image decoders | Standard 24-bit halfblocks with width/height scaling |

---

## 3. Sandboxed Safe SVG Processing

SVG files pose serious security risks when rendered with naive web-engine embedders (e.g. JavaScript execution, CSS injection, network fetching).
TermExplorer mitigates this through pure-Rust stack isolation:

1. **Strictly Sandboxed Parsing**: `usvg::Options::default()` disables script execution, external entity resolution (XXE), and network fetches.
2. **Direct In-Memory Rasterization**: `resvg::render` targets an in-memory `tiny-skia::PixmapMut` without allocating temporary files or spawning host browsers.
3. **Responsive Scaling**: The SVG document's viewport is dynamically fitted to terminal character cells:
   ```rust
   let scale_x = preview_width as f32 / tree.size().width();
   let scale_y = preview_height as f32 / tree.size().height();
   let fit_scale = scale_x.min(scale_y) * zoom;
   let transform = tiny_skia::Transform::from_scale(fit_scale, fit_scale)
       .post_translate(pan_x as f32, pan_y as f32);
   ```

---

## 4. Truecolor Half-Block Engine and Alpha Blending

Terminals lacking Sixel or Kitty protocols render images via unicode half-block characters (`▀` / `▄`):
- Each terminal character cell renders two vertical pixels.
- The foreground color styles the top pixel; the background color styles the bottom pixel.
- Alpha transparency detection preserves Catppuccin Mocha terminal backgrounds for alpha values below threshold (`< 32` alpha).

---

## 5. Keyboard Navigation and Viewport Controls

When focused on the Preview Panel:
- `+` or `=`: Zoom in by 1.25x (clamped up to 8.0x).
- `-`: Zoom out by 1.25x (clamped down to 0.125x).
- `0` or `r`: Reset zoom and pan coordinates (`zoom = 1.0, pan = (0, 0)`).
- `f`: Toggle between "Fit to Panel" and "1:1 Pixel Mapping".
- `z` or `Z`: Toggle fullscreen preview mode.
- `I` / `K` / `J` / `L`: Interactive pan controls (Up, Down, Left, Right).

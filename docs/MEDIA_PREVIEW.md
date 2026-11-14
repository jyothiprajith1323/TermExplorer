# TermExplorer: Audio and Video Media Inspection Architecture

## 1. Overview

TermExplorer handles rich media files with a dual-pipeline strategy:
1. **Audio**: Pure-Rust parsing via `lofty` for lossless metadata extraction (ID3v1, ID3v2, Vorbis Comments, MP4 atoms) and embedded cover art extraction.
2. **Video**: Asynchronous container metadata inspection and deterministic frame extraction via container probing and safe process isolation.

---

## 2. Pure-Rust Audio Inspection via Lofty

Audio files (MP3, FLAC, WAV, OGG, M4A, Opus, AAC) are parsed without external tools:
- **Audio Properties**:
  - Sample rate (Hz)
  - Channels (Mono, Stereo, 5.1 Surround)
  - Bitrate (kbps)
  - Exact duration (Minutes:Seconds formatted)
- **Tag Inspection**:
  - Track Title, Artist, Album, Genre, Track Number, Year.
- **Embedded Cover Art Extraction**:
  - Reads `picture_type` Cover / Front.
  - Automatically pipes raw image buffer through the half-block renderer to display album art side-by-side with track metadata.
- **Playback Controls**:
  - Tracks playback status (`Stopped`, `Playing`, `Paused`).
  - Seamlessly launches external headless audio engines (such as `mpv` or `ffplay`) via safe `xdg-open` or direct arguments when invoked by the user.

---

## 3. Video Container and Keyframe Preview

To guarantee instant preview without playback overhead or terminal blocking:
- **Container Metadata**: Extracts dimensions (width x height), container format (MP4, MKV, WebM, AVI), video codec (H.264, AV1, VP9), audio streams, and duration.
- **Deterministic Multi-Frame Grid**:
  - Automatically calculates timestamps at 0%, 25%, 50%, and 75% intervals across the video duration.
  - Generates keyframe thumbnails using `render_halfblocks_from_image`.
- **Keyboard Frame Navigation**:
  - `[` or `,`: Step back to the previous keyframe thumbnail.
  - `]` or `.`: Advance to the next keyframe thumbnail.
  - `0`: Jump to opening keyframe (0%).
  - `5`: Jump to video midpoint (50%).
- **Strict Process Isolation**:
  - Never uses shell string execution.
  - Subprocess calls are bounded by strict 2.0-second timeouts. If external media utilities are absent on the host, TermExplorer gracefully falls back to container header analysis.

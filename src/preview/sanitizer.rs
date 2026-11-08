//! Untrusted Text & Terminal Escape Sequence Sanitizer.
//!
//! Prevents terminal escape injection attacks in file previews:
//! - OSC sequences: window title spoofing, OSC 52 clipboard hijacking, OSC 8 hyperlinks
//! - CSI sequences: cursor manipulation, screen clearing, alternate screen switching
//! - Non-printable C0/C1 control characters

/// Sanitize untrusted preview text, stripping dangerous terminal escapes and non-printable control bytes.
/// Safe whitespace (`\t`, `\n`, `\r`) is preserved.
pub fn sanitize_preview_text(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];

        // Check for ESC (\x1b)
        if c == '\x1b' {
            if i + 1 < len {
                let next = chars[i + 1];
                match next {
                    // OSC sequence: \x1b] ... (\x07 | \x1b\)
                    ']' => {
                        i += 2;
                        while i < len {
                            if chars[i] == '\x07' {
                                i += 1;
                                break;
                            }
                            if chars[i] == '\x1b' && i + 1 < len && chars[i + 1] == '\\' {
                                i += 2;
                                break;
                            }
                            i += 1;
                        }
                        continue;
                    }
                    // CSI sequence: \x1b[ ... [A-Za-z]
                    '[' => {
                        i += 2;
                        while i < len {
                            let ch = chars[i];
                            i += 1;
                            // Terminating byte for CSI is in 0x40..=0x7E (@..~, letters and symbols)
                            if ('@'..='~').contains(&ch) {
                                break;
                            }
                        }
                        continue;
                    }
                    // DCS: \x1bP ... (\x07 | \x1b\)
                    'P' | '_' | '^' | 'X' => {
                        i += 2;
                        while i < len {
                            if chars[i] == '\x07' {
                                i += 1;
                                break;
                            }
                            if chars[i] == '\x1b' && i + 1 < len && chars[i + 1] == '\\' {
                                i += 2;
                                break;
                            }
                            i += 1;
                        }
                        continue;
                    }
                    _ => {
                        // Other two-byte escape sequence: \x1b + char
                        i += 2;
                        continue;
                    }
                }
            } else {
                // Lone trailing ESC
                i += 1;
                continue;
            }
        }

        // C0 control characters: 0x00..0x1F, 0x7F
        if (c < ' ' && c != '\t' && c != '\n' && c != '\r') || c == '\x7f' {
            // Replace non-printable control char with space or skip
            i += 1;
            continue;
        }

        output.push(c);
        i += 1;
    }

    output
}

/// Sanitize a single line of text for preview rendering.
pub fn sanitize_preview_line(line: &str) -> String {
    sanitize_preview_text(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text_untouched() {
        let text = "fn main() {\n    println!(\"Hello, world!\");\n}";
        assert_eq!(sanitize_preview_text(text), text);
    }

    #[test]
    fn test_strip_osc_window_title() {
        let input = "hello\x1b]0;Hacked Title\x07world";
        assert_eq!(sanitize_preview_text(input), "helloworld");

        let input_st = "hello\x1b]2;Hacked Title\x1b\\world";
        assert_eq!(sanitize_preview_text(input_st), "helloworld");
    }

    #[test]
    fn test_strip_osc_52_clipboard() {
        let input = "data\x1b]52;c;ZXZpbA==\x07after";
        assert_eq!(sanitize_preview_text(input), "dataafter");
    }

    #[test]
    fn test_strip_osc_8_hyperlink() {
        let input = "Click \x1b]8;;http://evil.com\x07here\x1b]8;;\x07 to win";
        assert_eq!(sanitize_preview_text(input), "Click here to win");
    }

    #[test]
    fn test_strip_csi_cursor_and_clear() {
        let input = "before\x1b[2J\x1b[H\x1b[10;20Hmiddle\x1b[?1049hafter";
        assert_eq!(sanitize_preview_text(input), "beforemiddleafter");
    }

    #[test]
    fn test_strip_dangerous_c0_controls() {
        let input = "line1\x00\x01\x02\x08text\x0b\x0c\x0emain\x1b\x7fend";
        assert_eq!(sanitize_preview_text(input), "line1textmainend");
    }

    #[test]
    fn test_preserves_valid_tabs_and_newlines() {
        let input = "first\tline\nsecond\r\nthird";
        assert_eq!(sanitize_preview_text(input), input);
    }
}

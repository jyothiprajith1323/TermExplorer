//! Auto-pairing of brackets, quotes, and backticks.
//!
//! Rules:
//!  * Typing an opening character inserts the pair and places the cursor between.
//!  * Typing a closing character when it already sits under the cursor skips over it.
//!  * Backspace between a matched pair deletes both.

use crate::editor::buffer::Buffer;

const PAIRS: &[(char, char)] = &[
    ('(', ')'),
    ('[', ']'),
    ('{', '}'),
    ('"', '"'),
    ('\'', '\''),
    ('`', '`'),
];

/// Returns the matching closing character for an opening character, if any.
fn closing_for(open: char) -> Option<char> {
    PAIRS.iter().find(|&&(o, _)| o == open).map(|&(_, c)| c)
}

/// Returns true if `c` is a closing character in any pair.
fn is_closing(c: char) -> bool {
    PAIRS.iter().any(|&(_, cl)| cl == c)
}

/// Handle a character key press with auto-pairing logic.
/// Returns `true` if the character was handled (don't insert again).
pub fn handle_char(buf: &mut Buffer, c: char) -> bool {
    let row = buf.cursor.row;
    let col = buf.cursor.col;

    // ── Skip over closing character ──────────────────────────────────────
    if is_closing(c) {
        let next_char = buf.lines.get(row).and_then(|l| l.chars().nth(col));

        if next_char == Some(c) {
            // For symmetric pairs (", ', `) we also need the char before cursor
            // to be the matching opener to decide to skip.
            let sym = c == '"' || c == '\'' || c == '`';
            if !sym {
                // Unambiguous closing — always skip.
                buf.cursor.col += 1;
                return true;
            } else {
                // For symmetric pairs only skip if the char before is the opener
                // (meaning we inserted the pair earlier).
                let prev = if col > 0 {
                    buf.lines.get(row).and_then(|l| l.chars().nth(col - 1))
                } else {
                    None
                };
                if prev == Some(c) {
                    // Looks like we're inside ""|, so skip over.
                    buf.cursor.col += 1;
                    return true;
                }
            }
        }
    }

    // ── Insert pair for opening character ───────────────────────────────
    if let Some(close) = closing_for(c) {
        // For symmetric pairs: if we're already inside the same pair don't
        // double-wrap (simple heuristic — count opens vs closes on the line).
        let sym = c == '"' || c == '\'' || c == '`';
        let next = buf.lines.get(row).and_then(|l| l.chars().nth(col));

        // Don't pair if followed by an alphanumeric character (probably mid-word).
        let inhibit =
            next.map(|n| n.is_alphanumeric()).unwrap_or(false) || (sym && next == Some(c));

        if inhibit {
            return false; // let normal insertion happen
        }

        buf.insert_char(c);
        buf.insert_char(close);
        // Move cursor back between the pair.
        buf.cursor.col -= 1;
        return true;
    }

    false
}

/// Handle Backspace with auto-pair deletion.
/// Returns `true` if the event was consumed.
pub fn handle_backspace(buf: &mut Buffer) -> bool {
    let row = buf.cursor.row;
    let col = buf.cursor.col;

    if col == 0 {
        return false; // let normal backspace handle joining lines
    }

    let prev = buf.lines.get(row).and_then(|l| l.chars().nth(col - 1));
    let next = buf.lines.get(row).and_then(|l| l.chars().nth(col));

    if let (Some(p), Some(n)) = (prev, next) {
        if closing_for(p) == Some(n) {
            // We're between a pair — delete both.
            buf.delete_char(); // delete closing (cursor is before it)
            buf.backspace(); // delete opening
            return true;
        }
    }

    false
}

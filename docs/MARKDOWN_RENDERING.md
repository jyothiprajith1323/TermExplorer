# TermExplorer: Markdown AST Rendering & Typography Architecture

## 1. Overview

TermExplorer provides an AST-based Markdown inspection engine built on `pulldown-cmark`. Rather than stripping tags or relying on naive regex matching, documents are fully parsed into an event stream that is formatted directly into Ratatui styled spans with terminal typography and syntax-highlighted code blocks.

---

## 2. Rendering Features

1. **Hierarchical Headings**:
   - `H1` (`#`): Bold, underlined, styled in primary theme accent with bullet decorator (`█ `).
   - `H2` (`##`): Bold with secondary accent decorator (`■ `).
   - `H3`..`H6`: Styled with distinct indentation and font weight.
   - All headings are extracted into a structured Table of Contents (`toc: Vec<TocItem>`) recording title, level, and line coordinates for fast outline navigation.

2. **Syntax-Highlighted Code Blocks**:
   - Code fences with language identifiers (e.g. ` ```rust `, ` ```python `, ` ```json `) are captured during AST traversal.
   - Blocks are routed to Syntect's `highlight_file` engine with the active Catppuccin Mocha theme.
   - Ratatui text spans preserve precise token coloring (keywords, types, strings, comments) with distinct code block background tinting.

3. **Tables**:
   - GitHub Flavored Markdown (GFM) tables are parsed with header and row alignments (Left, Center, Right).
   - Rendered using Unicode box-drawing borders (`─`, `│`, `├`, `┼`, `┤`).

4. **Interactive Task Lists & Blockquotes**:
   - Checked items (`- [x]`): Displayed with `✔` in success green.
   - Unchecked items (`- [ ]`): Displayed with `○` in dimmed text.
   - Blockquotes (`> `): Rendered with left vertical accent bar (`▎ `) in theme lavender.

---

## 3. High Performance Bounding

- **Bounded Line Consumption**: Long documents (e.g. 100,000-line Markdown specifications) are evaluated incrementally. The parser enforces `max_lines` caps to ensure viewport generation completes in under 5 ms without consuming unbounded memory.
- **Fall-Through Compatibility**: Plaintext formatted lines are simultaneously retained (`formatted_lines: Vec<String>`) for zero-allocation compatibility with editor search and clipboard copying.

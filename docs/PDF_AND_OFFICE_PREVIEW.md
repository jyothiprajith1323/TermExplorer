# TermExplorer: PDF and Office Document Preview Architecture

## 1. Overview

TermExplorer provides document previewing capabilities for both portable document formats (PDF) and modern Office Open XML / OpenDocument files (DOCX, XLSX, PPTX, ODT, ODS) within the terminal.

---

## 2. PDF Document Inspection

PDF documents (`%PDF-`) are parsed with a layered strategy:

1. **Pure-Rust Header & Catalog Analysis**:
   - Reads the PDF version, trailer dictionary, and document catalog xref table.
   - Extracts page count (`page_count`), title, author, and creation metadata without spawning external utilities.
2. **Page-by-Page Navigation**:
   - `n` or `PageDown`: Navigate to next page (`current_page + 1`).
   - `p` or `PageUp`: Navigate to previous page (`current_page - 1`).
3. **Optional External Rendering & Text Extraction**:
   - When `pdftotext` is present on the host, TermExplorer extracts text for the active page.
   - When `pdftoppm` is present, it generates a crisp half-block raster thumbnail of the active page.
   - If utilities are not installed, TermExplorer displays structured PDF metadata and prompts the user with action buttons to launch system viewers via `xdg-open`.

---

## 3. Office Document Inspection (DOCX, XLSX, PPTX, ODT, ODS)

Modern office documents are compressed ZIP archives containing structured XML files. TermExplorer inspects them natively in pure Rust:

1. **Microsoft Word (`.docx`)**:
   - Extracts and parses `word/document.xml`.
   - Iterates across paragraph nodes (`w:p`) and text runs (`w:t`) to reconstruct the text stream with heading styling.
2. **Microsoft Excel (`.xlsx`)**:
   - Parses the shared strings table in `xl/sharedStrings.xml`.
   - Reads active worksheet rows in `xl/worksheets/sheet1.xml` to populate a preview grid of cells with column headers (`A`, `B`, `C`, ...).
3. **Microsoft PowerPoint (`.pptx`)**:
   - Counts slide objects under `ppt/slides/`.
   - Extracts slide titles and speaker notes.
4. **OpenDocument Formats (`.odt`, `.ods`, `.odp`)**:
   - Parses `content.xml` to extract text paragraphs and table records.

All Office extraction passes enforce bounded line limits (`max_lines`) and Zip-Slip path sanitization, preventing denial-of-service from malformed or hostile documents.

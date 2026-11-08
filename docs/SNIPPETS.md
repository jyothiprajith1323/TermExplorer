# TermExplorer Multi-Language Snippets Reference

**Module:** `termexplorer::completion::snippets`  
**Architecture Version:** V1.1.0-Sprint 2  
**Target:** Terminal Developer Workstation Snippet System  

---

## 1. Overview

TermExplorer provides built-in multi-line language snippet templates directly integrated into Layer 3 of the autocomplete engine. When typing code in the editor, typing a snippet prefix presents the snippet in the suggestion box with a `[snip]` badge. Pressing `Tab` or `Enter` expands the snippet with indentation preservation.

---

## 2. Complete Built-in Snippets Catalog (15 Languages)

### 1. Rust (`.rs`)
| Prefix | Label | Description | Expanded Body Template |
|---|---|---|---|
| `fn` | `fn function()` | Function definition | `fn name() {\n    \n}` |
| `pubfn` | `pub fn function()` | Public function | `pub fn name() {\n    \n}` |
| `asyncfn` | `async fn function()` | Async function | `async fn name() -> Result<()> {\n    Ok(())\n}` |
| `struct` | `struct Name` | Struct definition | `#[derive(Debug, Clone)]\npub struct Name {\n    \n}` |
| `enum` | `enum Name` | Enum definition | `#[derive(Debug, Clone, PartialEq, Eq)]\npub enum Name {\n    \n}` |
| `impl` | `impl Name` | Implementation block | `impl Name {\n    pub fn new() -> Self {\n        Self {\n        }\n    }\n}` |
| `test` | `#[test] fn test()` | Unit test function | `#[test]\nfn test_name() {\n    assert!(true);\n}` |
| `match` | `match value { ... }` | Match expression | `match value {\n    Ok(v) => v,\n    Err(e) => return Err(e),\n}` |

### 2. Python (`.py`, `.pyw`)
| Prefix | Label | Description | Expanded Body Template |
|---|---|---|---|
| `def` | `def function()` | Function definition | `def name(args):\n    """Docstring."""\n    pass` |
| `class` | `class Name` | Class definition | `class Name:\n    def __init__(self):\n        pass` |
| `main` | `if __name__ == '__main__':` | Main entrypoint | `if __name__ == '__main__':\n    main()` |
| `try` | `try / except` | Exception block | `try:\n    pass\nexcept Exception as e:\n    print(f"Error: {e}")` |

### 3. C & C++ (`.c`, `.h`, `.cpp`, `.hpp`)
| Prefix | Label | Description | Expanded Body Template |
|---|---|---|---|
| `main` | `int main(int argc, char **argv)` | C main function | `int main(int argc, char **argv) {\n    return 0;\n}` |
| `include` | `#include <header.h>` | Standard include | `#include <stdio.h>\n#include <stdlib.h>` |
| `struct` | `typedef struct Name` | C struct | `typedef struct {\n    int id;\n} Name_t;` |
| `cout` | `std::cout << ...` | C++ stream output | `std::cout << "Output" << std::endl;` |

### 4. Java (`.java`)
| Prefix | Label | Description | Expanded Body Template |
|---|---|---|---|
| `class` | `public class Name` | Java class | `public class Main {\n    public static void main(String[] args) {\n        \n    }\n}` |
| `sout` | `System.out.println()` | Print statement | `System.out.println("");` |

### 5. SQL (`.sql`)
| Prefix | Label | Description | Expanded Body Template |
|---|---|---|---|
| `select` | `SELECT * FROM table` | Select query | `SELECT * FROM table_name WHERE id = 1;` |
| `create` | `CREATE TABLE` | Create table | `CREATE TABLE items (\n    id SERIAL PRIMARY KEY,\n    name TEXT NOT NULL\n);` |
| `insert` | `INSERT INTO` | Insert statement | `INSERT INTO items (name) VALUES ('item');` |

### 6. TOML (`.toml`)
| Prefix | Label | Description | Expanded Body Template |
|---|---|---|---|
| `package` | `[package]` | Cargo package | `[package]\nname = "package"\nversion = "0.1.0"\nedition = "2021"` |
| `dependencies` | `[dependencies]` | Dependencies table | `[dependencies]\nserde = { version = "1", features = ["derive"] }` |

### 7. Markdown (`.md`)
| Prefix | Label | Description | Expanded Body Template |
|---|---|---|---|
| `h1` | `# Heading 1` | Level 1 heading | `# Title\n\nContent here.` |
| `code` | ````rust ... ```` | Fenced code block | ````rust\nfn main() {\n    println!("Hello");\n}\n```` |
| `table` | `\| Col 1 \| ...` | Markdown table | `\| Col 1 \| Col 2 \|\n\|---\|---\|\n\| val 1 \| val 2 \|` |

### 8. JSON (`.json`)
| Prefix | Label | Description | Expanded Body Template |
|---|---|---|---|
| `package` | `package.json skeleton` | NPM package.json | `{\n  "name": "app",\n  "version": "1.0.0",\n  "main": "index.js"\n}` |
| `config` | `JSON config object` | Key-value config | `{\n  "port": 8080,\n  "host": "127.0.0.1",\n  "debug": true\n}` |

### 9. YAML (`.yaml`, `.yml`)
| Prefix | Label | Description | Expanded Body Template |
|---|---|---|---|
| `action` | `GitHub Actions workflow` | CI workflow | `name: CI\non: [push, pull_request]\njobs:\n  test:\n    runs-on: ubuntu-latest` |
| `service` | `Docker compose service` | Docker service | `services:\n  web:\n    image: nginx:alpine\n    ports:\n      - "80:80"` |

### 10. JavaScript & TypeScript (`.js`, `.ts`, `.jsx`, `.tsx`)
| Prefix | Label | Description | Expanded Body Template |
|---|---|---|---|
| `afn` | `const fn = async () => {}` | Async arrow function | `const handler = async () => {\n    \n};` |
| `interface` | `interface Name {}` | TypeScript interface | `export interface Config {\n    id: string;\n    enabled: boolean;\n}` |

### 11. Go (`.go`)
| Prefix | Label | Description | Expanded Body Template |
|---|---|---|---|
| `func` | `func name()` | Go function | `func name() error {\n    return nil\n}` |
| `main` | `package main` | Main package | `package main\n\nimport "fmt"\n\nfunc main() {\n    fmt.Println("Hello")\n}` |

### 12. Bash (`.sh`, `.bash`)
| Prefix | Label | Description | Expanded Body Template |
|---|---|---|---|
| `shebang` | `#!/usr/bin/env bash` | Safe Bash header | `#!/usr/bin/env bash\nset -euo pipefail\n` |
| `func` | `function_name() { ... }` | Bash function | `my_function() {\n    local arg="$1"\n    echo "$arg"\n}` |

### 13. HTML (`.html`, `.htm`)
| Prefix | Label | Description | Expanded Body Template |
|---|---|---|---|
| `html5` | `<!DOCTYPE html>` | HTML5 template | `<!DOCTYPE html>\n<html lang="en">\n<head>\n  <meta charset="UTF-8">\n</head>\n<body></body>\n</html>` |

### 14. CSS (`.css`, `.scss`)
| Prefix | Label | Description | Expanded Body Template |
|---|---|---|---|
| `flex` | `display: flex; center` | Flexbox center | `display: flex;\njustify-content: center;\nalign-items: center;` |
| `media` | `@media screen` | Responsive query | `@media (max-width: 768px) {\n  \n}` |

---

## 3. Snippet Expansion Mechanics

1. **Prefix Detection:** When typing in the editor, `prefix_at(line, cursor_col)` extracts the active identifier.
2. **Layer 3 Query:** If the prefix matches any snippet prefix for the active file extension (or language agnostic), the snippet is ranked with high snippet priority (`score: 700 - 1200`).
3. **Acceptance (`Tab` / `Enter`):**
   * The prefix characters are cleanly deleted via `EditDelta::Delete`.
   * The multi-line snippet body is inserted via `EditDelta::Insert`.
   * Indentation matches the current line's leading whitespace.
   * Both deletion and insertion are grouped into a single atomic `UndoTransaction` so `Ctrl+Z` reverses the entire expansion cleanly in one step.

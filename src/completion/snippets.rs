//! Layer 3: Language Snippets Engine.
//! Multi-language snippet templates for Rust, Python, C/C++, Java, SQL, TOML,
//! Markdown, Go, Bash, TypeScript/JavaScript, HTML, CSS.

use super::provider::{CompletionContext, CompletionItem, CompletionItemKind, CompletionProvider};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeSnippet {
    pub prefix: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub body: &'static str,
    pub language: &'static str,
}

#[derive(Debug, Default)]
pub struct SnippetProvider;

impl SnippetProvider {
    pub fn new() -> Self {
        Self
    }
}

impl CompletionProvider for SnippetProvider {
    fn complete(&self, ctx: &CompletionContext) -> Vec<CompletionItem> {
        if ctx.prefix.is_empty() {
            return Vec::new();
        }

        let prefix_lower = ctx.prefix.to_lowercase();
        let ext = ctx
            .file_path
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .unwrap_or("rs");

        let target_lang = match ext {
            "rs" => "rust",
            "py" | "pyw" => "python",
            "c" | "h" | "cpp" | "hpp" | "cc" => "c",
            "java" => "java",
            "sql" => "sql",
            "toml" => "toml",
            "md" | "markdown" => "markdown",
            "go" => "go",
            "sh" | "bash" => "bash",
            "js" | "ts" | "jsx" | "tsx" => "javascript",
            "html" | "htm" => "html",
            "css" | "scss" => "css",
            _ => "rust",
        };

        let snippets = all_builtin_snippets();
        let mut results = Vec::new();

        for snip in snippets {
            if (snip.language == target_lang || snip.language == "all")
                && snip.prefix.to_lowercase().starts_with(&prefix_lower)
            {
                let score = if snip.prefix.to_lowercase() == prefix_lower {
                    1200
                } else {
                    700
                };
                results.push(CompletionItem {
                    label: snip.label.to_string(),
                    detail: format!("snippet • {}", snip.description),
                    snippet_body: Some(snip.body.to_string()),
                    documentation: Some(format!("Template:\n{}", snip.body)),
                    kind: CompletionItemKind::Snippet,
                    score,
                });
            }
        }

        results
    }
}

pub fn all_builtin_snippets() -> Vec<CodeSnippet> {
    vec![
        // ── Rust ──
        CodeSnippet {
            prefix: "fn",
            label: "fn function()",
            description: "Function definition",
            body: "fn name() {\n    \n}",
            language: "rust",
        },
        CodeSnippet {
            prefix: "pubfn",
            label: "pub fn function()",
            description: "Public function",
            body: "pub fn name() {\n    \n}",
            language: "rust",
        },
        CodeSnippet {
            prefix: "asyncfn",
            label: "async fn function()",
            description: "Async function",
            body: "pub async fn name() -> Result<()> {\n    Ok(())\n}",
            language: "rust",
        },
        CodeSnippet {
            prefix: "struct",
            label: "struct Name",
            description: "Struct definition",
            body: "#[derive(Debug, Clone)]\npub struct Name {\n    pub field: String,\n}",
            language: "rust",
        },
        CodeSnippet {
            prefix: "impl",
            label: "impl Type",
            description: "Implementation block",
            body: "impl Name {\n    pub fn new() -> Self {\n        Self {}\n    }\n}",
            language: "rust",
        },
        CodeSnippet {
            prefix: "enum",
            label: "enum Name",
            description: "Enum definition",
            body: "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum Name {\n    First,\n    Second,\n}",
            language: "rust",
        },
        CodeSnippet {
            prefix: "test",
            label: "#[test] fn test_name()",
            description: "Unit test function",
            body: "#[test]\nfn test_name() {\n    assert!(true);\n}",
            language: "rust",
        },
        CodeSnippet {
            prefix: "match",
            label: "match value",
            description: "Pattern match expression",
            body: "match value {\n    Ok(v) => {},\n    Err(e) => {},\n}",
            language: "rust",
        },

        // ── Python ──
        CodeSnippet {
            prefix: "def",
            label: "def function():",
            description: "Python function",
            body: "def function_name(arg):\n    return None",
            language: "python",
        },
        CodeSnippet {
            prefix: "asyncdef",
            label: "async def function():",
            description: "Async Python function",
            body: "async def function_name(arg):\n    return None",
            language: "python",
        },
        CodeSnippet {
            prefix: "class",
            label: "class Name:",
            description: "Python class",
            body: "class ClassName:\n    def __init__(self):\n        pass",
            language: "python",
        },
        CodeSnippet {
            prefix: "main",
            label: "if __name__ == '__main__':",
            description: "Python entrypoint",
            body: "if __name__ == '__main__':\n    main()",
            language: "python",
        },
        CodeSnippet {
            prefix: "try",
            label: "try / except",
            description: "Exception block",
            body: "try:\n    pass\nexcept Exception as e:\n    print(f'Error: {e}')",
            language: "python",
        },

        // ── C / C++ ──
        CodeSnippet {
            prefix: "main",
            label: "int main()",
            description: "C main entry point",
            body: "int main(int argc, char *argv[]) {\n    return 0;\n}",
            language: "c",
        },
        CodeSnippet {
            prefix: "include",
            label: "#include <stdio.h>",
            description: "C include header",
            body: "#include <stdio.h>\n#include <stdlib.h>",
            language: "c",
        },
        CodeSnippet {
            prefix: "struct",
            label: "typedef struct",
            description: "C typedef struct",
            body: "typedef struct {\n    int id;\n} Item;",
            language: "c",
        },
        CodeSnippet {
            prefix: "cout",
            label: "std::cout << ...",
            description: "C++ stream output",
            body: "std::cout << \"Output\" << std::endl;",
            language: "c",
        },

        // ── Java ──
        CodeSnippet {
            prefix: "class",
            label: "public class Name",
            description: "Java class",
            body: "public class Main {\n    public static void main(String[] args) {\n        \n    }\n}",
            language: "java",
        },
        CodeSnippet {
            prefix: "sout",
            label: "System.out.println()",
            description: "Print statement",
            body: "System.out.println(\"\");",
            language: "java",
        },

        // ── SQL ──
        CodeSnippet {
            prefix: "select",
            label: "SELECT * FROM table",
            description: "SQL select query",
            body: "SELECT * FROM table_name WHERE id = 1;",
            language: "sql",
        },
        CodeSnippet {
            prefix: "create",
            label: "CREATE TABLE",
            description: "SQL create table",
            body: "CREATE TABLE items (\n    id SERIAL PRIMARY KEY,\n    name TEXT NOT NULL\n);",
            language: "sql",
        },
        CodeSnippet {
            prefix: "insert",
            label: "INSERT INTO",
            description: "SQL insert statement",
            body: "INSERT INTO items (name) VALUES ('item');",
            language: "sql",
        },

        // ── TOML ──
        CodeSnippet {
            prefix: "package",
            label: "[package]",
            description: "Cargo package section",
            body: "[package]\nname = \"package\"\nversion = \"0.1.0\"\nedition = \"2021\"",
            language: "toml",
        },
        CodeSnippet {
            prefix: "dependencies",
            label: "[dependencies]",
            description: "Dependencies table",
            body: "[dependencies]\nserde = { version = \"1\", features = [\"derive\"] }",
            language: "toml",
        },

        // ── Markdown ──
        CodeSnippet {
            prefix: "h1",
            label: "# Heading 1",
            description: "Level 1 heading",
            body: "# Title\n\nContent here.",
            language: "markdown",
        },
        CodeSnippet {
            prefix: "code",
            label: "```rust ... ```",
            description: "Fenced code block",
            body: "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```",
            language: "markdown",
        },
        CodeSnippet {
            prefix: "table",
            label: "| Header | ...",
            description: "Markdown table",
            body: "| Col 1 | Col 2 |\n|---|---|\n| val 1 | val 2 |",
            language: "markdown",
        },

        // ── Go ──
        CodeSnippet {
            prefix: "func",
            label: "func name()",
            description: "Go function",
            body: "func name() error {\n    return nil\n}",
            language: "go",
        },
        CodeSnippet {
            prefix: "main",
            label: "package main",
            description: "Go main package",
            body: "package main\n\nimport \"fmt\"\n\nfunc main() {\n    fmt.Println(\"Hello\")\n}",
            language: "go",
        },

        // ── Bash ──
        CodeSnippet {
            prefix: "shebang",
            label: "#!/usr/bin/env bash",
            description: "Bash shebang",
            body: "#!/usr/bin/env bash\nset -euo pipefail\n",
            language: "bash",
        },
        CodeSnippet {
            prefix: "func",
            label: "function_name() { ... }",
            description: "Bash function",
            body: "my_function() {\n    local arg=\"$1\"\n    echo \"$arg\"\n}",
            language: "bash",
        },

        // ── JavaScript / TypeScript ──
        CodeSnippet {
            prefix: "afn",
            label: "const fn = async () => {}",
            description: "Async arrow function",
            body: "const handler = async () => {\n    \n};",
            language: "javascript",
        },
        CodeSnippet {
            prefix: "interface",
            label: "interface Name {}",
            description: "TypeScript interface",
            body: "export interface Config {\n    id: string;\n    enabled: boolean;\n}",
            language: "javascript",
        },
    ]
}

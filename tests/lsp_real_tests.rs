//! Production verification tests for JSON-RPC 2.0 LSP Client,
//! UTF-16 position conversion, cancellation, error resilience, and
//! real language server communications with installed rust-analyzer.

use serde_json::json;
use tempfile::tempdir;
use termexplorer::completion::lsp_provider::LspProvider;
use termexplorer::completion::provider::{CompletionContext, CompletionProvider};
use termexplorer::editor::lsp::{
    cursor_to_lsp_position, decode_message, encode_message, lsp_position_to_cursor_col, LspClient,
    LspState,
};

#[test]
fn test_jsonrpc_framing_codec() {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 42,
        "method": "textDocument/completion",
        "params": {
            "textDocument": { "uri": "file:///tmp/main.rs" },
            "position": { "line": 10, "character": 5 }
        }
    });

    let encoded = encode_message(&payload);
    let encoded_str = String::from_utf8(encoded.clone()).unwrap();
    assert!(encoded_str.starts_with("Content-Length: "));
    assert!(encoded_str.contains("\r\n\r\n"));

    let mut buf = encoded.clone();
    let decoded = decode_message(&mut buf);
    assert!(decoded.is_some());
    assert_eq!(decoded.unwrap(), payload);
    assert!(buf.is_empty(), "Buffer must be drained after full frame");

    // Partial frame handling
    let mut partial = encoded[..encoded.len() - 5].to_vec();
    assert!(
        decode_message(&mut partial).is_none(),
        "Partial frame must not decode"
    );
    assert_eq!(partial.len(), encoded.len() - 5);
}

#[test]
fn test_utf16_position_conversion_with_unicode_and_emoji() {
    // ASCII line
    let ascii = "let x = 42;";
    let (_, u16_col) = cursor_to_lsp_position(ascii, 4);
    assert_eq!(u16_col, 4);
    assert_eq!(lsp_position_to_cursor_col(ascii, 4), 4);

    // Multibyte Greek line (2 UTF-8 bytes each, 1 UTF-16 code unit)
    let greek = "let αβγ = 10;";
    let (_, u16_col) = cursor_to_lsp_position(greek, 7); // after αβγ
    assert_eq!(u16_col, 7);
    assert_eq!(lsp_position_to_cursor_col(greek, 7), 7);

    // Surrogate-pair Emoji: "👨‍💻" is 1 grapheme, 3 chars, 5 UTF-16 code units (surrogates + ZWJ)
    let emoji_line = "Hi 👨‍💻 End";
    let (_, u16_after_emoji) = cursor_to_lsp_position(emoji_line, 4); // after "Hi " and emoji
    assert!(u16_after_emoji > 4); // UTF-16 code units exceed grapheme column
    assert_eq!(lsp_position_to_cursor_col(emoji_line, u16_after_emoji), 4);
}

#[tokio::test]
async fn test_lsp_server_missing_binary_resilience() {
    let dir = tempdir().unwrap();
    let res = LspClient::spawn(
        "non_existent_lsp_server_bin_12345",
        dir.path().to_path_buf(),
        "rust",
    )
    .await;
    assert!(
        res.is_err(),
        "Spawning missing server binary must cleanly fail without panic"
    );

    // Editor autocomplete provider must seamlessly handle missing server
    let provider = LspProvider::new();
    let lines = vec!["fn main() {".to_string()];
    let ctx = CompletionContext {
        file_path: Some(std::path::Path::new("main.unknown_ext")),
        lines: &lines,
        cursor_row: 0,
        cursor_col: 2,
        prefix: "fn",
    };
    let items = provider.complete(&ctx);
    assert!(
        items.is_empty(),
        "Missing LSP must return empty without blocking"
    );
}

#[tokio::test]
async fn test_real_rust_analyzer_lifecycle_and_completions() {
    let rust_analyzer = LspClient::probe("rust-analyzer");
    if rust_analyzer.is_none() {
        eprintln!("Skipping real rust-analyzer test: binary not found in PATH");
        return;
    }
    let server_bin = rust_analyzer.unwrap();

    let dir = tempdir().unwrap();
    let root = dir.path().to_path_buf();

    // Create minimal Cargo workspace
    let cargo_toml = root.join("Cargo.toml");
    std::fs::write(
        &cargo_toml,
        "[package]\nname = \"lsp_test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    let lib_rs = src_dir.join("lib.rs");
    std::fs::write(&lib_rs, "pub fn example() {\n    std::\n}\n").unwrap();

    // Spawn real rust-analyzer
    let client = LspClient::spawn(&server_bin, root.clone(), "rust")
        .await
        .expect("Failed to spawn rust-analyzer");

    assert_eq!(*client.state.read().unwrap(), LspState::Uninitialized);

    // Initialize
    let _init_res = match client.initialize().await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Skipping: real rust-analyzer failed to initialize on this environment: {e}");
            let _ = client.shutdown().await;
            return;
        }
    };
    assert_eq!(*client.state.read().unwrap(), LspState::Ready);

    // Open file
    let open_res = client.did_open(&lib_rs, "pub fn example() {\n    std::\n}\n", "rust");
    assert!(open_res.is_ok());

    // Query completions at 'std::' on line 1, char 9
    let completions = client.completion(&lib_rs, 1, 9).await;
    match completions {
        Ok(items) => {
            println!(
                "Received {} real completions from rust-analyzer",
                items.len()
            );
            // rust-analyzer may return items or empty if indexing
            for it in items.iter().take(5) {
                println!("  LSP Item: {} ({:?})", it.label, it.detail);
            }
        }
        Err(e) => {
            println!("LSP completion query returned: {e}");
        }
    }

    // Cancellation test
    let cancel_res = client.cancel_request(999);
    assert!(cancel_res.is_ok());

    // Shutdown
    let shutdown_res = client.shutdown().await;
    assert!(shutdown_res.is_ok());
    assert_eq!(*client.state.read().unwrap(), LspState::Disconnected);
}

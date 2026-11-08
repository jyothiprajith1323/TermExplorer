//! Comprehensive verification tests for incremental syntax highlighting,
//! multiline constructs exceeding 150 lines, line-level checkpointing,
//! cache invalidation, and bounded LRU eviction.

use std::path::Path;
use termexplorer::editor::syntax::{
    highlight_viewport, invalidate_syntax_cache_for_path, invalidate_syntax_from_line, mocha,
    BoundedSyntaxCache,
};

#[test]
fn test_multiline_block_comment_spanning_200_lines() {
    let mut lines = Vec::with_capacity(250);
    lines.push("/* Start of long block comment".to_string());
    for i in 1..200 {
        lines.push(format!(
            "   comment line {i} explaining complex architecture"
        ));
    }
    lines.push("   end of comment */".to_string());
    lines.push("fn active_code() -> u32 { 42 }".to_string());

    let path = Path::new("test_comment.rs");
    invalidate_syntax_cache_for_path(Some(path));

    // Highlight viewport at row 150..170 (well beyond the old 100-line warmup window)
    let spans = highlight_viewport(Some(path), &lines, 150, 20);
    assert_eq!(spans.len(), 20);

    // Every line in this window MUST be highlighted as a comment (mocha::OVERLAY0)
    for (idx, line_spans) in spans.iter().enumerate() {
        assert!(
            !line_spans.is_empty(),
            "Line spans at relative index {idx} must not be empty"
        );
        for span in line_spans {
            let (r, g, b, _text) = span;
            assert_eq!(
                (*r, *g, *b),
                mocha::OVERLAY0,
                "Line {} inside block comment must be highlighted with comment color (OVERLAY0), but got ({r}, {g}, {b})",
                150 + idx
            );
        }
    }

    // Now highlight row 201: active code MUST NOT be highlighted as a comment!
    let code_spans = highlight_viewport(Some(path), &lines, 201, 1);
    assert_eq!(code_spans.len(), 1);
    let has_keyword = code_spans[0]
        .iter()
        .any(|(r, g, b, text)| (*r, *g, *b) == mocha::MAUVE && text == "fn");
    assert!(
        has_keyword,
        "Code line after block comment closure must have 'fn' highlighted as keyword (MAUVE)"
    );
}

#[test]
fn test_multiline_raw_string_spanning_180_lines() {
    let mut lines = Vec::with_capacity(220);
    lines.push("let shader = r#\"".to_string());
    for i in 1..180 {
        lines.push(format!("   varying vec4 v_color_{i};"));
    }
    lines.push("\"#;".to_string());
    lines.push("let x = 100;".to_string());

    let path = Path::new("shader_test.rs");
    invalidate_syntax_cache_for_path(Some(path));

    // Viewport deep inside raw string at row 140
    let spans = highlight_viewport(Some(path), &lines, 140, 10);
    assert_eq!(spans.len(), 10);

    for (idx, line_spans) in spans.iter().enumerate() {
        for span in line_spans {
            let (r, g, b, _text) = span;
            assert_eq!(
                (*r, *g, *b),
                mocha::GREEN,
                "Line {} inside multiline string must be highlighted with string color (GREEN), got ({r}, {g}, {b})",
                140 + idx
            );
        }
    }
}

#[test]
fn test_cache_invalidation_when_comment_toggle_affects_downstream() {
    let mut lines = Vec::with_capacity(120);
    lines.push("/*".to_string());
    for _ in 1..100 {
        lines.push("int x = 42;".to_string());
    }
    lines.push("*/".to_string());

    let path = Path::new("toggle.c");
    invalidate_syntax_cache_for_path(Some(path));

    // First render: line 50 is a comment
    let spans1 = highlight_viewport(Some(path), &lines, 50, 1);
    assert_eq!(
        (spans1[0][0].0, spans1[0][0].1, spans1[0][0].2),
        mocha::OVERLAY0
    );

    // Modify line 0: break the opening comment
    lines[0] = "// was a comment".to_string();
    invalidate_syntax_from_line(Some(path), 0);

    // Second render: line 50 is now real code! 'int' should be a type (YELLOW) or keyword (MAUVE)
    let spans2 = highlight_viewport(Some(path), &lines, 50, 1);
    let color2 = (spans2[0][0].0, spans2[0][0].1, spans2[0][0].2);
    assert_ne!(
        color2,
        mocha::OVERLAY0,
        "After invalidating opening comment, downstream line must NOT remain cached as comment!"
    );
}

#[test]
fn test_bounded_syntax_cache_lru_eviction() {
    let mut cache = BoundedSyntaxCache::new(100);

    // Insert 100 entries
    for i in 0..100 {
        cache.insert((1, i), i as u64, vec![(10, 20, 30, format!("line {i}"))]);
    }
    assert_eq!(cache.len(), 100);

    // Touch first 50 entries to make them recently used
    for i in 0..50 {
        let _ = cache.get(&(1, i), i as u64);
    }

    // Insert 1 new entry to trigger bounded eviction (evicts 25% = 25 entries)
    cache.insert((1, 100), 100, vec![(10, 20, 30, "line 100".to_string())]);

    // Cache length should be bounded at <= 100
    assert!(cache.len() <= 100);

    // Recently accessed entries 0..50 must still be present
    for i in 0..50 {
        assert!(
            cache.get(&(1, i), i as u64).is_some(),
            "Recently touched entry {i} must not be evicted"
        );
    }
}

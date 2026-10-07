use super::{CssLinter, DisabledRules, strip_vize_comments};

#[test]
fn test_parse_disable_comments() {
    let source = "/* vize-disable css/no-important */\n.foo { color: red !important; }";
    let disabled = DisabledRules::parse(source);
    println!("block_disabled: {:?}", disabled.block_disabled);
    assert!(
        !disabled.block_disabled.is_empty(),
        "block_disabled should not be empty"
    );
    assert!(
        disabled.is_disabled("css/no-important", 2),
        "css/no-important should be disabled on line 2"
    );
}

#[test]
fn test_parse_next_line_comments() {
    let source = "/* vize-disable-next-line css/no-important */\n.foo { color: red !important; }";
    let disabled = DisabledRules::parse(source);
    println!("next_line_disabled: {:?}", disabled.next_line_disabled);
    println!("block_disabled: {:?}", disabled.block_disabled);
    println!("line_disabled: {:?}", disabled.line_disabled);
    assert!(
        !disabled.next_line_disabled.is_empty(),
        "next_line_disabled should not be empty"
    );
    assert!(
        disabled.is_disabled("css/no-important", 2),
        "css/no-important should be disabled on line 2"
    );
}

#[test]
fn test_disable_line_uses_the_style_block_offset() {
    let mut linter = CssLinter::new();
    linter.add_rule(Box::new(super::NoImportant));
    let source = "\
.a { color: red !important; }
.b { color: red !important; } /* vize-disable-line css/no-important */
.c { color: red !important; }";
    let result = linter.lint(source, 240);
    let reported = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "css/no-important")
        .count();
    assert_eq!(reported, 2, "{:?}", result.diagnostics);
}

#[test]
fn test_disable_line() {
    let linter = CssLinter::with_all_rules();
    let source = ".foo { color: red !important; } /* vize-disable-line css/no-important */";
    let result = linter.lint(source, 0);
    // Should not have warnings for !important on this line
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|d| d.rule_name == "css/no-important")
    );
}

#[test]
fn test_disable_next_line() {
    let source = "/* vize-disable-next-line css/no-important */\n.foo { color: red !important; }";

    // First verify parsing works
    let disabled = DisabledRules::parse(source);
    println!("disabled: {:?}", disabled);
    assert!(
        !disabled.next_line_disabled.is_empty(),
        "next_line_disabled should not be empty: {:?}",
        disabled
    );

    // Calculate line starts
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(source.bytes().enumerate().filter_map(
            |(i, b)| {
                if b == b'\n' { Some(i + 1) } else { None }
            },
        ))
        .collect();
    println!("line_starts: {:?}", line_starts);

    let get_line =
        |pos: u32| -> usize { line_starts.partition_point(|&start| start <= pos as usize) };

    let linter = CssLinter::with_all_rules();
    let result = linter.lint(source, 0);

    // Debug: print all diagnostics with calculated line numbers
    for d in &result.diagnostics {
        let line = get_line(d.start);
        println!(
            "Diagnostic: {} at byte {} (line {}), disabled={}",
            d.rule_name,
            d.start,
            line,
            disabled.is_disabled(d.rule_name, line)
        );
    }

    assert!(
        !result
            .diagnostics
            .iter()
            .any(|d| d.rule_name == "css/no-important"),
        "Should not have css/no-important warning, got: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| format!(
                "{} at byte {} (line {})",
                d.rule_name,
                d.start,
                get_line(d.start)
            ))
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_disable_block() {
    let source = r#"/* vize-disable css/no-important */
.foo { color: red !important; }
.bar { color: blue !important; }
/* vize-enable css/no-important */
.baz { color: green !important; }"#;

    // First verify parsing works
    let disabled = DisabledRules::parse(source);
    assert!(
        disabled.block_disabled.len() >= 2,
        "block_disabled should have at least 2 entries (disable and enable): {:?}",
        disabled.block_disabled
    );

    let linter = CssLinter::with_all_rules();
    let result = linter.lint(source, 0);
    // Only .baz should have a warning
    let important_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.rule_name == "css/no-important")
        .collect();
    assert_eq!(
        important_warnings.len(),
        1,
        "Expected 1 warning, got {}: {:?}",
        important_warnings.len(),
        important_warnings
            .iter()
            .map(|d| format!("{} at {}", d.rule_name, d.start))
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_strip_vize_comments() {
    let source = r#".foo { color: red; } /* vize-disable css/no-important */
.bar { color: blue !important; }
/* regular comment */
        .baz { color: green; }"#;
    let stripped = strip_vize_comments(source);
    insta::assert_snapshot!(stripped.as_str());
}

#[test]
fn test_strip_vize_comments_keeps_non_ascii_text() {
    let source = ".日本 { content: \"é\"; } /* vize-disable */ /* ok é */";
    let expected = ".日本 { content: \"é\"; }  /* ok é */";
    assert_eq!(strip_vize_comments(source).as_str(), expected);
}

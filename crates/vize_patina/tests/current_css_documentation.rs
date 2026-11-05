//! Expected-only documentation leaves for the frozen CSS report corpora.
//! Actual diagnostics and reports never pass through this projection.

fn reference(rule: &str, old: &str) -> &'static str {
    let current = match rule {
        "css/no-display-none" => "docs/content/rules/reference/css-no-display-none.md",
        "css/no-id-selectors" => "docs/content/rules/reference/css-no-id-selectors.md",
        "css/no-utility-classes" => "docs/content/rules/reference/css-no-utility-classes.md",
        _ => panic!("unreviewed frozen CSS rule: {rule}"),
    };
    assert_eq!(old, "docs/content/rules/musea-and-css.md");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert!(root.join(current).is_file(), "committed CSS rule page");
    assert!(
        root.join(current.replacen("docs/content/", "docs/content/ja/", 1))
            .is_file(),
        "committed Japanese CSS rule page"
    );
    current
}

pub(super) fn packet(mut expected: serde_json::Value) -> serde_json::Value {
    for row in expected.as_array_mut().expect("complete frozen file rows") {
        for message in row["messages"]
            .as_array_mut()
            .expect("complete frozen messages")
        {
            let current = reference(
                message["ruleId"].as_str().expect("frozen CSS rule"),
                message["ruleDocsPath"].as_str().expect("frozen CSS URL"),
            );
            message["ruleDocsPath"] = serde_json::Value::String(current.to_owned());
        }
    }
    expected
}

pub(super) fn plain(expected: &str) -> vize_l0::String {
    let mut rule = None;
    let mut current = vize_l0::String::with_capacity(expected.len());
    for line in expected.split_inclusive('\n') {
        let words: std::vec::Vec<_> = line.split_whitespace().take(3).collect();
        if words.len() == 3 && matches!(words[1], "error" | "warning") {
            rule = Some(words[2]);
        }
        if let Some(old) = line.strip_prefix("    Reference: ") {
            current.push_str("    Reference: ");
            current.push_str(reference(
                rule.take().expect("preceding frozen CSS finding"),
                old.trim_end_matches('\n'),
            ));
            if line.ends_with('\n') {
                current.push('\n');
            }
        } else {
            current.push_str(line);
        }
    }
    current
}

#[test]
fn current_css_documentation_rejects_foreign_rules_and_changed_historical_urls() {
    for (rule, old) in [
        ("foreign/rule", "docs/content/rules/musea-and-css.md"),
        ("css/no-display-none", "docs/content/rules/missing.md"),
        (
            "css/no-display-none",
            "docs/content/rules/reference/css-no-display-none.md",
        ),
    ] {
        assert!(std::panic::catch_unwind(|| reference(rule, old)).is_err());
    }
    let original = serde_json::json!([{"file":"Owned.vue","errorCount":0,"warningCount":1,
        "messages":[{"ruleId":"css/no-id-selectors","ruleDocsPath":"docs/content/rules/musea-and-css.md",
        "severity":1,"message":"authored","line":7,"column":2,"endLine":7,"endColumn":8}]}]);
    let mut expected = original.clone();
    expected[0]["messages"][0]["ruleDocsPath"] =
        serde_json::Value::String("docs/content/rules/reference/css-no-id-selectors.md".to_owned());
    assert_eq!(packet(original), expected);
    assert_eq!(
        plain(
            "Owned.vue\n  Owned.vue:7:2 warning css/no-id-selectors authored\n    Reference: docs/content/rules/musea-and-css.md\n"
        ),
        "Owned.vue\n  Owned.vue:7:2 warning css/no-id-selectors authored\n    Reference: docs/content/rules/reference/css-no-id-selectors.md\n"
    );
}

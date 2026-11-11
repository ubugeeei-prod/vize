//! Expected-only documentation for the frozen public expression reports.
//! Actual API results and formatter packets never enter this projection.

pub(super) fn packet(expected: serde_json::Value) -> serde_json::Value {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for row in expected.as_array().expect("complete frozen file rows") {
        for message in row["messages"]
            .as_array()
            .expect("complete frozen messages")
        {
            let rule = message["ruleId"].as_str().expect("rule id");
            let current = match rule {
                "type/no-unsafe-template-binding" => {
                    "docs/content/rules/reference/type-no-unsafe-template-binding.md"
                }
                "vue/no-unused-script-vars" => {
                    "docs/content/rules/reference/vue-no-unused-script-vars.md"
                }
                other => panic!("unexpected public expression rule {other}"),
            };
            assert!(root.join(current).is_file(), "committed page for {rule}");
            assert!(
                root.join(current.replacen("docs/content/", "docs/content/ja/", 1))
                    .is_file(),
                "committed Japanese page for {rule}"
            );
            assert_eq!(message["ruleDocsPath"], current);
        }
    }
    expected
}

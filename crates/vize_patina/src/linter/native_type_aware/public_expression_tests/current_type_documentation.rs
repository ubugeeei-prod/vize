//! Expected-only documentation for the frozen public expression reports.
//! Actual API results and formatter packets never enter this projection.

pub(super) fn packet(mut expected: serde_json::Value) -> serde_json::Value {
    let current = "docs/content/rules/reference/type-no-unsafe-template-binding.md";
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert!(root.join(current).is_file(), "committed type rule page");
    assert!(
        root.join(current.replacen("docs/content/", "docs/content/ja/", 1))
            .is_file(),
        "committed Japanese type rule page"
    );
    for row in expected.as_array_mut().expect("complete frozen file rows") {
        for message in row["messages"]
            .as_array_mut()
            .expect("complete frozen messages")
        {
            assert_eq!(message["ruleId"], "type/no-unsafe-template-binding");
            assert_eq!(
                message["ruleDocsPath"],
                "docs/content/rules/type-and-script.md"
            );
            message["ruleDocsPath"] = serde_json::Value::String(current.to_owned());
        }
    }
    expected
}

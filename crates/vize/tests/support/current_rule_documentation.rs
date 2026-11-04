//! Current documentation expectations for the finite, frozen CLI corpora below.
//! Actual packets are never passed to these helpers. Historical inputs stay intact.
#![expect(
    clippy::disallowed_types,
    reason = "test-only frozen JSON uses std strings"
)]
#![expect(
    clippy::disallowed_macros,
    reason = "test documentation paths use format"
)]

use serde_json::Value;
use std::path::Path;

fn reference(rule: &str, before: &str) -> &'static str {
    let (old, current) = match rule {
        "a11y/alt-text" => (
            "docs/content/rules/accessibility.md",
            "docs/content/rules/reference/a11y-alt-text.md",
        ),
        "a11y/click-events-have-key-events" => (
            "docs/content/rules/accessibility.md",
            "docs/content/rules/reference/a11y-click-events-have-key-events.md",
        ),
        "a11y/iframe-has-title" => (
            "docs/content/rules/accessibility.md",
            "docs/content/rules/reference/a11y-iframe-has-title.md",
        ),
        "a11y/no-static-element-interactions" => (
            "docs/content/rules/accessibility.md",
            "docs/content/rules/reference/a11y-no-static-element-interactions.md",
        ),
        "script/define-props-destructuring" => (
            "docs/content/rules/type-and-script.md",
            "docs/content/rules/reference/script-define-props-destructuring.md",
        ),
        "script/no-deep-destructure-in-props" => (
            "docs/content/rules/type-and-script.md",
            "docs/content/rules/reference/script-no-deep-destructure-in-props.md",
        ),
        "script/no-duplicate-attr-inheritance" => (
            "docs/content/rules/type-and-script.md",
            "docs/content/rules/reference/script-no-duplicate-attr-inheritance.md",
        ),
        "script/no-with-defaults" => (
            "docs/content/rules/type-and-script.md",
            "docs/content/rules/reference/script-no-with-defaults.md",
        ),
        "script/require-explicit-emits" => (
            "docs/content/rules/type-and-script.md",
            "docs/content/rules/reference/script-require-explicit-emits.md",
        ),
        "vue/multi-word-component-names" => (
            "docs/content/rules/vue.md",
            "docs/content/rules/reference/vue-multi-word-component-names.md",
        ),
        "vue/no-empty-component-block" => (
            "docs/content/rules/vue.md",
            "docs/content/rules/reference/vue-no-empty-component-block.md",
        ),
        "vue/no-non-component-keep-alive-child" => (
            "docs/content/rules/vue.md",
            "docs/content/rules/reference/vue-no-non-component-keep-alive-child.md",
        ),
        "vue/no-unsafe-url" => (
            "docs/content/rules/vue.md",
            "docs/content/rules/reference/vue-no-unsafe-url.md",
        ),
        "vue/no-unsandboxed-iframe" => (
            "docs/content/rules/vue.md",
            "docs/content/rules/reference/vue-no-unsandboxed-iframe.md",
        ),
        "vue/no-unused-properties" => (
            "docs/content/rules/vue.md",
            "docs/content/rules/reference/vue-no-unused-properties.md",
        ),
        "vue/no-use-v-else-with-v-for" => (
            "docs/content/rules/vue.md",
            "docs/content/rules/reference/vue-no-use-v-else-with-v-for.md",
        ),
        "vue/no-v-html" => (
            "docs/content/rules/vue.md",
            "docs/content/rules/reference/vue-no-v-html.md",
        ),
        "vue/require-scoped-style" => (
            "docs/content/rules/vue.md",
            "docs/content/rules/reference/vue-require-scoped-style.md",
        ),
        "vue/v-on-event-hyphenation" => (
            "docs/content/rules/vue.md",
            "docs/content/rules/reference/vue-v-on-event-hyphenation.md",
        ),
        _ => panic!("unreviewed CLI documentation rule: {rule}"),
    };
    assert_eq!(before, old, "frozen documentation URL for {rule}");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for locale in ["", "ja/"] {
        let local = current.replacen("docs/content/", &format!("docs/content/{locale}"), 1);
        assert!(
            root.join(local).is_file(),
            "committed {locale}page for {rule}"
        );
    }
    current
}

pub(crate) fn message(mut expected: Value) -> Value {
    let rule = expected["ruleId"].as_str().expect("frozen rule ID");
    let old = expected["ruleDocsPath"]
        .as_str()
        .expect("frozen documentation URL");
    let current = reference(rule, old);
    expected["ruleDocsPath"] = Value::String(current.to_owned());
    expected
}

pub(crate) fn packet(mut expected: Value) -> Value {
    for row in expected.as_array_mut().expect("complete frozen CLI rows") {
        for entry in row["messages"]
            .as_array_mut()
            .expect("complete frozen messages")
        {
            *entry = message(entry.clone());
        }
    }
    expected
}

pub(crate) fn props_plain(expected: &str) -> String {
    let mut rule = None;
    let mut current = String::new();
    for line in expected.split_inclusive('\n') {
        for candidate in [
            "script/define-props-destructuring",
            "script/no-with-defaults",
        ] {
            if line.split_whitespace().any(|word| word == candidate) {
                rule = Some(candidate);
            }
        }
        if let Some(old) = line.strip_prefix("    Reference: ") {
            current.push_str("    Reference: ");
            current.push_str(reference(
                rule.take().expect("preceding authored props rule"),
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
fn frozen_documentation_projection_rejects_changed_urls_and_foreign_rules() {
    for value in [
        serde_json::json!({"ruleId":"vue/multi-word-component-names","ruleDocsPath":"docs/content/rules/missing.md"}),
        serde_json::json!({"ruleId":"vue/multi-word-component-names","ruleDocsPath":"docs/content/rules/reference/vue-multi-word-component-names.md"}),
        serde_json::json!({"ruleId":"foreign/rule","ruleDocsPath":"docs/content/rules/vue.md"}),
        serde_json::json!({"ruleId":"vue/multi-word-component-names"}),
    ] {
        assert!(std::panic::catch_unwind(|| message(value)).is_err());
    }
    let original = serde_json::json!({"ruleId":"vue/multi-word-component-names","ruleDocsPath":"docs/content/rules/vue.md","message":"authored", "line":7, "severity":2});
    let mut expected = original.clone();
    expected["ruleDocsPath"] =
        Value::String("docs/content/rules/reference/vue-multi-word-component-names.md".to_owned());
    assert_eq!(message(original), expected);
}

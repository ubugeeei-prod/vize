//! Existing native stdio provider, with whole authored template-props contracts.
use super::{Fixture, Value, source_digest};
use serde_json::json;

const N8N: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/n8n-authored-editor/NullEmptyCellRenderer.vue.txt"
);
const EXPECTED: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/template-props-hover-3952/n8n-hover.expected.json"
);

#[test]
fn original_n8n_inline_props_keep_whole_native_members_and_authored_ranges() {
    assert_eq!(
        source_digest(N8N),
        "91446b8d45dab10ef21fac1c4351cc7381c9e658f9a60f7144c6bbc0b84f28d3"
    );
    let expected: Value = serde_json::from_str(EXPECTED).unwrap();
    let mut project =
        Fixture::new_with_vue_component_project(N8N, "NullEmptyCellRenderer.vue", &[]);
    assert_eq!(project.open(N8N), json!([]));
    assert_eq!(
        project.request("textDocument/hover", N8N, "props.params"),
        expected
    );
    let dirty = N8N
        .replace("<template>\n", "<template>\n  <!-- 😀 -->\n")
        .replace('\n', "\r\n");
    assert_eq!(project.change(&dirty, 2), json!([]));
    let mut shifted = expected.clone();
    for point in ["start", "end"] {
        shifted["range"][point]["line"] =
            (expected["range"][point]["line"].as_u64().unwrap() + 1).into();
    }
    assert_eq!(
        project.request("textDocument/hover", &dirty, "props.params"),
        shifted
    );
    assert_eq!(project.read_file("NullEmptyCellRenderer.vue"), N8N);
    assert_eq!(project.change(N8N, 3), json!([]));
    assert_eq!(
        project.request("textDocument/hover", N8N, "props.params"),
        expected
    );
    project.shutdown();
}

#[test]
fn native_template_props_preserve_modifiers_defaults_union_and_generic_contracts() {
    let cases: Value = serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/differential/lsp/template-props-hover-3952/controls.json"
    ))
    .unwrap();
    for case in cases.as_array().unwrap() {
        let source = case["source"].as_str().unwrap();
        let start = source.find("<template>").unwrap();
        let needle = &source[start..][source[start..].find("props.").unwrap()..];
        let mut project = Fixture::new_with_vue_component_project(source, "App.vue", &[]);
        assert_eq!(project.open(source), json!([]), "{}", case["name"]);
        let point = super::project::position(source, needle);
        let expected = json!({
            "contents": {"kind":"markdown", "value":format!("```typescript\n{}\n```",case["signature"].as_str().unwrap())},
            "range": {"start":point,"end":{"line":point["line"],"character":point["character"].as_u64().unwrap()+5}}
        });
        assert_eq!(
            project.request("textDocument/hover", source, needle),
            expected,
            "{}",
            case["name"]
        );
        project.shutdown();
    }
}

#[test]
fn native_template_props_keep_top_level_readonly_and_nested_members_writable() {
    let valid = N8N.replace("props.params.value", "props.params.value = 'changed'");
    let mut project = Fixture::new_with_vue_component_project(&valid, "App.vue", &[]);
    assert_eq!(project.open(&valid), json!([]));
    let invalid = valid.replace(
        "props.params.value = 'changed'",
        "props.params = { value: 'changed' }",
    );
    let mut start = super::project::position(&invalid, "props.params =");
    start["character"] = (start["character"].as_u64().unwrap() + 6).into();
    let end = json!({"line":start["line"],"character":start["character"].as_u64().unwrap()+6});
    assert_eq!(
        project.change(&invalid, 2),
        json!([{
            "code":2540,
            "message":"Cannot assign to 'params' because it is a read-only property.",
            "range":{"start":start,"end":end},
            "severity":1,
            "source":"vize/types"
        }])
    );
    assert_eq!(project.change(&valid, 3), json!([]));
    project.shutdown();
}

//! Actual stdio LSP requests expose imported, registered, global, and native tags.
#![cfg(test)]
#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod project;
use project::Fixture;
use serde_json::{Value, json};

const SOURCE: &str = include_str!("fixtures/tag-completion/Parent.vue");

fn completion(fixture: &mut Fixture, source: &str, needle: &str) -> Vec<Value> {
    let result = fixture.request_with(
        "textDocument/completion",
        source,
        needle,
        json!({"context":{"triggerKind":1}}),
    );
    result
        .as_array()
        .or_else(|| result["items"].as_array())
        .expect("completion list")
        .clone()
}

fn selected(items: &[Value]) -> Vec<Value> {
    items
        .iter()
        .map(|item| json!({"label":item["label"],"textEdit":item["textEdit"]}))
        .collect()
}

#[test]
fn tag_prefixes_and_unsaved_names_have_exact_native_lsp_edits() {
    let mut fixture = Fixture::new(SOURCE, false);
    fixture.write_file(
        "Child.vue",
        include_str!("fixtures/tag-completion/Child.vue"),
    );
    fixture.write_file(
        "components.d.ts",
        "export {}\ndeclare module 'vue' { interface GlobalComponents { GlobalCard: unknown } }",
    );
    let _ = fixture.open(SOURCE);
    for (index, (prefix, expected)) in [
        ("<Chi", "Child"),
        ("<chi", "child"),
        ("<sp", "span"),
        ("<Global", "GlobalCard"),
    ]
    .into_iter()
    .enumerate()
    {
        let source = SOURCE.replace("<Chi", prefix);
        let _ = fixture.change(&source, index as i64 + 2);
        let items = completion(&mut fixture, &source, prefix);
        assert_eq!(
            selected(&items),
            [
                json!({"label":expected,"textEdit":{"range":{"start":{"line":5,"character":3},"end":{"line":5,"character":2+prefix.len()}},"newText":expected}})
            ]
        );
    }
    let renamed = SOURCE
        .replace("import Child", "import OtherChild")
        .replace("<Chi", "<Other");
    let _ = fixture.change(&renamed, 6);
    assert_eq!(
        selected(&completion(&mut fixture, &renamed, "<Other")),
        [
            json!({"label":"OtherChild","textEdit":{"range":{"start":{"line":5,"character":3},"end":{"line":5,"character":8}},"newText":"OtherChild"}})
        ]
    );
    let options = SOURCE
        .replace("<script setup", "<script")
        .replace(
            "</script>",
            "export default { components: { LocalCard: Child } }\n</script>",
        )
        .replace("<Chi", "<Local");
    let _ = fixture.change(&options, 7);
    assert_eq!(
        selected(&completion(&mut fixture, &options, "<Local")),
        [
            json!({"label":"LocalCard","textEdit":{"range":{"start":{"line":6,"character":3},"end":{"line":6,"character":8}},"newText":"LocalCard"}})
        ]
    );
    fixture.shutdown();
}

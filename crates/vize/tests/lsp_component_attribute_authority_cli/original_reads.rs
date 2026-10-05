//! Whole original read/prepare vectors accompany the mandatory four-edit rename.

use serde_json::{Value, json};

use super::{APP, CLASSY, Fixture, ITEM, TYPES, range};

#[test]
fn original_key_and_all_suffix_reads_keep_full_vectors_after_unsaved_utf16_shifts() {
    for newline in ["\n", "\r\n"] {
        let app = APP.replace('\n', newline);
        let classy = CLASSY.replace('\n', newline);
        let item = ITEM.replace('\n', newline);
        let types = TYPES.replace('\n', newline);
        let mut fixture = Fixture::new(&[
            ("src/Item.vue", &item),
            ("src/types.ts", &types),
            ("src/App.vue", &app),
            ("src/Classy.vue", &classy),
        ]);
        fixture.open("src/App.vue", &app, 1);
        fixture.open("src/Classy.vue", &classy, 1);
        assert_key(&mut fixture, &app);
        assert_suffix(&mut fixture, &classy);
        let changed_app = format!("<!-- 😀 unsaved -->{newline}{app}");
        let changed_classy = format!("<!-- 😀 unsaved -->{newline}{classy}");
        fixture.change("src/App.vue", &changed_app, 2, json!([]));
        fixture.change("src/Classy.vue", &changed_classy, 2, json!([]));
        assert_key(&mut fixture, &changed_app);
        assert_suffix(&mut fixture, &changed_classy);
        fixture.change("src/App.vue", &app, 3, json!([]));
        fixture.change("src/Classy.vue", &classy, 3, json!([]));
        assert_key(&mut fixture, &app);
        assert_suffix(&mut fixture, &classy);
        fixture.shutdown();
    }
}

pub(super) fn at(source: &str, offset: usize) -> Value {
    range(source, offset, "")["start"].clone()
}

fn assert_key(fixture: &mut Fixture, source: &str) {
    let key = source.find(":key=\"`entry-${i}`\"").unwrap() + 2;
    for method in ["textDocument/prepareRename", "textDocument/rename"] {
        assert_eq!(
            fixture.request(
                "src/App.vue",
                method,
                at(source, key),
                json!({"newName":"keyZz"})
            ),
            Value::Null
        );
    }
    for include in [false, true] {
        let result = fixture.request(
            "src/App.vue",
            "textDocument/references",
            at(source, key),
            json!({"context":{"includeDeclaration":include}}),
        );
        let _: Vec<lsp_types::Location> = serde_json::from_value(result.clone()).unwrap();
        assert_eq!(result, json!([]), "key has no defineEmits identity");
    }
    let result = fixture.request(
        "src/App.vue",
        "textDocument/definition",
        at(source, key),
        json!({}),
    );
    assert_eq!(result, Value::Null, "key has no child defineProps member");
    for (attribute, member) in [(":item-kind", "itemKind"), (":label", "label")] {
        let offset = source.find(attribute).unwrap() + 2;
        let result = fixture.request(
            "src/App.vue",
            "textDocument/definition",
            at(source, offset),
            json!({}),
        );
        let _: lsp_types::GotoDefinitionResponse = serde_json::from_value(result.clone()).unwrap();
        assert_eq!(
            result,
            json!({
                "uri":fixture.uri("src/Item.vue"),
                "range":range(ITEM, ITEM.find(&format!("{member}?:")).unwrap(), member)
            }),
            "a real known child prop keeps its complete member target"
        );
    }
}

fn assert_suffix(fixture: &mut Fixture, source: &str) {
    let offsets = [
        "const hasSuffix",
        "v-if=\"hasSuffix",
        "{{ hasSuffix",
        "'with-gap': hasSuffix",
    ]
    .map(|prefix| source.find(prefix).unwrap() + prefix.len() - "hasSuffix".len());
    let locations = offsets.map(|offset| {
        json!({"uri":fixture.uri("src/Classy.vue"),"range":range(source,offset,"hasSuffix")})
    });
    for offset in offsets {
        let position = at(source, offset + 1);
        let definition = fixture.request(
            "src/Classy.vue",
            "textDocument/definition",
            position.clone(),
            json!({}),
        );
        let _: lsp_types::GotoDefinitionResponse =
            serde_json::from_value(definition.clone()).unwrap();
        assert_eq!(definition, locations[0]);
        let prepared = fixture.request(
            "src/Classy.vue",
            "textDocument/prepareRename",
            position.clone(),
            json!({}),
        );
        let _: lsp_types::PrepareRenameResponse = serde_json::from_value(prepared.clone()).unwrap();
        assert_eq!(prepared, range(source, offset, "hasSuffix"));
        for include in [false, true] {
            let result = fixture.request(
                "src/Classy.vue",
                "textDocument/references",
                position.clone(),
                json!({"context":{"includeDeclaration":include}}),
            );
            let _: Vec<lsp_types::Location> = serde_json::from_value(result.clone()).unwrap();
            let expected = if include {
                json!(locations)
            } else {
                json!(&locations[1..])
            };
            assert_eq!(
                result, expected,
                "every original occurrence retains the full vector"
            );
        }
        let result = fixture.request(
            "src/Classy.vue",
            "textDocument/rename",
            position,
            json!({"newName":"showSuffix"}),
        );
        let expected = offsets.map(|offset| {
            json!({
                "range":range(source,offset,"hasSuffix"),"newText":"showSuffix"
            })
        });
        assert_eq!(
            result,
            json!({"changes":{fixture.uri("src/Classy.vue"):expected}})
        );
        apply_complete(source, result, "hasSuffix", "showSuffix");
    }
}

pub(super) fn apply_complete(source: &str, actual: Value, old: &str, new: &str) {
    apply_expected(source, actual, &source.replace(old, new));
}

pub(super) fn apply_expected(source: &str, actual: Value, expected: &str) {
    let actual: lsp_types::WorkspaceEdit = serde_json::from_value(actual).unwrap();
    let mut edits = actual
        .changes
        .unwrap()
        .into_values()
        .flatten()
        .collect::<Vec<_>>();
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.start));
    let mut applied = source.to_owned();
    for edit in edits {
        let start = super::byte_offset(source, edit.range.start);
        let end = super::byte_offset(source, edit.range.end);
        applied.replace_range(start..end, &edit.new_text);
    }
    assert_eq!(applied, expected);
}

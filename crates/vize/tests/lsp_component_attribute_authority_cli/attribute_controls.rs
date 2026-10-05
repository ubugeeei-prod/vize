//! The reporter's static style and nested v-for bindings retain separate identities.

use serde_json::{Value, json};

use super::original_reads::{apply_expected, at};
use super::{APP, Fixture, ITEM, TYPES, range};

const CHILD: &str = "<script setup lang=\"ts\">\ndefineProps<{ size: string; label: string }>();\n</script>\n<template><button>{{ size }}{{ label }}</button></template>\n";
const STYLE: &str = "<script setup lang=\"ts\">\nimport Child from './Child.vue';\n</script>\n<template>😀<Child size=\"large\" style=\"width: 40%\" label=\"Go\" /></template>\n";

#[test]
fn static_component_style_does_not_prepare_or_rename_its_import_binding() {
    for newline in ["\n", "\r\n"] {
        let source = STYLE.replace('\n', newline);
        let child = CHILD.replace('\n', newline);
        let mut fixture = Fixture::new(&[("src/Style.vue", &source), ("src/Child.vue", &child)]);
        fixture.open("src/Style.vue", &source, 1);
        assert_style(&mut fixture, &source, &child);
        let changed = format!("<!-- 😀 unsaved -->{newline}{source}");
        fixture.change("src/Style.vue", &changed, 2, json!([]));
        assert_style(&mut fixture, &changed, &child);
        fixture.change("src/Style.vue", &source, 3, json!([]));
        assert_style(&mut fixture, &source, &child);
        fixture.shutdown();
    }
}

fn assert_style(fixture: &mut Fixture, source: &str, child: &str) {
    let style = source.find("style=\"width: 40%\"").unwrap() + 1;
    for method in [
        "textDocument/prepareRename",
        "textDocument/rename",
        "textDocument/definition",
    ] {
        assert_eq!(
            fixture.request(
                "src/Style.vue",
                method,
                at(source, style),
                json!({"newName":"styleZz"})
            ),
            Value::Null
        );
    }
    for include in [false, true] {
        let actual = fixture.request(
            "src/Style.vue",
            "textDocument/references",
            at(source, style),
            json!({"context":{"includeDeclaration":include}}),
        );
        let _: Vec<lsp_types::Location> = serde_json::from_value(actual.clone()).unwrap();
        assert_eq!(actual, json!([]));
    }
    for member in ["size", "label"] {
        let offset = source.find(&format!("{member}=\"")).unwrap() + 1;
        let actual = fixture.request(
            "src/Style.vue",
            "textDocument/definition",
            at(source, offset),
            json!({}),
        );
        let _: lsp_types::GotoDefinitionResponse = serde_json::from_value(actual.clone()).unwrap();
        assert_eq!(
            actual,
            json!({"uri":fixture.uri("src/Child.vue"),"range":range(child,child.find(&format!("{member}:")).unwrap(),member)})
        );
    }
    // The real component import itself retains its existing whole binding rename.
    let offsets = [
        source.find("import Child").unwrap() + 7,
        source.find("<Child").unwrap() + 1,
    ];
    let actual = fixture.request(
        "src/Style.vue",
        "textDocument/rename",
        at(source, offsets[0] + 1),
        json!({"newName":"RenamedChild"}),
    );
    let expected = offsets
        .map(|offset| json!({"range":range(source,offset,"Child"),"newText":"RenamedChild"}));
    assert_eq!(
        actual,
        json!({"changes":{fixture.uri("src/Style.vue"):expected}})
    );
    let applied = source
        .replace("import Child ", "import RenamedChild ")
        .replace("<Child ", "<RenamedChild ");
    apply_expected(source, actual, &applied);
}

#[test]
fn original_nested_loop_aliases_keep_exact_local_rename_and_read_vectors() {
    for newline in ["\n", "\r\n"] {
        let source = APP.replace('\n', newline);
        let item = ITEM.replace('\n', newline);
        let types = TYPES.replace('\n', newline);
        let mut fixture = Fixture::new(&[
            ("src/App.vue", &source),
            ("src/Item.vue", &item),
            ("src/types.ts", &types),
        ]);
        fixture.open("src/App.vue", &source, 1);
        assert_loops(&mut fixture, &source);
        let changed = format!("<!-- 😀 unsaved -->{newline}{source}");
        fixture.change("src/App.vue", &changed, 2, json!([]));
        assert_loops(&mut fixture, &changed);
        fixture.change("src/App.vue", &source, 3, json!([]));
        assert_loops(&mut fixture, &source);
        fixture.shutdown();
    }
}

fn assert_loops(fixture: &mut Fixture, source: &str) {
    for (name, prefixes, renamed) in [
        ("index", ["(row, index)", "row-${index}"], "rowIndex"),
        ("i", ["(entry, i)", "entry-${i}"], "entryIndex"),
    ] {
        let offsets =
            prefixes.map(|prefix| source.find(prefix).unwrap() + prefix.find(name).unwrap());
        let locations = offsets.map(
            |offset| json!({"uri":fixture.uri("src/App.vue"),"range":range(source,offset,name)}),
        );
        for offset in offsets {
            let position = at(source, offset);
            let prepared = fixture.request(
                "src/App.vue",
                "textDocument/prepareRename",
                position.clone(),
                json!({}),
            );
            let _: lsp_types::PrepareRenameResponse =
                serde_json::from_value(prepared.clone()).unwrap();
            assert_eq!(prepared, range(source, offset, name));
            let definition = fixture.request(
                "src/App.vue",
                "textDocument/definition",
                position.clone(),
                json!({}),
            );
            let _: lsp_types::GotoDefinitionResponse =
                serde_json::from_value(definition.clone()).unwrap();
            assert_eq!(definition, locations[0]);
            for include in [false, true] {
                let actual = fixture.request(
                    "src/App.vue",
                    "textDocument/references",
                    position.clone(),
                    json!({"context":{"includeDeclaration":include}}),
                );
                let _: Vec<lsp_types::Location> = serde_json::from_value(actual.clone()).unwrap();
                assert_eq!(
                    actual,
                    if include {
                        json!(locations)
                    } else {
                        json!(&locations[1..])
                    }
                );
            }
            let actual = fixture.request(
                "src/App.vue",
                "textDocument/rename",
                position,
                json!({"newName":renamed}),
            );
            let expected =
                offsets.map(|offset| json!({"range":range(source,offset,name),"newText":renamed}));
            assert_eq!(
                actual,
                json!({"changes":{fixture.uri("src/App.vue"):expected}})
            );
            // Replace only the complete authored alias spans, not substring `i` in other names.
            let typed: lsp_types::WorkspaceEdit = serde_json::from_value(actual).unwrap();
            let mut edits = typed
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
            let mut expected = source.to_owned();
            for offset in offsets.into_iter().rev() {
                expected.replace_range(offset..offset + name.len(), renamed);
            }
            assert_eq!(applied, expected);
        }
    }
}

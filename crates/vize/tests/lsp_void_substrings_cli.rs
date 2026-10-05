#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use serde_json::{Value, json};

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod support;
use support::{Fixture, position};

const ORIGINAL: &str =
    include_str!("../../../tests/_fixtures/differential/lsp/void-substring-bindings/App.vue.txt");
const CONFIG: &str =
    include_str!("../../../tests/_fixtures/differential/lsp/void-substring-bindings/tsconfig.json");
const NAMES: &[&str] = &[
    "v", "o", "i", "id", "vo", "oid", "voi", "ix", "idx", "ID", "void2", "n", "x", "e", "c", "t",
    "name", "key", "ref", "type", "value", "slot", "is", "style", "tag",
];

#[test]
fn original_report_preserves_complete_typed_hover_references_and_rename() {
    let mut fixture = Fixture::new_with_vue(ORIGINAL);
    fixture.write_file("tsconfig.json", CONFIG);
    assert_eq!(fixture.open(ORIGINAL), json!([]));
    assert_binding(
        &mut fixture,
        ORIGINAL,
        "id",
        "1",
        &["id =", "id }}", "id +"],
    );
    assert_binding(
        &mut fixture,
        ORIGINAL,
        "name",
        "\"a\"",
        &["name =", "name }}"],
    );
    fixture.shutdown();
}

#[test]
fn every_reported_name_preserves_utf16_identity_shadows_and_unsaved_edits() {
    for newline in ["\n", "\r\n"] {
        let mut source = String::from("<script setup lang=\"ts\">\n");
        for name in NAMES {
            source.push_str(&format!("const {name} = 1;\n"));
        }
        source.push_str("function inner(id: number, i: number) { return id + i; }\nconst literal = 'id i void'; // id i\n</script>\n<template>\n<p>{{ '😀' }}");
        for name in NAMES {
            source.push_str(&format!(" {{{{ {name} }}}} {{{{ {name} + 1 }}}}"));
        }
        source.push_str("</p>\n</template>\n");
        let source = source.replace('\n', newline);
        let mut fixture = Fixture::new_with_vue(&source);
        fixture.write_file("tsconfig.json", CONFIG);
        assert_eq!(fixture.open(&source), json!([]));
        for name in NAMES {
            let declaration = format!("const {name} =");
            let bare = format!("{{{{ {name} }}}}");
            let compound = format!("{{{{ {name} + 1 }}}}");
            // Full occurrence delimiters keep `x` distinct from `ix` and `idx`.
            let offsets = [
                source.find(&declaration).unwrap() + "const ".len(),
                source.find(&bare).unwrap() + "{{ ".len(),
                source.find(&compound).unwrap() + "{{ ".len(),
            ];
            assert_binding_at(&mut fixture, &source, name, "1", &offsets);
        }
        let offsets = [
            source.find("id: number").unwrap(),
            source.find("id + i").unwrap(),
        ];
        assert_references_and_rename(&mut fixture, &source, "id", &offsets);
        fixture.shutdown();
    }
}

fn assert_binding(fixture: &mut Fixture, source: &str, name: &str, ty: &str, needles: &[&str]) {
    let offsets: Vec<_> = needles
        .iter()
        .map(|needle| source.find(needle).unwrap())
        .collect();
    assert_binding_at(fixture, source, name, ty, &offsets);
}

fn assert_binding_at(fixture: &mut Fixture, source: &str, name: &str, ty: &str, offsets: &[usize]) {
    for &offset in offsets {
        // `request_with` accepts a unique suffix, preserving the exact authored position.
        let needle = &source[offset..];
        assert_eq!(
            fixture.request("textDocument/hover", source, needle),
            json!({
                "contents": { "kind": "markdown", "value": format!("```typescript\nconst {name}: {ty}\n```") },
                "range": range(source, offset, name)
            }),
            "hover for {name} at {offset}"
        );
    }
    assert_references_and_rename(fixture, source, name, offsets);
}

fn assert_references_and_rename(
    fixture: &mut Fixture,
    source: &str,
    name: &str,
    offsets: &[usize],
) {
    let ranges: Vec<_> = offsets
        .iter()
        .map(|&offset| range(source, offset, name))
        .collect();
    for &offset in offsets {
        let needle = &source[offset..];
        for include_declaration in [true, false] {
            let expected: Vec<_> = ranges
                .iter()
                .skip(usize::from(!include_declaration))
                .map(|range| json!({ "uri": fixture.uri, "range": range }))
                .collect();
            assert_eq!(
                fixture.request_with(
                    "textDocument/references",
                    source,
                    needle,
                    json!({ "context": { "includeDeclaration": include_declaration } })
                ),
                json!(expected),
                "references for {name} at {offset}, includeDeclaration={include_declaration}"
            );
        }
        let edits: Vec<_> = ranges
            .iter()
            .map(|range| json!({ "range": range, "newText": "renamedX" }))
            .collect();
        let rename = fixture.request_with(
            "textDocument/rename",
            source,
            needle,
            json!({ "newName": "renamedX" }),
        );
        assert_eq!(
            rename,
            json!({ "changes": { &fixture.uri: edits } }),
            "rename for {name} at {offset}"
        );
        let mut expected = source.to_owned();
        for &offset in offsets.iter().rev() {
            expected.replace_range(offset..offset + name.len(), "renamedX");
        }
        let actual = apply_edits(source, rename["changes"][&fixture.uri].as_array().unwrap());
        assert_eq!(
            actual, expected,
            "whole authored source after {name} rename"
        );
        // Restore the original after each real unsaved rename to retain identical requests.
        assert_eq!(fixture.change(&actual, next_version()), json!([]));
        assert_eq!(fixture.change(source, next_version()), json!([]));
    }
}

fn next_version() -> i64 {
    static VERSION: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(2);
    VERSION.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn range(source: &str, offset: usize, token: &str) -> Value {
    let start = position(source, &source[offset..]);
    let end = json!({ "line": start["line"], "character": start["character"].as_u64().unwrap() + token.encode_utf16().count() as u64 });
    json!({ "start": start, "end": end })
}

fn apply_edits(source: &str, edits: &[Value]) -> String {
    let mut result = source.to_owned();
    for edit in edits.iter().rev() {
        let start = byte_offset(source, &edit["range"]["start"]);
        let end = byte_offset(source, &edit["range"]["end"]);
        result.replace_range(start..end, edit["newText"].as_str().unwrap());
    }
    result
}

fn byte_offset(source: &str, position: &Value) -> usize {
    let line = position["line"].as_u64().unwrap() as usize;
    let units = position["character"].as_u64().unwrap() as usize;
    let line_offset: usize = source.split_inclusive('\n').take(line).map(str::len).sum();
    let mut seen = 0;
    for (index, character) in source[line_offset..].char_indices() {
        if seen == units {
            return line_offset + index;
        }
        seen += character.len_utf16();
        assert!(seen <= units, "edit split a UTF-16 character");
    }
    assert_eq!(seen, units);
    source.len()
}

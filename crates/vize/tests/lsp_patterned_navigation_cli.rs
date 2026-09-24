#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use serde_json::{Value, json};

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod support;
use support::{Fixture, position};

const SOURCE: &str = r#"<script setup lang="ts">
const rows = 'outer'
const result = {} as { kind: 'ok'; rows: number[] } | { kind: 'err'; rows: string }
</script>
<template v-match="result">
  <template v-when="{ kind: 'ok', const rows } if (rows.length > 0)">
    <p>{{ rows.length }}</p>
    <p v-for="rows in rows">{{ rows.toFixed() }}</p>
  </template>
  <p v-when="{ kind: 'err', const rows }">{{ rows.toUpperCase() }}</p>
  <p v-when="_">{{ rows.toUpperCase() }}</p>
</template>
<style scoped>p { color: v-bind(rows); }</style>"#;

fn range(source: &str, needle: &str, len: u64) -> Value {
    let start = position(source, needle);
    let mut end = start.clone();
    end["character"] = json!(start["character"].as_u64().unwrap() + len);
    json!({ "start": start, "end": end })
}

fn references(fixture: &mut Fixture, source: &str, needle: &str, declarations: bool) -> Value {
    fixture.request_with(
        "textDocument/references",
        source,
        needle,
        json!({ "context": { "includeDeclaration": declarations } }),
    )
}

fn edits(edit: &Value, uri: &str) -> Vec<Value> {
    let mut edits = Vec::new();
    if let Some(changes) = edit["changes"].as_object() {
        for (path, changes) in changes {
            assert_eq!(path, uri, "unexpected edited document: {edit:#}");
            edits.extend(changes.as_array().unwrap().iter().cloned());
        }
    }
    if let Some(changes) = edit["documentChanges"].as_array() {
        for change in changes {
            assert_eq!(change["textDocument"]["uri"], uri);
            edits.extend(change["edits"].as_array().unwrap().iter().cloned());
        }
    }
    edits.sort_by_key(|edit| {
        (
            edit["range"]["start"]["line"].as_u64().unwrap(),
            edit["range"]["start"]["character"].as_u64().unwrap(),
        )
    });
    edits
}

fn rename_edits(source: &str) -> Vec<Value> {
    vec![
        json!({ "range": range(source, "const rows } if", 10), "newText": "rows: const values" }),
        json!({ "range": range(source, "rows.length >", 4), "newText": "values" }),
        json!({ "range": range(source, "rows.length }}", 4), "newText": "values" }),
        json!({ "range": range(source, "rows\">", 4), "newText": "values" }),
    ]
}

fn apply(source: &str, edits: &[Value]) -> String {
    let offset = |position: &Value| {
        let line = position["line"].as_u64().unwrap() as usize;
        let character = position["character"].as_u64().unwrap() as usize;
        let start: usize = source.split_inclusive('\n').take(line).map(str::len).sum();
        let mut utf16 = 0;
        source[start..]
            .char_indices()
            .find_map(|(at, ch)| {
                let matches = utf16 == character;
                utf16 += ch.len_utf16();
                matches.then_some(start + at)
            })
            .unwrap()
    };
    let mut result = source.to_owned();
    for edit in edits.iter().rev() {
        result.replace_range(
            offset(&edit["range"]["start"])..offset(&edit["range"]["end"]),
            edit["newText"].as_str().unwrap(),
        );
    }
    result
}

#[test]
fn native_pattern_rename_keeps_the_object_property_name() {
    let mut fixture = Fixture::new(SOURCE, true);
    assert_eq!(fixture.open(SOURCE), json!([]));
    let renamed = fixture.request_with(
        "textDocument/rename",
        SOURCE,
        "rows.length }}",
        json!({ "newName": "values" }),
    );
    assert_eq!(
        edits(&renamed, &fixture.uri),
        rename_edits(SOURCE),
        "{renamed:#}"
    );
    let expected = SOURCE
        .replace("const rows } if", "rows: const values } if")
        .replace("rows.length", "values.length")
        .replace("in rows", "in values");
    let repaired = apply(SOURCE, &edits(&renamed, &fixture.uri));
    assert_eq!(repaired, expected);
    assert_eq!(fixture.change(&repaired, 2), json!([]));
    fixture.shutdown();
}

#[test]
fn native_pattern_references_and_rename_respect_arm_and_loop_scopes() {
    let mut fixture = Fixture::new(SOURCE, true);
    assert_eq!(fixture.open(SOURCE), json!([]));
    let declaration = range(SOURCE, "rows } if", 4);
    let uses = [
        range(SOURCE, "rows.length >", 4),
        range(SOURCE, "rows.length }}", 4),
        range(SOURCE, "rows\">", 4),
    ];
    let all: Vec<_> = std::iter::once(&declaration)
        .chain(&uses)
        .map(|range| json!({ "uri": fixture.uri, "range": range }))
        .collect();
    for needle in ["rows } if", "rows.length >", "rows.length }}", "rows\">"] {
        let refs = references(&mut fixture, SOURCE, needle, true);
        assert_eq!(refs, json!(all), "references at {needle}: {refs:#}");
        let uses: Vec<_> = uses
            .iter()
            .map(|range| json!({ "uri": fixture.uri, "range": range }))
            .collect();
        assert_eq!(references(&mut fixture, SOURCE, needle, false), json!(uses));
        let prepare = fixture.request("textDocument/prepareRename", SOURCE, needle);
        assert_eq!(prepare, range(SOURCE, needle, 4), "{prepare:#}");
        let renamed = fixture.request_with(
            "textDocument/rename",
            SOURCE,
            needle,
            json!({ "newName": "values" }),
        );
        assert_eq!(
            edits(&renamed, &fixture.uri),
            rename_edits(SOURCE),
            "{renamed:#}"
        );
    }
    fixture.shutdown();
}

#[test]
fn native_pattern_single_unused_binding_does_not_fall_back_to_same_named_symbols() {
    let source = r#"<script setup lang="ts">const value = 1; const result = { value: 'ok' };</script>
<template v-match="result"><p v-when="{ const value }"/></template>"#;
    let mut fixture = Fixture::new(source, true);
    assert_eq!(fixture.open(source), json!([]));
    assert_eq!(
        references(&mut fixture, source, "value }", false),
        json!([])
    );
    let renamed = fixture.request_with(
        "textDocument/rename",
        source,
        "value }",
        json!({ "newName": "local" }),
    );
    assert_eq!(
        edits(&renamed, &fixture.uri),
        vec![
            json!({ "range": range(source, "const value }", 11), "newText": "value: const local" })
        ]
    );
    let repaired = apply(source, &edits(&renamed, &fixture.uri));
    assert_eq!(fixture.change(&repaired, 2), json!([]));
    fixture.shutdown();
}

#[test]
fn native_outer_binding_keeps_style_references_without_pattern_binding_leaks() {
    let mut fixture = Fixture::new(SOURCE, true);
    assert_eq!(fixture.open(SOURCE), json!([]));
    let ranges = [
        range(SOURCE, "rows =", 4),
        range(SOURCE, "rows.toUpperCase() }}</p>\n</template>", 4),
        range(SOURCE, "rows);", 4),
    ];
    for declarations in [false, true] {
        let expected: Vec<_> = ranges
            .iter()
            .skip(usize::from(!declarations))
            .map(|range| json!({ "uri": fixture.uri, "range": range }))
            .collect();
        assert_eq!(
            references(&mut fixture, SOURCE, "rows =", declarations),
            json!(expected)
        );
    }
    let renamed = fixture.request_with(
        "textDocument/rename",
        SOURCE,
        "rows =",
        json!({ "newName": "outer" }),
    );
    let expected: Vec<_> = ranges
        .iter()
        .map(|range| json!({ "range": range, "newText": "outer" }))
        .collect();
    assert_eq!(edits(&renamed, &fixture.uri), expected);
    assert_eq!(fixture.change(&apply(SOURCE, &expected), 2), json!([]));
    fixture.shutdown();
}

#[test]
fn native_pattern_unicode_crlf_nested_rename_preserves_shorthand_trivia() {
    let source = SOURCE
        .replace("<template v-match=", "<template>\n<section v-match=")
        .replace("</template>", "</section>\n</template>")
        .replacen("</section>\n</template>", "</template>", 1)
        .replace("rows", "\u{6570}\u{5024}")
        .replace(
            "const \u{6570}\u{5024} } if",
            "const  \u{6570}\u{5024} } if",
        )
        .replace("<p>{{", "<p>\u{1f600}{{")
        .replace('\n', "\r\n");
    let mut fixture = Fixture::new(&source, true);
    assert_eq!(fixture.open(&source), json!([]));
    let renamed = fixture.request_with(
        "textDocument/rename",
        &source,
        "\u{6570}\u{5024}.length }}",
        json!({ "newName": "values" }),
    );
    let expected = source
        .replace(
            "const  \u{6570}\u{5024} } if",
            "\u{6570}\u{5024}: const  values } if",
        )
        .replace("\u{6570}\u{5024}.length", "values.length")
        .replace("in \u{6570}\u{5024}", "in values");
    let repaired = apply(&source, &edits(&renamed, &fixture.uri));
    assert_eq!(repaired, expected, "{renamed:#}");
    assert_eq!(fixture.change(&repaired, 2), json!([]));
    fixture.shutdown();
}

#[test]
fn native_nested_patterns_keep_array_rest_alias_and_callback_identities() {
    let source = r#"<script setup lang="ts">
const groups = [[1, 2]] as number[][]
const head = 'outer'
</script>
<template v-match="groups">
  <template v-when="[const group, ...const remaining] as whole">
    <section v-match="group">
      <p v-when="[const head, ...const tail]">{{ head.toFixed() }}{{ tail.length }}{{ [head].map(head => head.toFixed()) }}</p>
      <p v-when="_"/>
    </section>
    {{ remaining.length }}{{ whole.length }}
  </template>
  <p v-when="_">{{ head.toUpperCase() }}</p>
</template>"#;
    for cross_file in [false, true] {
        let mut fixture = Fixture::new_with_cross_file(source, true, cross_file);
        assert_eq!(fixture.open(source), json!([]));
        for (index, (declaration, uses, name, size)) in [
            ("head,", vec!["head.toFixed() }}", "head].map"], "first", 4),
            ("remaining]", vec!["remaining.length"], "rest", 9),
            ("whole\">", vec!["whole.length"], "all", 5),
        ]
        .into_iter()
        .enumerate()
        {
            let ranges: Vec<_> = std::iter::once(declaration)
                .chain(uses.iter().copied())
                .map(|needle| range(source, needle, size))
                .collect();
            let expected: Vec<_> = ranges
                .iter()
                .map(|range| json!({ "uri": fixture.uri, "range": range }))
                .collect();
            assert_eq!(
                references(&mut fixture, source, uses[0], true),
                json!(expected)
            );
            let definition = fixture.request("textDocument/definition", source, uses[0]);
            assert_eq!(
                definition,
                json!({ "uri": fixture.uri, "range": ranges[0] })
            );
            let renamed = fixture.request_with(
                "textDocument/rename",
                source,
                uses[0],
                json!({ "newName": name }),
            );
            let expected: Vec<_> = ranges
                .iter()
                .map(|range| json!({ "range": range, "newText": name }))
                .collect();
            assert_eq!(edits(&renamed, &fixture.uri), expected);
            let changed = apply(source, &expected);
            assert_eq!(fixture.change(&changed, 2 + index as i64 * 2), json!([]));
            assert_eq!(fixture.change(source, 3 + index as i64 * 2), json!([]));
        }
        let malformed = source.replace("...const tail", "...const head");
        let diagnostics = fixture.change(&malformed, 8);
        assert!(
            diagnostics
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["code"] == "patterned-template")
        );
        assert_eq!(
            fixture.request_with(
                "textDocument/rename",
                &malformed,
                "head.toFixed() }}",
                json!({ "newName": "bad" })
            ),
            Value::Null
        );
        assert_eq!(fixture.change(source, 9), json!([]));
        fixture.shutdown();
    }
}

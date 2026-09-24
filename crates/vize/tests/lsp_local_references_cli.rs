#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
use serde_json::{Value, json};

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod support;
use support::{Fixture, position};

#[test]
fn document_references_preserve_shadowed_identity_and_unsaved_edits() {
    for newline in ["\n", "\r\n"] {
        let source = "<script setup lang=\"ts\">\nconst prefix = '\u{1f600}'; const local = 'fixed';\nfunction inner(local: number) { return local + 1; }\nconst unused = 1;\n</script>\n<template>{{ local.toUpperCase() }}</template>\n".replace('\n', newline);
        let mut fixture = Fixture::new_with_vue(&source);
        assert_eq!(fixture.open(&source), json!([]));
        assert_identity(&mut fixture, &source, &["local =", "local.toUpperCase"]);
        assert_identity(&mut fixture, &source, &["local: number", "local +"]);
        assert_identity(&mut fixture, &source, &["unused ="]);

        let ambient = source.replace(
            "function inner",
            &format!("declare const label: typeof local;{newline}function inner"),
        );
        assert_eq!(fixture.change(&ambient, 2), json!([]));
        assert_identity(
            &mut fixture,
            &ambient,
            &["local =", "local;", "local.toUpperCase"],
        );
        assert_identity(&mut fixture, &ambient, &["local: number", "local +"]);
        assert_eq!(fixture.change(&source, 3), json!([]));
        assert_identity(&mut fixture, &source, &["local =", "local.toUpperCase"]);
        fixture.shutdown();
    }
}

#[test]
fn document_references_exclude_importers_but_cross_file_mode_keeps_them() {
    let source = "<script lang=\"ts\">\nexport const shared = 1;\nconst copy = shared;\n</script>\n<template><div /></template>\n";
    let importer = "<script setup lang=\"ts\">\nimport { shared } from './App.vue';\nconst copy = shared;\n</script>\n<template>{{ shared }}</template>\n";
    for cross_file in [false, true] {
        let mut fixture = if cross_file {
            Fixture::new_with_vue_and_cross_file(source)
        } else {
            Fixture::new_with_vue(source)
        };
        let importer_uri = fixture.write_file("Consumer.vue", importer);
        assert_eq!(fixture.open(source), json!([]));
        for include_declaration in [true, false] {
            let mut expected: Vec<_> = ["shared =", "shared;"]
                .into_iter()
                .skip(usize::from(!include_declaration))
                .map(|needle| json!({ "uri": fixture.uri, "range": token_range(source, needle, "shared") }))
                .collect();
            if cross_file {
                expected.extend(
                    ["shared }", "shared;", "shared }}"]
                        .into_iter()
                        .skip(usize::from(!include_declaration))
                        .map(|needle| json!({ "uri": importer_uri, "range": token_range(importer, needle, "shared") })),
                );
            }
            assert_eq!(
                fixture.request_with(
                    "textDocument/references",
                    source,
                    "shared =",
                    json!({ "context": { "includeDeclaration": include_declaration } }),
                ),
                json!(expected),
                "crossFile={cross_file}, includeDeclaration={include_declaration}"
            );
        }
        fixture.shutdown();
    }
}

#[test]
fn plain_exports_keep_local_references_and_rename_identity() {
    for newline in ["\n", "\r\n"] {
        let source = "<script lang=\"ts\">\nexport const shared = 1;\nconst prefix = '\u{1f600}'; const copy = shared;\nfunction inner(shared: string) { return shared.toUpperCase(); }\nexport class Counter { value = 1; }\nconst counter: Counter = new Counter();\nexport enum Mode { One, Two }\nconst mode: Mode = Mode.One;\n</script>\n<template><div /></template>\n".replace('\n', newline);
        let mut fixture = Fixture::new_with_vue(&source);
        assert_eq!(fixture.open(&source), json!([]));
        assert_identity(&mut fixture, &source, &["shared =", "shared;"]);
        assert_identity(
            &mut fixture,
            &source,
            &["shared: string", "shared.toUpperCase"],
        );
        assert_identity(
            &mut fixture,
            &source,
            &["Counter {", "Counter =", "Counter()"],
        );
        assert_identity(&mut fixture, &source, &["Mode {", "Mode =", "Mode.One"]);
        let edited = source.replace("const copy = shared;", "const copy = shared + shared * 2;");
        assert_eq!(fixture.change(&edited, 2), json!([]));
        assert_identity(&mut fixture, &edited, &["shared =", "shared +", "shared *"]);
        assert_eq!(fixture.change(&source, 3), json!([]));
        assert_identity(&mut fixture, &source, &["shared =", "shared;"]);
        fixture.shutdown();
    }
}

#[test]
fn inline_exports_preserve_authored_identity_and_non_code_text() {
    for newline in ["\n", "\r\n"] {
        let source = "<script lang=\"ts\">\nconst prefix = '😀 export const fake = 0'; export /* keep */ const café = 1; export const second = café;\nconst literal = `export const untouched = 1`; /* export const hidden = 2 */\nexport\nconst third = second;\nconst copy = café;\n</script>\n<template><div /></template>\n".replace('\n', newline);
        let mut fixture = Fixture::new_with_vue(&source);
        assert_eq!(fixture.open(&source), json!([]));
        assert_identity(
            &mut fixture,
            &source,
            &[
                "café =",
                "café;",
                "café;\n</script>".replace('\n', newline).as_str(),
            ],
        );
        assert_identity(&mut fixture, &source, &["second =", "second;"]);
        assert_identity(&mut fixture, &source, &["third ="]);
        fixture.shutdown();
    }
}

#[test]
fn options_api_references_keep_data_declarations_without_parameter_shadows() {
    for newline in ["\n", "\r\n"] {
        let source = "<script lang=\"ts\">\nexport default {\n  data: () => ({\n    selected: null,\n  }),\n  methods: { echo(selected: string) { return selected.toUpperCase(); } },\n};\n</script>\n<template>{{ selected }}</template>\n".replace('\n', newline);
        let mut fixture = Fixture::new_with_vue_options_api(&source);
        assert_eq!(fixture.open(&source), json!([]));
        assert_identity(&mut fixture, &source, &["selected: null", "selected }}"]);
        assert_identity(
            &mut fixture,
            &source,
            &["selected: string", "selected.toUpperCase"],
        );
        fixture.shutdown();
    }
}

fn assert_identity(fixture: &mut Fixture, source: &str, needles: &[&str]) {
    let token = needles[0].split([' ', ':']).next().unwrap();
    let ranges: Vec<_> = needles
        .iter()
        .map(|needle| token_range(source, needle, token))
        .collect();
    for needle in needles {
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
                    json!({ "context": { "includeDeclaration": include_declaration } }),
                ),
                json!(expected),
                "references from {needle}, includeDeclaration={include_declaration}"
            );
        }
        let mut expected: Vec<_> = ranges
            .iter()
            .map(|range| json!({ "range": range, "newText": "renamed" }))
            .collect();
        let rename = fixture.request_with(
            "textDocument/rename",
            source,
            needle,
            json!({ "newName": "renamed" }),
        );
        let mut actual = rename["changes"][&fixture.uri]
            .as_array()
            .unwrap_or_else(|| panic!("{rename:#}"))
            .clone();
        sort_ranges(&mut actual);
        sort_ranges(&mut expected);
        assert_eq!(actual, expected, "rename from {needle}");
        assert_eq!(rename["changes"].as_object().unwrap().len(), 1);
        assert_eq!(rename.get("documentChanges"), None);
    }
}

fn sort_ranges(locations: &mut [Value]) {
    locations.sort_by_key(|location| {
        (
            location["range"]["start"]["line"].as_u64(),
            location["range"]["start"]["character"].as_u64(),
        )
    });
}

fn token_range(source: &str, needle: &str, token: &str) -> Value {
    let start = position(source, needle);
    let end = json!({ "line": start["line"], "character": start["character"].as_u64().unwrap() + token.encode_utf16().count() as u64 });
    json!({ "start": start, "end": end })
}

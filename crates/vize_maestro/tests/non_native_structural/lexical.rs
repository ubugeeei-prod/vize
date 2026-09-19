use tower_lsp::lsp_types::Url;
use vize_maestro::ide::IdeContext;
use vize_maestro::ide::references::ReferencesService;
use vize_maestro::server::ServerState;

fn references(source: &str, query: usize, declaration: bool) -> Option<Vec<(usize, usize)>> {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Structural.vue").unwrap();
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    let ctx = IdeContext::new(&state, &uri, query).unwrap();
    ReferencesService::references(&ctx, declaration).map(|locations| {
        locations
            .iter()
            .map(|location| {
                let start = vize_maestro::ide::position_to_offset(
                    source,
                    location.range.start.line,
                    location.range.start.character,
                )
                .unwrap();
                let end = vize_maestro::ide::position_to_offset(
                    source,
                    location.range.end.line,
                    location.range.end.character,
                )
                .unwrap();
                (start, end)
            })
            .collect()
    })
}

#[test]
fn nested_script_bindings_and_template_reads_keep_separate_identities() {
    for newline in ["\n", "\r\n"] {
        let source = "<script setup lang=\"ts\">\nconst emoji = '😀'; const café = 'fixed';\nfunction inner(café: number) { return café + 1; }\nconst text = 'café'; // café\nconst obj = { café: 1 };\n</script>\n<template>{{ café.toUpperCase() }}</template>".replace('\n', newline);
        let occurrences: Vec<_> = source
            .match_indices("café")
            .map(|(start, _)| (start, start + "café".len()))
            .collect();
        let outer = vec![occurrences[0], *occurrences.last().unwrap()];
        let inner = vec![occurrences[1], occurrences[2]];
        for query in outer.iter().map(|span| span.0) {
            assert_eq!(references(&source, query, true), Some(outer.clone()));
            assert_eq!(references(&source, query, false), Some(vec![outer[1]]));
        }
        for query in inner.iter().map(|span| span.0) {
            assert_eq!(references(&source, query, true), Some(inner.clone()));
            assert_eq!(references(&source, query, false), Some(vec![inner[1]]));
        }
        for query in occurrences[3..6].iter().map(|span| span.0) {
            assert_eq!(references(&source, query, true), None);
        }
    }
}

#[test]
fn v_for_slot_and_callback_locals_do_not_join_setup_references() {
    let source = "<script setup lang=\"ts\">\nconst item = 'outer'; const items = ['one'];\n</script>\n<template>{{ item }}<div v-for=\"item in items\">{{ item }}</div><Comp v-slot=\"{ item }\">{{ item }}</Comp><div>{{ items.map(item => item) }}</div></template>";
    let first = source.find("const item").unwrap() + 6;
    let outer_use = source.find("{{ item }}").unwrap() + 3;
    assert_eq!(
        references(source, first, true),
        Some(vec![(first, first + 4), (outer_use, outer_use + 4)])
    );
    let loop_definition = source.find("v-for=\"item").unwrap() + 7;
    let loop_use = source[loop_definition..].find("{{ item }}").unwrap() + loop_definition + 3;
    assert_eq!(
        references(source, loop_use, true),
        Some(vec![
            (loop_definition, loop_definition + 4),
            (loop_use, loop_use + 4)
        ])
    );
}

#[test]
fn unresolved_member_names_are_not_lexical_symbols() {
    let source = "<script setup>const key = 1; const object = { key: 2 }; object.key;</script><template>{{ key }}</template>";
    assert_eq!(
        references(source, source.find("object.key").unwrap() + 7, true),
        None
    );
}

#[test]
fn css_expressions_resolve_in_setup_scope_and_ignore_strings_comments_and_callback_locals() {
    let source = "<script setup>const color = 'red'; function inner(color) { return color; }</script><style>/* v-bind(color) */ .x { content: 'v-bind(color)'; color: v-bind(color); width: v-bind('color + \"px\"'); height: v-bind('((color) => color)(1)'); }</style>";
    let definition = source.find("color =").unwrap();
    let direct = source.find("color: v-bind(color)").unwrap() + "color: v-bind(".len();
    let expression = source.find("color +").unwrap();
    let expected = vec![
        (definition, definition + 5),
        (direct, direct + 5),
        (expression, expression + 5),
    ];
    for query in [definition, direct, expression] {
        assert_eq!(references(source, query, true), Some(expected.clone()));
    }
    let parameter = source.find("inner(color)").unwrap() + 6;
    let returned = source.find("return color").unwrap() + 7;
    assert_eq!(
        references(source, parameter, true),
        Some(vec![(parameter, parameter + 5), (returned, returned + 5)])
    );
}

fn renamed(source: &str, query: &str, new: &str) -> Option<String> {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Rename.vue").unwrap();
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    let ctx = IdeContext::new(&state, &uri, source.find(query).unwrap()).unwrap();
    let edit = vize_maestro::ide::rename::RenameService::rename(&ctx, new)?;
    let mut edits = edit.changes.unwrap().remove(&uri).unwrap();
    edits.sort_by_key(|edit| (edit.range.start.line, edit.range.start.character));
    let mut output = source.to_owned();
    for edit in edits.into_iter().rev() {
        let start = vize_maestro::ide::position_to_offset(
            source,
            edit.range.start.line,
            edit.range.start.character,
        )
        .unwrap();
        let end = vize_maestro::ide::position_to_offset(
            source,
            edit.range.end.line,
            edit.range.end.character,
        )
        .unwrap();
        output.replace_range(start..end, &edit.new_text);
    }
    Some(output)
}

#[test]
fn rename_preserves_shorthand_keys_imports_and_destructure_defaults() {
    for newline in ["\n", "\r\n"] {
        let source = "<script setup lang=\"ts\">\nimport { café } from './module';\nconst object = { café }; function inner(café: number) { return café; }\n</script><template>{{ café }}</template>".replace('\n', newline);
        let expected = source
            .replace("{ café } from", "{ café as label } from")
            .replace("object = { café }", "object = { café: label }")
            .replace("{{ café }}", "{{ label }}");
        assert_eq!(renamed(&source, "café } from", "label"), Some(expected));
        assert_eq!(renamed(&source, "café } from", "const"), None);
    }
    let source = "<script setup>const { value = 1 } = state; const object = { value };</script><template>{{ value }}</template>";
    let expected = "<script setup>const { value: count = 1 } = state; const object = { value: count };</script><template>{{ count }}</template>";
    assert_eq!(renamed(source, "value =", "count"), Some(expected.into()));
}

#[test]
fn rename_expands_same_name_bindings_without_changing_public_props() {
    let source = "<script setup>const count = 1;</script><template><Child :count/><Child v-bind:count.prop/><Child :count=\"count\"/></template>";
    let expected = "<script setup>const tally = 1;</script><template><Child :count=\"tally\"/><Child v-bind:count.prop=\"tally\"/><Child :count=\"tally\"/></template>";
    assert_eq!(renamed(source, "count =", "tally"), Some(expected.into()));
}

#[test]
fn rename_rejects_binding_collisions_and_capture_but_allows_disjoint_scopes() {
    for source in [
        "<script setup>const value = 1; const count = 2; consume(value);</script>",
        "<script setup>const value = 1; function inner(count) { return value; }</script>",
        "<script setup>const value = 1; consume(count);</script>",
        "<script setup>const value = 1;</script><template>{{ count }}</template>",
        "<script setup>const value = 1;</script><style>.x {color: v-bind(count)}</style>",
    ] {
        assert_eq!(renamed(source, "value =", "count"), None, "{source}");
    }
    let source = "<script setup>const value = 1; function inner(count) { return count; }</script><template>{{ value }}</template>";
    assert_eq!(
        renamed(source, "value =", "count"),
        Some(source.replace("value", "count"))
    );
    assert_eq!(
        renamed(source, "value =", "数"),
        Some(source.replace("value", "数"))
    );
}

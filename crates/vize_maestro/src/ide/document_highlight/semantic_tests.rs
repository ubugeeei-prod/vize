use super::tests::{READ, WRITE, highlights_at, state_for};
use crate::ide::CodeLensService;
use serde_json::json;

const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/binding-occurrences-original/Field.vue"
));

#[test]
fn original_bindings_keep_whole_ranges_and_exact_one_one_three_lenses() {
    for crlf in [false, true] {
        let source = if crlf {
            ORIGINAL.replace('\n', "\r\n")
        } else {
            ORIGINAL.into()
        };
        let (state, uri) = state_for(&source, "file:///Field.vue", "vue");
        assert_eq!(
            highlights_at(&state, &uri, &source, 1, 7),
            vec![(1, 6, 11, WRITE), (7, 22, 27, READ)]
        );
        assert_eq!(
            highlights_at(&state, &uri, &source, 2, 7),
            vec![(2, 6, 10, WRITE), (9, 27, 31, READ)]
        );
        let ids = vec![
            (3, 6, 8, WRITE),
            (7, 15, 17, READ),
            (8, 14, 16, READ),
            (9, 13, 15, READ),
        ];
        assert_eq!(highlights_at(&state, &uri, &source, 3, 7), ids);
        assert_eq!(highlights_at(&state, &uri, &source, 9, 14), ids);
        for (line, character) in [(1, 17), (2, 17), (8, 10), (9, 6), (9, 18)] {
            assert!(highlights_at(&state, &uri, &source, line, character).is_empty());
        }
        assert_eq!(
            serde_json::to_value(CodeLensService::get_lenses(&state, &source, &uri)).unwrap(),
            json!([
                {"range":{"start":{"line":1,"character":0},"end":{"line":1,"character":0}},"command":{"title":"1 template/style reference","command":"vize.findReferences"}},
                {"range":{"start":{"line":2,"character":0},"end":{"line":2,"character":0}},"command":{"title":"1 template/style reference","command":"vize.findReferences"}},
                {"range":{"start":{"line":3,"character":0},"end":{"line":3,"character":0}},"command":{"title":"3 template/style references","command":"vize.findReferences"}}
            ])
        );
    }
}

#[test]
fn script_and_template_shadow_ids_never_join_the_setup_binding() {
    let source = "<script setup>\nconst id=1\nfunction f(id){ return id }\n{const id=2; use(id)}\n</script>\n<template>\n<div v-for=\"id in [1]\">{{ id }}</div>\n<p>{{ id }}</p>\n</template>";
    let (state, uri) = state_for(source, "file:///Shadow.vue", "vue");
    assert_eq!(
        highlights_at(&state, &uri, source, 1, 7),
        vec![(1, 6, 8, WRITE), (7, 6, 8, READ)]
    );
    assert_eq!(
        highlights_at(&state, &uri, source, 2, 12),
        vec![(2, 11, 13, WRITE), (2, 23, 25, READ)]
    );
    assert_eq!(
        highlights_at(&state, &uri, source, 3, 8),
        vec![(3, 7, 9, WRITE), (3, 17, 19, READ)]
    );
    assert_eq!(
        highlights_at(&state, &uri, source, 6, 27),
        vec![(6, 12, 14, WRITE), (6, 26, 28, READ)]
    );
}

#[test]
fn physical_unicode_css_quoted_reads_exclude_comments_strings_keys_and_selectors() {
    let source = "<script setup>\nconst id=1\nconst label='id'\nconst object={id: 1}\n// id\n</script>\n<template>\n<p title=\"id\">日本語😀 {{ id }} {{ object.id }}</p>\n</template>\n<style>\n#id{ color:v-bind('id'); content:'id'; /* v-bind(id) */ }\n</style>";
    for crlf in [false, true] {
        let source = if crlf {
            source.replace('\n', "\r\n")
        } else {
            source.into()
        };
        let (state, uri) = state_for(&source, "file:///Unicode.vue", "vue");
        assert_eq!(
            highlights_at(&state, &uri, &source, 1, 7),
            vec![(1, 6, 8, WRITE), (7, 23, 25, READ), (10, 19, 21, READ)]
        );
        assert_eq!(
            highlights_at(&state, &uri, &source, 10, 19),
            vec![(1, 6, 8, WRITE), (7, 23, 25, READ), (10, 19, 21, READ)]
        );
        for (line, character) in [
            (2, 14),
            (3, 15),
            (4, 4),
            (7, 11),
            (7, 40),
            (10, 2),
            (10, 35),
            (10, 50),
        ] {
            assert!(highlights_at(&state, &uri, &source, line, character).is_empty());
        }
    }
}

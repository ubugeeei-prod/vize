use std::path::Path;

use crate::batch::{ContentMapperSpan, generate_vue_content_mapper_transform};

const BODY: &str = "const first: number = \"a\"\nconst middle = 1\n\nconst last: boolean = \"c\"\n";

fn setup_spans(source: &str) -> (String, Vec<ContentMapperSpan>, Vec<ContentMapperSpan>) {
    let result = generate_vue_content_mapper_transform(Path::new("App.vue"), source)
        .expect("content mapper transform");
    let start = result
        .text
        .find("// User setup code")
        .expect("setup region");
    let end = start
        + result.text[start..]
            .find("// @vize-map:")
            .expect("setup map comment");
    let inside = result
        .mappings
        .iter()
        .filter(|span| (start..end).contains(&span.0[0]))
        .cloned()
        .collect();
    (result.text.to_string(), inside, result.mappings)
}

#[test]
fn lf_setup_lines_share_one_verbatim_span() {
    let source = format!(
        "<script setup lang=\"ts\">\n{BODY}</script>\n<template><div>{{{{ first }}}}</div></template>\n"
    );
    let (text, spans, _) = setup_spans(&source);
    let [span] = spans.as_slice() else {
        panic!("one setup span: {spans:?}")
    };
    let [at, len, original, original_len, kind, _] = span.0;
    assert_eq!((kind, len), (0, original_len));
    assert_eq!(
        &text[at..at + len],
        &source[original..original + original_len]
    );
    assert_eq!(&source[original..original + original_len], BODY.trim_end());
}

#[test]
fn crlf_setup_lines_stay_one_span_per_line() {
    let body = BODY.replace('\n', "\r\n");
    let source = format!(
        "<script setup lang=\"ts\">\r\n{body}</script>\r\n<template><div>{{{{ first }}}}</div></template>\r\n"
    );
    let (text, spans, _) = setup_spans(&source);
    assert_eq!(spans.len(), 3, "{spans:?}");
    for span in &spans {
        let [at, len, original, original_len, kind, _] = span.0;
        assert_eq!((kind, len), (0, original_len));
        assert_eq!(
            &text[at..at + len],
            &source[original..original + original_len]
        );
    }
}

#[test]
fn shared_originals_keep_disjoint_or_equal_pieces() {
    let source = "<script setup lang=\"ts\">\nconst props = defineProps<{\n  modelValue: boolean\n  label: string\n}>()\n</script>\n<template><div>{{ props.label }}</div></template>\n";
    let (_, spans, all) = setup_spans(source);
    let shared_key = source.find("modelValue").expect("shared prop key");
    let shared_key_len = "modelValue".len();
    assert!(
        all.iter()
            .filter(|span| (span.0[2], span.0[3]) == (shared_key, shared_key_len))
            .count()
            > 1,
        "the prop key must have multiple owners: {all:?}"
    );
    assert!(
        spans
            .iter()
            .any(|span| (span.0[2], span.0[3]) == (shared_key, shared_key_len)),
        "the shared prop key must retain its exact setup span: {spans:?}"
    );
    assert!(
        spans.len() > 1,
        "a shared prop key keeps its line split: {spans:?}"
    );
    for (index, left) in all.iter().enumerate() {
        for right in &all[index + 1..] {
            let (left, right) = (left.0, right.0);
            let overlap = left[2] < right[2] + right[3] && right[2] < left[2] + left[3];
            assert!(
                !overlap || (left[2], left[3]) == (right[2], right[3]),
                "{left:?} partially overlaps {right:?}"
            );
        }
    }
}

#[test]
fn only_a_line_break_joins_verbatim_pieces() {
    let verbatim = |at: usize, len: usize| ContentMapperSpan([at, len, at, len, 0, 7]);
    let text = "{ name }\nnext line";
    // `{ `, `name`, ` }` touch or share one space: each keeps its exact span.
    let pieces = vec![verbatim(0, 2), verbatim(2, 4), verbatim(6, 2)];
    assert_eq!(
        super::join_verbatim_runs(text, text, pieces.clone()),
        pieces
    );
    let spaced = vec![verbatim(0, 1), verbatim(2, 4)];
    assert_eq!(
        super::join_verbatim_runs(text, text, spaced.clone()),
        spaced
    );
    // Across the shared `\n`, the two lines map as one run at the same offsets.
    let lines = vec![verbatim(0, 8), verbatim(9, 9)];
    assert_eq!(
        super::join_verbatim_runs(text, text, lines),
        vec![verbatim(0, 18)]
    );
}

#[test]
fn unsafe_join_guards_keep_verbatim_pieces_split() {
    let verbatim = |at: usize, len: usize, original: usize, features: usize| {
        ContentMapperSpan([at, len, original, len, 0, features])
    };
    let text = "ab\ncd";

    let mapped_inside = vec![
        verbatim(0, 2, 0, 7),
        verbatim(3, 2, 3, 7),
        ContentMapperSpan([5, 0, 2, 1, 1, 0]),
    ];
    assert_eq!(
        super::join_verbatim_runs(text, text, mapped_inside.clone()),
        mapped_inside
    );

    let feature_mismatch = vec![verbatim(0, 2, 0, 7), verbatim(3, 2, 3, 0)];
    assert_eq!(
        super::join_verbatim_runs(text, text, feature_mismatch.clone()),
        feature_mismatch
    );

    let wide_features = vec![verbatim(0, 2, 0, 1_048_575), verbatim(3, 2, 3, 1_048_575)];
    assert_eq!(
        super::join_verbatim_runs(text, text, wide_features),
        vec![verbatim(0, 5, 0, 1_048_575)]
    );

    let gap_mismatch = vec![verbatim(0, 2, 0, 7), verbatim(3, 2, 3, 7)];
    assert_eq!(
        super::join_verbatim_runs(text, "ab\tcd", gap_mismatch.clone()),
        gap_mismatch
    );

    let tab_gap = vec![verbatim(0, 2, 0, 7), verbatim(3, 2, 3, 7)];
    assert_eq!(
        super::join_verbatim_runs("ab\tcd", "ab\tcd", tab_gap.clone()),
        tab_gap
    );

    let crlf_gap = vec![verbatim(0, 2, 0, 7), verbatim(4, 2, 4, 7)];
    assert_eq!(
        super::join_verbatim_runs("ab\r\ncd", "ab\r\ncd", crlf_gap.clone()),
        crlf_gap
    );

    let shared_original = vec![
        verbatim(0, 2, 0, 7),
        verbatim(3, 2, 3, 7),
        verbatim(6, 2, 0, 7),
    ];
    assert_eq!(
        super::join_verbatim_runs("ab\ncd\nab", "ab\ncd\nab", shared_original.clone()),
        shared_original
    );
}

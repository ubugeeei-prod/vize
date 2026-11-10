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

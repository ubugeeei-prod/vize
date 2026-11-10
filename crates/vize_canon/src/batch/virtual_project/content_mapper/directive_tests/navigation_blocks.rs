//! A prop-navigation block and the gaps it touches form one ignored interval.

use super::super::protocol::{
    ContentMapperDiagnosticDirective as Row, ContentMapperDiagnosticDirectives as Table,
    ContentMapperSpan as Span, DIRECTIVE_POLICY_EXPECT, DIRECTIVE_POLICY_IGNORE,
};
use super::super::unmapped_ignores::unmapped_ignores;
use super::published;

const TEXT: &str = "void Child;\n  const __vize_props_nav_0 = undefined as unknown as P & globalThis.Record<string, unknown>;\n  // @ts-ignore Navigation-only references; no diagnostic owner.\n  void __vize_props_nav_0.title; void __vize_props_nav_0.count; void __vize_props_nav_0['data-x'];\nconst after = 1;\n";
const KEYS: [&str; 5] = ["Child", "title", "count", "data-x", "after"];

fn spans(text: &str) -> Vec<Span> {
    KEYS.iter()
        .map(|key| Span([text.find(key).unwrap(), key.len(), 0, key.len(), 0, 0]))
        .collect()
}

fn rows(text: &str, directives: Vec<Row>) -> Vec<[usize; 6]> {
    let table = Table {
        directives,
        unused_expect_directive_diagnostics: Vec::new(),
    };
    unmapped_ignores(Some(table), &spans(text), text)
        .unwrap()
        .directives
        .into_iter()
        .map(|row| row.0)
        .collect()
}

#[test]
fn one_ignore_covers_the_navigation_block_and_its_gaps() {
    let child_end = TEXT.find("Child").unwrap() + 5;
    let after = TEXT.find("after").unwrap();
    assert_eq!(
        rows(TEXT, Vec::new()),
        [
            [0, 0, 0, 5, DIRECTIVE_POLICY_IGNORE, 0],
            [0, 0, child_end, after, DIRECTIVE_POLICY_IGNORE, 0],
            [0, 0, after + 5, TEXT.len(), DIRECTIVE_POLICY_IGNORE, 0],
        ]
    );
}

#[test]
fn an_explicit_tuple_touching_the_block_keeps_every_gap() {
    let title = TEXT.find("title").unwrap();
    let expect = Row([1, 2, title, title + 5, DIRECTIVE_POLICY_EXPECT, 0]);
    let plain = TEXT.replace("__vize_props_nav_", "__vize_propz_nav_");
    let plain_title = plain.find("title").unwrap();
    let touched = rows(TEXT, vec![expect]);
    // Five spans leave six gaps; the expect keeps its own interval.
    assert_eq!(touched.len(), 7);
    assert_eq!(touched.iter().filter(|row| **row == expect.0).count(), 1);
    let lengths =
        |rows: Vec<[usize; 6]>| rows.iter().map(|row| row[3] - row[2]).collect::<Vec<_>>();
    assert_eq!(
        lengths(touched),
        lengths(rows(
            &plain,
            vec![Row([
                1,
                2,
                plain_title,
                plain_title + 5,
                DIRECTIVE_POLICY_EXPECT,
                0
            ])]
        ))
    );
}

#[test]
fn only_a_numbered_const_whose_ignored_line_holds_only_its_references_opens_a_block() {
    for text in [
        TEXT.replace("__vize_props_nav_0 =", "__vize_props_nav_x ="),
        TEXT.replace(
            "void __vize_props_nav_0.count",
            "void __vize_props_nav_1.count",
        ),
        TEXT.replace("; void __vize_props_nav_0.count", "; void other.count"),
        TEXT.replace("// @ts-ignore Navigation-only", "// Navigation-only"),
        TEXT.replace(
            "  const __vize_props_nav_0",
            "  x; const __vize_props_nav_0",
        ),
    ] {
        assert_eq!(rows(&text, Vec::new()).len(), 6, "{text}");
    }
}

#[test]
fn a_reference_line_the_comment_does_not_cover_keeps_its_gaps() {
    let text = TEXT.replace(
        "; void __vize_props_nav_0.count",
        ";\n  void __vize_props_nav_0.count",
    );
    let rows = rows(&text, Vec::new());
    let covered = |key: &str| {
        let at = text.find(key).unwrap();
        rows.iter()
            .any(|row| row[2] < at + key.len() && at < row[3])
    };
    assert!(covered("title"), "{rows:?}");
    assert!(!covered("count") && !covered("data-x"), "{rows:?}");
}

#[test]
fn published_parent_ignores_each_navigation_block_once() {
    let source = "<script setup lang=\"ts\">\nimport Child from './Child.vue'\nconst n = 1\n</script>\n<template>\n  <Child :a=\"n\" :b=\"n\" data-x=\"y\" />\n</template>\n";
    let result = published(source);
    let text = result.text.as_str();
    let start = text
        .find("const __vize_props_nav_0 = ")
        .expect("navigation block");
    let references = text[start..].split_inclusive('\n').nth(2).unwrap();
    assert_eq!(
        references.trim_start(),
        "void __vize_props_nav_0.a; void __vize_props_nav_0.b; void __vize_props_nav_0.dataX;\n",
        "{text}"
    );
    let end = start
        + text[start..]
            .split_inclusive('\n')
            .take(3)
            .map(str::len)
            .sum::<usize>()
        - 1;
    let ignores = result
        .diagnostic_directives
        .expect("directives")
        .directives
        .into_iter()
        .filter(|row| row.0[4] == DIRECTIVE_POLICY_IGNORE && row.0[2] < end && start < row.0[3])
        .map(|row| (row.0[2], row.0[3]))
        .collect::<Vec<_>>();
    assert_eq!(ignores.len(), 1, "{ignores:?}\n{text}");
    assert!(
        ignores[0].0 <= start && end <= ignores[0].1,
        "{ignores:?}\n{text}"
    );
}

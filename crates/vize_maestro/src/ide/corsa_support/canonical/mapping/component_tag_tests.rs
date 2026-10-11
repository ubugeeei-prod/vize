//! Producer tag anchors cannot shadow native queries in authored `:is` values.

use crate::ide::DiagnosticService;
use serde::Deserialize;
use std::ops::Range;
use tower_lsp::lsp_types::Url;
use vize_carton::String;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    name: String,
    script: String,
    template: String,
    template_offset: usize,
    tag_range: Range<usize>,
    query: Option<Query>,
    #[serde(default)]
    opaque: bool,
}

#[derive(Deserialize)]
struct Query {
    offset: usize,
    token: String,
}

#[test]
fn component_tag_anchors_preserve_their_complete_authored_ranges_and_native_value_queries() {
    let cases: Vec<Case> = serde_json::from_str(include_str!(
        "../../../../../../../tests/_fixtures/differential/lsp/component-tag-source-ranges/cases.json"
    ))
    .unwrap();
    for case in cases {
        let prefix = format!(
            "<script setup lang=\"ts\">\n{}\n</script>\n<template>",
            case.script
        );
        assert!(prefix.len() <= case.template_offset);
        let source = format!(
            "{prefix}{}{}</template>",
            " ".repeat(case.template_offset - prefix.len()),
            case.template
        );
        let uri = Url::parse("file:///workspace/Component.vue").unwrap();
        let result = DiagnosticService::generate_virtual_ts(&uri, &source, false, false)
            .expect("actual native projection");
        let tag_rows = result
            .source_mappings
            .iter()
            .filter(|row| row.src_range.start == case.tag_range.start)
            .collect::<Vec<_>>();
        if case.opaque {
            assert!(tag_rows.is_empty(), "{} is authored opaque text", case.name);
            assert!(!result.code.contains("__vize_dynamic_is_0"));
            continue;
        }
        assert!(
            tag_rows.len() >= 3,
            "{} retains navigation, props and slot tag anchors: {tag_rows:?}",
            case.name
        );
        for row in tag_rows {
            assert_eq!(row.src_range, case.tag_range, "{}: {row:?}", case.name);
        }
        if let Some(query) = case.query {
            assert_eq!(
                source.get(query.offset..query.offset + query.token.len()),
                Some(query.token.as_str()),
                "{} authored query remains at its original first byte",
                case.name
            );
            let generated = super::source_offset_to_virtual_generated_offset(&result, query.offset)
                .expect("native value query");
            assert_eq!(
                result.code.get(generated..generated + query.token.len()),
                Some(query.token.as_str()),
                "{} native query must address the copied expression",
                case.name
            );
        }
    }
}

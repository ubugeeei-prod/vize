//! Bounded malformed-generic recovery retains independent authored TypeScript.
#![expect(clippy::expect_used, reason = "complete authored mapping laws")]
#![expect(clippy::unwrap_used, reason = "complete authored mapping laws")]

use std::{ops::Range, path::Path};

use serde_json::json;
use vize_carton::{String, cstr};

use crate::batch::{ContentMapperTransform, generate_vue_content_mapper_transform};

const TYPE_IMPORT: &str = "import type { External } from './types';";
const VALUE_IMPORT: &str = "import { consume } from './runtime';";
const BODY: &str = "const independent: number = 'bad';\nconst value: External = consume();";
const RAW: &str = "T extends";
const REFUSAL: &str = "Cannot safely project this malformed generic attribute";

fn transform(source: &str) -> ContentMapperTransform {
    generate_vue_content_mapper_transform(Path::new("src/App.vue"), source).unwrap()
}

/// Require one complete authored chunk, with every byte mapped verbatim once.
fn exact_mapped_chunk(source: &str, result: &ContentMapperTransform, chunk: &str) -> Range<usize> {
    let authored: Vec<_> = source.match_indices(chunk).map(|(at, _)| at).collect();
    assert_eq!(authored.len(), 1, "the complete authored chunk is unique");
    let start = *authored.first().unwrap();
    let original = start..start + chunk.len();
    let mut fragments = Vec::new();
    for row in &result.mappings {
        let [generated, length, authored, authored_length, kind, _] = row.0;
        let from = original.start.max(authored);
        let end = original.end.min(authored + authored_length);
        if kind != 0 || from >= end {
            continue;
        }
        assert_eq!(length, authored_length);
        let generated = generated + from - authored;
        let generated_range = generated..generated + end - from;
        let bytes = result.text.get(generated_range.clone()).unwrap();
        assert_eq!(Some(bytes), source.get(from..end));
        fragments.push((
            from - original.start,
            end - original.start,
            generated_range,
            bytes,
        ));
    }
    assert_eq!(fragments.len(), 1, "{chunk}: {fragments:?}");
    let (from, end, generated, bytes) = fragments.pop().unwrap();
    assert_eq!((from, end, bytes), (0, chunk.len(), chunk));
    assert_eq!(result.text.match_indices(chunk).count(), 1);
    generated
}

#[test]
fn malformed_generic_recovery_preserves_whole_imports_and_independent_body_mappings() {
    for newline in ["\n", "\r\n"] {
        for prefix in ["", "<!-- 日本 🧭 -->"] {
            for lang in ["ts", "tsx"] {
                let source = cstr!(
                    "{prefix}<script setup lang=\"{lang}\" generic=\"{RAW}\">\n{TYPE_IMPORT}\n{VALUE_IMPORT}\n{BODY}\n</script>\n<template><div /></template>\n"
                ).replace('\n', newline);
                let result = transform(&source);
                assert_eq!(
                    serde_json::to_value(&result.diagnostics).unwrap(),
                    json!([])
                );
                let signature = cstr!("function __setup<{RAW}>() {{");
                let signatures: Vec<_> = result
                    .text
                    .match_indices(signature.as_str())
                    .map(|(at, _)| at)
                    .collect();
                assert_eq!(signatures.len(), 1);
                assert_eq!(
                    result.text.match_indices(cstr!("<{RAW}>").as_str()).count(),
                    1
                );
                let signature = *signatures.first().unwrap();
                let raw_generated = signature + "function __setup<".len();
                let raw_authored =
                    source.find(cstr!("generic=\"{RAW}\"").as_str()).unwrap() + "generic=\"".len();
                let signature_rows: Vec<_> = result
                    .mappings
                    .iter()
                    .map(|row| row.0)
                    .filter(|row| {
                        raw_generated - 1 <= row[0] && row[0] <= raw_generated + RAW.len()
                    })
                    .collect();
                assert_eq!(
                    signature_rows,
                    vec![
                        [raw_generated - 1, 1, raw_authored - 1, 1, 1, 0],
                        [raw_generated, RAW.len(), raw_authored, RAW.len(), 0, 0],
                        [
                            raw_generated + RAW.len(),
                            1,
                            raw_authored + RAW.len(),
                            1,
                            1,
                            0
                        ],
                    ]
                );
                let type_import = exact_mapped_chunk(&source, &result, TYPE_IMPORT);
                let value_import = exact_mapped_chunk(&source, &result, VALUE_IMPORT);
                let body = exact_mapped_chunk(&source, &result, &BODY.replace('\n', newline));
                assert!(type_import.end <= signature);
                assert!(value_import.end <= signature);
                assert!(raw_generated + RAW.len() < body.start);
                let echoes: Vec<_> = result
                    .diagnostic_directives
                    .iter()
                    .flat_map(|directives| &directives.directives)
                    .map(|directive| directive.0)
                    .collect();
                assert_eq!(echoes, Vec::<[usize; 6]>::new());
                for fabricated in [
                    "export type Props<",
                    " = Props<T>",
                    "typeof __setup<T>",
                    "__VizeGenericComponent<T>",
                    "type __VizeGenericContext<",
                ] {
                    assert_eq!(
                        result.text.match_indices(fabricated).count(),
                        0,
                        "{fabricated}"
                    );
                }
            }
        }
    }
}

#[test]
fn unsupported_malformed_generic_shapes_return_one_complete_authored_refusal() {
    for newline in ["\n", "\r\n"] {
        for prefix in ["", "<!-- 日本 🧭 -->"] {
            for (plain, generic, body) in [
                ("", RAW, "namespace Local { export const value = 1; }"),
                ("", RAW, "export const leaked = 1;"),
                ("", RAW, "const text = 'unterminated;"),
                (
                    "<script lang=\"ts\">export const shared = 1;</script>\n",
                    RAW,
                    BODY,
                ),
                ("", "T>() {} function other<U", BODY),
            ] {
                let source = cstr!(
                    "{prefix}{plain}<script setup lang=\"ts\" generic=\"{generic}\">\n{body}\n</script>\n<template><div /></template>\n"
                ).replace('\n', newline);
                let result = transform(&source);
                let start = source
                    .find(cstr!("generic=\"{generic}\"").as_str())
                    .unwrap()
                    + "generic=\"".len();
                assert_eq!(
                    serde_json::to_value(&result.diagnostics).unwrap(),
                    json!([{
                        "messageText":REFUSAL,"start":start,"length":1,"code":100002,
                    }])
                );
                assert_eq!(
                    result.text,
                    String::from(
                        "declare const __vize_component: any;\nexport default __vize_component;\n"
                    )
                );
                assert_eq!(result.mappings, Vec::new());
                assert_eq!(result.semantic_links, Vec::new());
                assert_eq!(result.diagnostic_directives, None);
            }
        }
    }
}

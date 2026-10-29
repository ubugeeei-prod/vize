#![expect(clippy::unwrap_used, reason = "exact authored projection laws")]

use super::{InvalidReferencePath, emit, validate_reference_paths};
use vize_carton::String;

#[test]
fn complete_directives_preserve_literal_target_names_and_deduplicate() {
    let paths = [
        "",
        "/tmp/雪&<>/types.d.ts",
        "/tmp/雪&<>/types.d.ts",
        "/tmp/it's/types.d.ts",
        "/tmp/a\"b/types.d.ts",
        "./relative&types.d.ts",
    ]
    .map(String::from);
    let mut code = String::default();
    emit(&mut code, &paths);
    assert_eq!(
        code,
        concat!(
            "/// <reference path=\"/tmp/雪&<>/types.d.ts\" />\n",
            "/// <reference path=\"/tmp/it's/types.d.ts\" />\n",
            "/// <reference path='/tmp/a\"b/types.d.ts' />\n",
            "/// <reference path=\"./relative&types.d.ts\" />\n",
        )
    );
    assert_eq!(validate_reference_paths(&paths), Ok(()));
}

#[test]
fn unrepresentable_paths_report_the_exact_value_and_reason() {
    for (path, reason) in [
        (
            "a\nb.d.ts",
            "line breaks and NUL are not representable in a TypeScript reference path",
        ),
        (
            "a\rb.d.ts",
            "line breaks and NUL are not representable in a TypeScript reference path",
        ),
        (
            "a\0b.d.ts",
            "line breaks and NUL are not representable in a TypeScript reference path",
        ),
        (
            "a\u{2028}b.d.ts",
            "line breaks and NUL are not representable in a TypeScript reference path",
        ),
        (
            "a\u{2029}b.d.ts",
            "line breaks and NUL are not representable in a TypeScript reference path",
        ),
        (
            "a'\"b.d.ts",
            "a TypeScript reference path cannot contain both quote delimiters",
        ),
    ] {
        let paths = [String::from(path)];
        assert_eq!(
            validate_reference_paths(&paths),
            Err(InvalidReferencePath { path, reason })
        );
        let mut code = String::default();
        emit(&mut code, &paths);
        assert_eq!(code, "");
    }
}

#[test]
fn document_projection_rejects_unrepresentable_paths_with_a_configuration_error() {
    use crate::batch::{ImportRewriter, generate_vue_document_virtual_ts};
    use crate::virtual_ts::VirtualTsOptions;
    use std::path::Path;

    for (path, reason) in [
        (
            "types\nb.d.ts",
            "line breaks and NUL are not representable in a TypeScript reference path",
        ),
        (
            "types\rb.d.ts",
            "line breaks and NUL are not representable in a TypeScript reference path",
        ),
        (
            "types\0b.d.ts",
            "line breaks and NUL are not representable in a TypeScript reference path",
        ),
        (
            "types\u{2028}b.d.ts",
            "line breaks and NUL are not representable in a TypeScript reference path",
        ),
        (
            "types\u{2029}b.d.ts",
            "line breaks and NUL are not representable in a TypeScript reference path",
        ),
        (
            "types'\"b.d.ts",
            "a TypeScript reference path cannot contain both quote delimiters",
        ),
    ] {
        let options = VirtualTsOptions {
            reference_paths: vec![String::from(path)],
            ..Default::default()
        };
        let result = generate_vue_document_virtual_ts(
            Path::new("App.vue"),
            "<script setup lang=\"ts\">const value = 1;</script><template>{{ value }}</template>",
            &options,
            &ImportRewriter::new(),
            false,
        );
        let error = result.err().unwrap();
        assert_eq!(
            error.to_string(),
            vize_carton::cstr!(
                "Configuration error: Invalid declaration reference path {path:?}: {reason}"
            )
        );
    }
}

#[test]
fn literal_reference_headers_preserve_complete_authored_token_ranges() {
    use crate::batch::{ImportRewriter, generate_vue_document_virtual_ts};
    use crate::virtual_ts::VirtualTsOptions;
    use std::path::Path;

    let source = "<script setup lang=\"ts\">\nconst authoredMarker: number = 1;\n</script>\n<template>{{ authoredMarker }}</template>\n";
    let options = VirtualTsOptions {
        reference_paths: vec![String::from("/tmp/a\"b&雪/types.d.ts")],
        ..Default::default()
    };
    let output = generate_vue_document_virtual_ts(
        Path::new("App.vue"),
        source,
        &options,
        &ImportRewriter::new(),
        false,
    )
    .unwrap();
    let authored = source.find("authoredMarker").unwrap();
    let generated = output.code.find("const authoredMarker").unwrap() + "const ".len();
    assert_eq!(output.mapping.to_generated(authored), Some(generated));
    assert_eq!(output.mapping.to_authored(generated), Some(authored));
    assert_eq!(
        output
            .mapping
            .diagnostic_range_to_authored(generated, generated + "authoredMarker".len()),
        Some((authored, authored + "authoredMarker".len()))
    );
}

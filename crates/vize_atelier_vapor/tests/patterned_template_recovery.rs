#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "tests assert by panicking"
)]
#![expect(
    clippy::disallowed_macros,
    reason = "test fixtures and insta snapshots use std strings and format"
)]

use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_atelier_ssr::{SsrCompilerOptions, compile_ssr_with_options};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor_with_diagnostics};
use vize_carton::Allocator;

const SUBJECT: &str = "`v-match` requires a subject expression.";
const SHAPE: &str = "`v-match` does not accept directive arguments or modifiers.";
const ORPHAN: &str = "`v-when` branches must be direct children of a `v-match` container.";
const FALLBACK: &str =
    "`v-when=\"_\"` fallback arms must be last and unique within a `v-match` block.";
const CONFLICT: &str =
    "v-when cannot share an element with v-if, v-else-if, v-else, v-for or v-match.";

fn check(source: &str, expected: &[(&str, &str)]) {
    let expected: Vec<_> = expected
        .iter()
        .map(|(message, text)| {
            let start = source.find(text).expect("authored diagnostic target") as u32;
            (*message, start, start + text.len() as u32)
        })
        .collect();
    let allocator = Allocator::default();
    let (_, dom, _) = compile_template_with_options(
        &allocator,
        source,
        DomCompilerOptions {
            experimental_patterned_template: true,
            ..Default::default()
        },
    );
    let (_, ssr, _) = compile_ssr_with_options(
        &allocator,
        source,
        SsrCompilerOptions {
            experimental_patterned_template: true,
            ..Default::default()
        },
    );
    let (vapor, diagnostics) = compile_vapor_with_diagnostics(
        &allocator,
        source,
        VaporCompilerOptions {
            experimental_patterned_template: true,
            ..Default::default()
        },
    );
    assert_eq!(vapor.code, "");
    for errors in [&dom, &ssr, &diagnostics] {
        let actual: Vec<_> = errors
            .iter()
            .map(|error| {
                let loc = error.loc.as_ref().unwrap();
                (error.message.as_str(), loc.span.start, loc.span.end)
            })
            .collect();
        assert_eq!(actual, expected, "{source}");
    }
}

#[test]
fn invalid_headers_do_not_make_their_direct_arms_orphans() {
    for (header, message) in [
        ("v-match", SUBJECT),
        ("v-match=\"\"", SUBJECT),
        ("v-match=\"   \"", SUBJECT),
        ("v-match:arg=\"subject\"", SHAPE),
        ("v-match.once=\"subject\"", SHAPE),
    ] {
        for arms in [
            "",
            r#"<p v-when="'ok'">ok</p><p v-when="_">other</p>"#,
            r#"<p v-case="'ok'">ok</p><p v-case.default>other</p>"#,
        ] {
            let source = format!("<!-- \u{1f600} -->\r\n<template {header}>{arms}</template>");
            check(&source, &[(message, header)]);
        }
    }
}

#[test]
fn invalid_headers_preserve_independent_arm_errors() {
    for header in ["v-match", "v-match:arg=\"subject\""] {
        let header_error = if header == "v-match" { SUBJECT } else { SHAPE };
        let source = format!(
            r#"<template {header}><p v-when="let value"></p><p v-when="'ok'" v-if="show"></p><p v-when="_"></p><p v-when="'late'"></p></template>"#
        );
        check(
            &source,
            &[
                (header_error, header),
                ("Only const pattern bindings are supported.", " value"),
                (CONFLICT, "v-when=\"'ok'\""),
                (FALLBACK, "v-when=\"'late'\""),
            ],
        );
    }
}

#[test]
fn recovery_keeps_nested_errors_and_real_orphans() {
    check(
        r#"<template v-match><p v-when="_"><template v-match.once="nested"><i v-when="_"></i></template></p><section><p v-when="'orphan'"></p></section></template>"#,
        &[
            (SUBJECT, "v-match"),
            (SHAPE, "v-match.once=\"nested\""),
            (ORPHAN, "v-when=\"'orphan'\""),
        ],
    );
}

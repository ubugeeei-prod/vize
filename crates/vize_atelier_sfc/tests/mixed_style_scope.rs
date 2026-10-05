//! Preserve authored block scope in full-SFC adapters (#7856).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    reason = "complete source-built CSS observations cross a child-process oracle"
)]

use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, StyleCompileOptions, compile_sfc, parse_sfc,
};
use vize_l0::String;

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/benchmark-mixed-style-scope/original.vue.txt"
);

#[test]
fn full_sfc_scopes_only_each_authored_style_block() {
    let controls = [
        ("original", ORIGINAL),
        (
            "reordered",
            r#"<template><div class="cascade tail"></div></template><style>.tail{color:green;border:1px solid black}</style><style scoped>.cascade{color:red;color:blue!important;--gap:2px}@media(min-width:1px){.cascade{padding:1px 2px}}</style>"#,
        ),
        (
            "ordinary-only",
            r#"<template><div class="tail"></div></template><style>.tail{color:green;border:1px solid black}</style>"#,
        ),
        (
            "scoped-only",
            r#"<template><div class="cascade"></div></template><style scoped>.cascade{color:red;color:blue!important;--gap:2px}@media(min-width:1px){.cascade{padding:1px 2px}}</style>"#,
        ),
    ];
    let mut observations = Vec::new();
    for (name, source) in controls {
        for backend in ["dom", "ssr", "vapor"] {
            for aggregate_scoped in [false, true] {
                let descriptor =
                    parse_sfc(source, SfcParseOptions::default()).expect("parse original");
                let mut options = SfcCompileOptions {
                    scope_id: Some("abc12345".into()),
                    vapor: backend == "vapor",
                    ..Default::default()
                };
                options.template.ssr = backend == "ssr";
                options.style.scoped = aggregate_scoped;
                let result = compile_sfc(&descriptor, options).expect("compile full SFC");
                assert!(result.errors.is_empty(), "{:?}", result.errors);
                assert!(result.warnings.is_empty(), "{:?}", result.warnings);
                observations.push(serde_json::json!({
                    "name": name,
                    "source": source,
                    "backend": backend,
                    "aggregateScoped": aggregate_scoped,
                    "code": result.code.as_str(),
                    "css": result.css.expect("complete CSS join").as_str(),
                }));
            }
        }
    }
    let mut child = Command::new("node")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/tooling/support/mixed-style-scope.mjs"),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run Vue CSS/DOM oracle");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(
            serde_json::to_string(&observations)
                .expect("complete observations")
                .as_bytes(),
        )
        .expect("write observations");
    let output = child.wait_with_output().expect("oracle results");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn standalone_style_keeps_its_explicit_scoped_option() {
    let source = "<style>.tail{color:green}</style>";
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse ordinary style");
    let block = descriptor.styles.first().expect("original style");
    let css = vize_atelier_sfc::style::compile_style(
        block,
        &StyleCompileOptions {
            id: "data-v-abc12345".into(),
            scoped: true,
            ..Default::default()
        },
    )
    .expect("explicit standalone scoped request");
    assert!(css.contains(".tail[data-v-abc12345]"), "{css}");
}

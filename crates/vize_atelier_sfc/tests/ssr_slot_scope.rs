//! #6898: own :slotted() metadata, whole modules and real raw SSR HTML.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    reason = "fixtures retain complete compiler/runtime evidence"
)]

use serde_json::json;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcCompileResult, SfcParseOptions, TemplateCompileOptions, compile_sfc,
    parse_sfc,
};

macro_rules! fixture {
    ($name:literal) => {
        include_str!(concat!(
            "../../../tests/fixtures/sfc/ssr-slot-scope/",
            $name
        ))
    };
}

const CASES: &[(&str, &str, &str)] = &[
    (
        "forwarded-fallback-attrs",
        fixture!("forwarded-fallback-attrs.vue"),
        fixture!("forwarded-fallback-attrs.expected.txt"),
    ),
    (
        "forwarded-fallback-plain",
        fixture!("forwarded-fallback-plain.vue"),
        fixture!("forwarded-fallback.expected.txt"),
    ),
    (
        "forwarded-fallback-slotted",
        fixture!("forwarded-fallback-slotted.vue"),
        fixture!("forwarded-fallback-slotted.expected.txt"),
    ),
    (
        "no-style",
        fixture!("no-style.vue"),
        fixture!("unscoped.expected.txt"),
    ),
    (
        "plain-scoped",
        fixture!("plain-scoped.vue"),
        fixture!("scoped.expected.txt"),
    ),
    (
        "slotted",
        fixture!("slotted.vue"),
        fixture!("slotted.expected.txt"),
    ),
    (
        "legacy-slotted",
        fixture!("legacy-slotted.vue"),
        fixture!("slotted.expected.txt"),
    ),
    (
        "unscoped-slotted",
        fixture!("unscoped-slotted.vue"),
        fixture!("unscoped.expected.txt"),
    ),
    (
        "mixed-unscoped",
        fixture!("mixed-unscoped.vue"),
        fixture!("scoped.expected.txt"),
    ),
    (
        "second-scoped",
        fixture!("second-scoped.vue"),
        fixture!("slotted.expected.txt"),
    ),
    (
        "forwarded-plain",
        fixture!("forwarded-plain.vue"),
        fixture!("forwarded.expected.txt"),
    ),
    (
        "forwarded-slotted",
        fixture!("forwarded-slotted.vue"),
        fixture!("forwarded-slotted.expected.txt"),
    ),
];

fn compile(source: &str, filename: &str, scope_id: &str) -> (SfcCompileResult, bool) {
    let parse = SfcParseOptions {
        filename: filename.into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("parse authored SFC");
    let scoped = descriptor.styles.iter().any(|style| style.scoped);
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            parse,
            scope_id: Some(scope_id.into()),
            template: TemplateCompileOptions {
                ssr: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("compile actual SFC SSR module");
    assert!(result.errors.is_empty(), "{filename}: {:?}", result.errors);
    assert!(
        result.warnings.is_empty(),
        "{filename}: {:?}",
        result.warnings
    );
    (result, scoped)
}

#[test]
fn own_slotted_styles_preserve_complete_modules_and_raw_vue_html() {
    let (page, _) = compile(fixture!("page.vue"), "Page.vue", "page");
    let cases = CASES
        .iter()
        .map(|(name, source, expected)| {
            let (result, scoped) = compile(source, "Layout.vue", "layout");
            assert_eq!(result.code.as_str(), *expected, "whole module: {name}");
            json!({ "name": name, "source": source, "code": result.code,
            "hasScoped": scoped, "compileResult": result,
            "incomingScope": matches!(*name, "forwarded-fallback-plain"
                | "forwarded-fallback-slotted" | "forwarded-fallback-attrs") })
        })
        .collect::<Vec<_>>();
    let input = json!({ "check": true, "pageSource": fixture!("page.vue"),
        "pageCode": page.code, "pageCompileResult": page, "cases": cases });
    let mut child = Command::new("node")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/tooling/support/ssr-slot-scope.mjs"),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run real Vue SSR oracle");
    child
        .stdin
        .take()
        .expect("oracle stdin")
        .write_all(input.to_string().as_bytes())
        .expect("write real compiler outputs");
    let output = child.wait_with_output().expect("oracle completed");
    assert!(
        output.status.success(),
        "{}\n{}",
        std::string::String::from_utf8_lossy(&output.stderr),
        std::string::String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(output.stderr, b"");
    let result: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("full runtime receipt");
    assert_eq!(
        result["observations"]
            .as_array()
            .expect("observations")
            .len(),
        CASES.len()
    );
    // --nocapture retains the complete generated modules, compiler facets,
    // raw HTML and warnings in the actual source-built gate log.
    println!("{}", input);
    println!("{}", result);
}

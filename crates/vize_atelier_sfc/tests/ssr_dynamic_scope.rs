//! #6880: preserve the whole Vue Fes heading and raw eight-case SSR oracle.
#![expect(
    clippy::disallowed_types,
    reason = "complete original compiler/runtime observations"
)]

use serde_json::json;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc, parse_sfc,
};

#[test]
fn original_vuefes_heading_keeps_exactly_one_dynamic_ssr_scope() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/compiler/vuefes-heading-cascade/VFHeading.vue.txt"
    );
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/compiler/vuefes-heading-cascade/scope.expected.ssr.json"
    )).expect("whole hosted official SSR object");
    let parse = SfcParseOptions {
        filename: "VFHeading.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("whole original SFC");
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            parse,
            scope_id: Some("ea49ded1".into()),
            template: TemplateCompileOptions {
                ssr: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("compile the actual native SSR module");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    let input = json!({ "source": source, "compileResult": result, "expected": expected,
        "scopeId": "data-v-ea49ded1", "hasScoped": descriptor.styles.iter().any(|style| style.scoped) });
    let mut child = Command::new("node")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/tooling/support/ssr-dynamic-scope.mjs"),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run the declared stable Vue SSR runtime");
    child
        .stdin
        .take()
        .expect("runtime stdin")
        .write_all(&serde_json::to_vec(&input).expect("complete compiler packet"))
        .expect("write complete compiler result");
    let output = child.wait_with_output().expect("runtime completed");
    assert!(
        output.status.success(),
        "{}\n{}",
        std::string::String::from_utf8_lossy(&output.stderr),
        std::string::String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(output.stderr, b"");
    let observed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("whole raw SSR object");
    assert_eq!(observed["ssr"], expected);
    println!("{input}");
    println!("{observed}");
}

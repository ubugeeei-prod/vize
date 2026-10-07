//! #7970 preserves actual Vue 1 raw coercion rather than widening Vue 3 merging.
#![cfg(feature = "legacy")]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "complete authored compatibility corpus and actual runtime transport"
)]

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::{
    CodegenMode, CodegenOptions, ParserOptions, TransformOptions, codegen, parser, transform,
};
use vize_l0::{Allocator, config::VueVersion};

#[test]
fn raw_slot_numeric_values_keep_the_original_implicit_and_explicit_coercion() {
    let rows: Vec<Value> = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/compiler/component-slot-text-7970/raw-cases.json"
    ))
    .expect("whole authored raw controls");
    let mut observations = Vec::new();
    for row in rows {
        let source = row["source"].as_str().expect("whole source");
        let allocator = Allocator::new();
        let (mut root, errors) = parser::parse_with_options(
            &allocator,
            source,
            ParserOptions {
                dialect: VueVersion::V1,
                is_native_tag: Some(vize_l0::is_native_tag),
                ..Default::default()
            },
        );
        assert!(errors.is_empty(), "{errors:?}");
        assert!(
            transform(
                &allocator,
                &mut root,
                TransformOptions {
                    dialect: VueVersion::V1,
                    prefix_identifiers: true,
                    hoist_static: false,
                    ..Default::default()
                },
                None
            )
            .is_empty()
        );
        let options = CodegenOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: true,
            ..Default::default()
        };
        let plain = codegen::generate(&root, options.clone());
        let mapped = codegen::generate(
            &root,
            CodegenOptions {
                source_map: true,
                ..options
            },
        );
        assert_eq!(plain.code, mapped.code);
        assert_eq!(plain.preamble, mapped.preamble);
        assert!(plain.map.is_none());
        assert!(mapped.map.is_some());
        let code = [mapped.preamble.as_str(), "\n", mapped.code.as_str()].concat();
        observations.push(json!({ "case": row, "code": code, "map": mapped.map,
            "diagnostics": [] }));
    }
    assert_eq!(observations.len(), 5);
    let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support/slot-text-raw-7970.mjs");
    let mut child = Command::new("node")
        .arg(runner)
        .env("NODE_ENV", "production")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("real pinned Vue runtime");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(&serde_json::to_vec(&observations).expect("whole modules"))
        .expect("complete write");
    let output = child.wait_with_output().expect("runtime exits");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("complete receipt");
    assert_eq!(receipt["qualified"], 5);
    assert_eq!(receipt["nativeHandled"], 0);
}

//! #7970: retained and selected text runs plus the entire original SFCs.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "regression corpus retains complete source and runtime observations"
)]

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::{
    CodegenMode, CodegenOptions, ParserOptions, TransformOptions, WhitespaceStrategy,
    codegen::{CodegenResult, generate},
    parser::{parse_with_options, with_whitespace_strategy},
    transform,
};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcCompileResult, SfcScriptOutputMode, TemplateCompileOptions,
    compile_sfc_for_adapter, parse_sfc,
};
use vize_l0::Allocator;
use vize_l1_to_l2::{DomEmitMode, DomEmitOptions, LegacyCaps, emit_dom_source_with_options};

const CASES: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/component-slot-text-7970/cases.json"
);
const APP: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/component-slot-text-7970/App.vue.txt"
);
const CHILD: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/component-slot-text-7970/MyTitle.vue.txt"
);

fn options(comments: bool, map: bool) -> DomCompilerOptions {
    DomCompilerOptions {
        mode: CodegenMode::Module,
        prefix_identifiers: true,
        hoist_static: false,
        comments,
        source_map: map,
        ..Default::default()
    }
}

fn retained(source: &str, comments: bool, map: bool) -> CodegenResult {
    let allocator = Allocator::new();
    let (mut root, errors) = parse_with_options(
        &allocator,
        source,
        ParserOptions {
            is_void_tag: vize_l0::is_void_tag,
            is_native_tag: Some(vize_l0::is_native_tag),
            comments,
            ..Default::default()
        },
    );
    assert!(errors.is_empty(), "{source}: {errors:?}");
    assert!(
        transform(
            &allocator,
            &mut root,
            TransformOptions {
                prefix_identifiers: true,
                hoist_static: false,
                ..Default::default()
            },
            None
        )
        .is_empty()
    );
    generate(
        &root,
        CodegenOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: true,
            source_map: map,
            ..Default::default()
        },
    )
}

fn sfc(source: &str, comments: bool) -> SfcCompileResult {
    let descriptor = parse_sfc(source, Default::default()).expect("original SFC parses");
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions {
            template: TemplateCompileOptions {
                compiler_options: Some(options(comments, false)),
                ..Default::default()
            },
            ..Default::default()
        },
        Default::default(),
        Default::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
    )
    .expect("complete original SFC compiles");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    assert!(result.css.is_none());
    assert!(result.map.is_none());
    assert!(result.macro_artifacts.is_empty());
    result
}

#[test]
fn selected_and_retained_slot_runs_agree_in_every_complete_module() {
    let fixtures: Vec<Value> = serde_json::from_str(CASES).expect("whole authored cases");
    let mut compared = 0;
    for comments in [false, true] {
        for fixture in &fixtures {
            let source = fixture["template"].as_str().expect("whole template");
            let old = retained(source, comments, false);
            let mapped = retained(source, comments, true);
            assert_eq!(old.code, mapped.code);
            assert_eq!(old.preamble, mapped.preamble);
            assert!(old.map.is_none());
            assert!(mapped.map.is_some());
            let allocator = Allocator::new();
            let native = emit_dom_source_with_options(
                &allocator,
                source,
                LegacyCaps::default(),
                &DomEmitOptions {
                    mode: DomEmitMode::Module,
                    prefix_identifiers: true,
                    hoist_static: false,
                    comments,
                    ..Default::default()
                },
            )
            .expect("existing selected surface remains admitted");
            assert_eq!(
                native.assembled(),
                [old.preamble.as_str(), "\n", old.code.as_str()].concat(),
                "{} / comments={comments}",
                fixture["name"]
            );
            compared += 1;
        }
    }
    assert_eq!(compared, 32);
}

#[test]
fn original_sfc_and_all_slot_controls_keep_whole_dom_through_updates() {
    let fixtures: Vec<Value> = serde_json::from_str(CASES).expect("whole authored cases");
    let mut cases = Vec::new();
    let mut sfcs = Vec::new();
    for (whitespace, strategy) in [
        ("condense", WhitespaceStrategy::Condense),
        ("preserve", WhitespaceStrategy::Preserve),
    ] {
        for comments in [false, true] {
            with_whitespace_strategy(strategy, || {
                for fixture in &fixtures {
                    let source = fixture["template"].as_str().expect("whole template");
                    let old = retained(source, comments, true);
                    let allocator = Allocator::new();
                    let (_, errors, current) =
                        compile_template_with_options(&allocator, source, options(comments, false));
                    assert!(errors.is_empty(), "{source}: {errors:?}");
                    assert_eq!(current.code, old.code);
                    assert_eq!(current.preamble, old.preamble);
                    let current_code =
                        [current.preamble.as_str(), "\n", current.code.as_str()].concat();
                    let retained_code = [old.preamble.as_str(), "\n", old.code.as_str()].concat();
                    cases.push(json!({ "name": fixture["name"], "source": source,
                        "whitespace": whitespace, "comments": comments,
                        "current": { "code": current_code,
                            "errors": [], "map": current.map },
                        "retained": { "code": retained_code, "map": old.map } }));
                }
                sfcs.push(json!({ "whitespace": whitespace, "comments": comments,
                    "appSource": APP, "childSource": CHILD,
                    "app": sfc(APP, comments), "child": sfc(CHILD, comments) }));
            });
        }
    }
    assert_eq!(cases.len(), 64);
    assert_eq!(sfcs.len(), 4);
    let input = json!({ "cases": cases, "sfcs": sfcs });
    let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support/component-slot-text-7970.mjs");
    let mut child = Command::new("node")
        .arg(runner)
        .env("NODE_ENV", "production")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("independent pinned Vue runtime");
    child
        .stdin
        .take()
        .expect("oracle stdin")
        .write_all(&serde_json::to_vec(&input).expect("complete observation bytes"))
        .expect("complete source modules");
    let output = child.wait_with_output().expect("runtime exits");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("whole runtime receipt");
    assert_eq!(receipt["qualifiedCases"], 64);
    assert_eq!(receipt["qualifiedOriginalSfcs"], 4);
    assert_eq!(
        receipt["nativeHandled"], 0,
        "SFC legacy adapter receives no native completion credit"
    );
}

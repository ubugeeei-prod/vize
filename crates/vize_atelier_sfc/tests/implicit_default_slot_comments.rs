//! #7822: complete public results and real Vue DOM/SSR/Vapor observations.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::unwrap_used,
    reason = "regression fixtures serialize complete public results and panic on failure"
)]

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::CodegenOptions;
use vize_atelier_dom::DomCompilerOptions;
use vize_atelier_sfc::{
    SfcCompileOptions, SfcCompileResult, SfcScriptOutputMode, TemplateCompileOptions,
    compile_sfc_for_adapter, parse_sfc,
};

const REPORTED: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/implicit-default-slot-comments/Reported.vue.txt"
);
const CASES: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/implicit-default-slot-comments/cases.json"
);

fn compile(source: &str, target: &str) -> SfcCompileResult {
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions {
            vapor: matches!(target, "vapor" | "vapor-ssr-fallback"),
            template: TemplateCompileOptions {
                ssr: matches!(target, "ssr" | "vapor-ssr-fallback"),
                compiler_options: Some(DomCompilerOptions {
                    comments: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        },
        Default::default(),
        Default::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
    )
    .unwrap();
    assert!(result.errors.is_empty(), "{target}: {:?}", result.errors);
    assert!(result.css.is_none());
    assert!(result.map.is_none());
    assert!(result.bindings.is_none());
    assert!(result.macro_artifacts.is_empty());
    if target == "vapor-ssr-fallback" {
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(
            result.warnings.first().unwrap().code.as_deref(),
            Some("VAPOR_SSR_FALLBACK")
        );
        assert_eq!(
            result.warnings.first().unwrap().message,
            "SFC Vapor SSR is not supported yet; falling back to standard SSR output."
        );
        assert_eq!(
            serde_json::to_value(&result.warnings.first().unwrap().loc).unwrap(),
            serde_json::to_value(&descriptor.template.as_ref().unwrap().loc).unwrap()
        );
    } else {
        assert!(
            result.warnings.is_empty(),
            "{target}: {:?}",
            result.warnings
        );
    }
    result
}

fn fixtures() -> Vec<Value> {
    let mut fixtures = vec![json!({"name": "reported", "source": REPORTED, "drop": true})];
    fixtures.extend(serde_json::from_str::<Vec<Value>>(CASES).unwrap());
    fixtures
}

#[test]
fn direct_comments_only_change_implicit_slots_beside_named_templates() {
    for target in ["dom", "ssr", "vapor", "vapor-ssr-fallback"] {
        for fixture in fixtures() {
            let source = fixture["source"].as_str().unwrap();
            let actual = compile(source, target);
            let without_comment = compile(&source.replace("<!-- note -->", ""), target);
            // Compare every public field. With maps disabled, removed trivia
            // must not change any output byte or runtime helper registration.
            if fixture["drop"] == true {
                let mut actual = serde_json::to_value(actual).unwrap();
                let mut expected = serde_json::to_value(without_comment).unwrap();
                // The fallback warning retains its original template span.
                if target == "vapor-ssr-fallback" {
                    actual["warnings"][0]["loc"] = Value::Null;
                    expected["warnings"][0]["loc"] = Value::Null;
                }
                assert_eq!(actual, expected, "{target}: {}", fixture["name"]);
            } else {
                assert_ne!(
                    actual.code, without_comment.code,
                    "{target}: {}",
                    fixture["name"]
                );
            }
        }
    }
}

#[test]
fn original_corpus_matches_independent_official_compilers_at_runtime() {
    for target in ["dom", "ssr", "vapor", "vapor-ssr-fallback"] {
        let cases: Vec<_> = fixtures()
            .into_iter()
            .map(|mut fixture| {
                fixture["current"] =
                    serde_json::to_value(compile(fixture["source"].as_str().unwrap(), target))
                        .unwrap();
                fixture
            })
            .collect();
        let input = json!({"target": target, "cases": cases});
        let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/tooling/support/implicit-default-slot-comments.mjs");
        let mut child = Command::new("node")
            .arg(runner)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.to_string().as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{target}: {}\n{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        let observations: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(observations["cases"].as_array().unwrap().len(), cases.len());
    }
}

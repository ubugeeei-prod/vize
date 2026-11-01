//! Authored U+3000 regressions, not a reconstruction of the reported application.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "integration fixtures retain complete process-independent compiler observations"
)]

use oxc_ast::ast::StringLiteral;
use oxc_ast_visit::Visit;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use vize_atelier_core::{CodegenResult, TemplateSyntaxMode};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_template_syntax};
use vize_atelier_sfc::{SfcCompileOptions, compile_sfc_with_template_syntax, parse_sfc};

const INPUT: &str = include_str!("fixtures/ideographic_if_whitespace.json");
const INPUT_SHA: &str = "12f5aa7b64146f3a9dc753b37060f2b82721553599bf8a79a39859954d43acd5";

#[derive(Default)]
struct Literals(Vec<String>);

impl<'a> Visit<'a> for Literals {
    fn visit_string_literal(&mut self, literal: &StringLiteral<'a>) {
        self.0.push(literal.value.to_string());
    }
}

fn module_observation(code: &str) -> Value {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, code, oxc_span::SourceType::mjs()).parse();
    let mut literals = Literals::default();
    literals.visit_program(&parsed.program);
    json!({
        "diagnostics":format!("{:#?}", parsed.diagnostics),
        "diagnosticsEmpty":parsed.diagnostics.is_empty(),
        "program":format!("{:#?}", parsed.program),
        "stringLiterals":literals.0
    })
}

#[test]
fn all_template_syntax_modes_preserve_ideographic_if_expression_semantics() {
    let input_hash = Sha256::digest(INPUT)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(input_hash, INPUT_SHA);
    let input: Value = serde_json::from_str(INPUT).unwrap();
    assert_eq!(
        input["schema"],
        "vize.compiler.ideographic-if-whitespace.v1"
    );
    let cases = input["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 6);
    let mut observations = Vec::new();
    for case in cases {
        let source = case["source"].as_str().unwrap();
        for syntax in [
            TemplateSyntaxMode::Standard,
            TemplateSyntaxMode::Strict,
            TemplateSyntaxMode::Quirks,
        ] {
            let parsed = parse_sfc(source, Default::default());
            let options = SfcCompileOptions::default();
            let options_debug = format!("{options:#?}");
            let compiled = parsed
                .as_ref()
                .ok()
                .map(|descriptor| compile_sfc_with_template_syntax(descriptor, options, syntax));
            let module = compiled
                .as_ref()
                .and_then(|result| result.as_ref().ok())
                .map(|result| module_observation(&result.code));
            observations.push(json!({
                "case":case,"templateSyntax":format!("{syntax:?}"),
                "backend":"default VDOM", "options":options_debug,
                "parse":parsed,"compile":compiled,"module":module
            }));
        }
    }
    // Capture every actual mode and full result, including code returned
    // with errors, before applying any semantic verdict to the baseline.
    eprintln!(
        "ideographic-if whole SFC observations: {}",
        serde_json::to_string_pretty(&json!({
            "input":input,"sourceRevision":std::env::var("GITHUB_SHA").ok(),
            "scope":"authored six-case default-VDOM regression; original app and versions unknown",
            "observations":observations
        }))
        .unwrap()
    );
    for observed in &observations {
        assert!(!observed["parse"]["Ok"].is_null(), "{observed:#}");
        let case = &observed["case"];
        let parsed_template = &observed["parse"]["Ok"]["template"];
        assert_eq!(parsed_template["content"], case["template"], "{observed:#}");
        if case["accepted"] == true {
            let result = &observed["compile"]["Ok"];
            assert!(!result.is_null(), "{observed:#}");
            let errors = result["errors"].as_array().unwrap();
            assert!(errors.is_empty(), "{observed:#}");
            assert_eq!(observed["module"]["diagnosticsEmpty"], true, "{observed:#}");
            let literals = observed["module"]["stringLiterals"].as_array().unwrap();
            let fullwidth = literals
                .iter()
                .filter(|value| **value == "\u{3000}")
                .collect::<Vec<_>>();
            if case["literal"].is_null() {
                assert!(fullwidth.is_empty(), "{observed:#}");
            } else {
                assert_eq!(fullwidth, vec![&case["literal"]], "{observed:#}");
            }
        } else {
            // The template-only SFC route propagates the template error;
            // generated code from another API is not source-build acceptance.
            let error = &observed["compile"]["Err"];
            assert!(!error.is_null(), "{observed:#}");
            assert_eq!(error["code"], "TEMPLATE_ERROR", "{observed:#}");
        }
    }
}

#[test]
fn invalid_ideographic_condition_retains_the_whole_original_byte_span() {
    let input: Value = serde_json::from_str(INPUT).unwrap();
    let case = input["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "invalid")
        .unwrap();
    let source = case["template"].as_str().unwrap();
    let expression = case["expression"].as_str().unwrap();
    let mut observations = Vec::new();
    for syntax in [
        TemplateSyntaxMode::Standard,
        TemplateSyntaxMode::Strict,
        TemplateSyntaxMode::Quirks,
    ] {
        let allocator = vize_l0::Allocator::default();
        let options = DomCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        };
        let (root, errors, generated) =
            compile_template_with_template_syntax(&allocator, source, options, syntax);
        let CodegenResult {
            code,
            preamble,
            map,
        } = generated;
        let error_spans = errors
            .iter()
            .map(|error| {
                json!({
                    "code":format!("{:?}",error.code),"message":error.message,
                    "start":error.loc.as_ref().map(|loc|loc.span.start),
                    "end":error.loc.as_ref().map(|loc|loc.span.end),
                    "source":error.loc.as_ref().map(|loc|loc.span.slice(source))
                })
            })
            .collect::<Vec<_>>();
        observations.push(json!({
            "templateSyntax":format!("{syntax:?}"),"source":source,
            "root":format!("{root:#?}"),"errors":format!("{errors:#?}"),
            "generated":{"code":code,"preamble":preamble,"map":map},"errorSpans":error_spans
        }));
    }
    eprintln!(
        "ideographic-if whole diagnostic observations: {}",
        serde_json::to_string_pretty(&observations).unwrap()
    );
    for observed in &observations {
        let spans = observed["errorSpans"].as_array().unwrap();
        assert_eq!(spans.len(), 1, "{observed:#}");
        let start = source.find(expression).unwrap();
        assert_eq!(spans[0]["code"], "InvalidExpression", "{observed:#}");
        assert_eq!(spans[0]["source"], expression, "{observed:#}");
        assert_eq!(spans[0]["start"], start, "{observed:#}");
        assert_eq!(spans[0]["end"], start + expression.len(), "{observed:#}");
    }
}

//! Full module contracts for closed-by-default setup exposure (#10891).
//!
//! Goldens retain public 0.595 output except the explicit default exposure edit.
//! They cover authored call ordering, async ordering, JavaScript/TypeScript,
//! client/SSR and production/development. Explicit and inline controls are unchanged.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde::Deserialize;
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    TemplateCompileOptions, compile_sfc_for_adapter, parse_sfc,
};

#[derive(Deserialize)]
struct Cases {
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    id: String,
    source: String,
    inline: bool,
    ssr: bool,
    prod: bool,
    expected: String,
}

#[test]
fn complete_setup_modules_preserve_private_and_explicit_public_contracts() {
    let cases: Cases =
        serde_json::from_str(include_str!("fixtures/script-setup-default-expose.json"))
            .expect("pinned module cases");
    assert_eq!(cases.cases.len(), 23);
    for case in cases.cases {
        let descriptor = parse_sfc(
            &case.source,
            SfcParseOptions {
                filename: "Closed.vue".into(),
                ..Default::default()
            },
        )
        .expect("parse authored fixture");
        let result = compile_sfc_for_adapter(
            &descriptor,
            SfcCompileOptions {
                parse: SfcParseOptions {
                    filename: "Closed.vue".into(),
                    ..Default::default()
                },
                script: ScriptCompileOptions {
                    inline_template: case.inline,
                    ..Default::default()
                },
                template: TemplateCompileOptions {
                    ssr: case.ssr,
                    is_prod: case.prod,
                    ..Default::default()
                },
                ..Default::default()
            },
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            CodegenOptions::default(),
            if case.inline {
                SfcScriptOutputMode::InlineTemplate
            } else {
                SfcScriptOutputMode::SeparateTemplate
            },
        )
        .expect("compile authored fixture");
        assert_eq!(result.code.as_str(), case.expected, "{}", case.id);
        let allocator = Allocator::default();
        let parsed = Parser::new(
            &allocator,
            result.code.as_str(),
            SourceType::default().with_module(!case.inline),
        )
        .with_options(oxc_parser::ParseOptions {
            allow_return_outside_function: case.inline,
            ..Default::default()
        })
        .parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{}: {:?}",
            case.id,
            parsed.diagnostics
        );
    }
}

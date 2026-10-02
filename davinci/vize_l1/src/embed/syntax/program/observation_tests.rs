use oxc_ast::ast::Statement;
use oxc_parser::{
    ParseOptions, Parser, ProgramObservation,
    config::{NoTokensParserConfig, RuntimeParserConfig, TokensParserConfig},
};
use oxc_span::SourceType;
use vize_l0::{Allocator, Span};

use super::{EmbedSource, Lang, ProgramOptions, parse_program_once};
use crate::embed::syntax::{EmbedHole, NativeSyntax};

fn source(text: &str) -> EmbedSource<'_> {
    EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap()
}

#[test]
fn nonfatal_regex_error_cannot_mint_admission_from_its_recovery_ast() {
    let allocator = Allocator::default();
    let text = "/* α */ const value = /x/uv;\r\n// β";
    let observed = Parser::new(allocator.as_oxc(), text, SourceType::mjs()).parse_observed();
    assert!(!observed.panicked());
    assert!(observed.diagnostics().has_errors());
    assert!(observed.admitted().is_none());
    assert_eq!(observed.comments().len(), 2);
    let tree = parse_program_once(&allocator, source(text), ProgramOptions::module(Lang::Js));
    assert_eq!(tree.hole(), Some(EmbedHole::Syntax));
    assert!(tree.program().is_none());
    assert!(tree.admitted_program().is_none());
    assert_eq!(tree.source().text(), text);
    assert_eq!(tree.comments().count(), 2);
    assert_eq!(tree.diagnostics().count(), observed.diagnostics().len());
    assert_eq!(
        tree.diagnostics().next().unwrap().message(),
        observed
            .diagnostics()
            .iter()
            .next()
            .unwrap()
            .message
            .as_ref(),
    );
    assert!(core::mem::needs_drop::<ProgramObservation<'_>>());
    assert!(core::mem::needs_drop::<NativeSyntax<'_>>());
    let retained = tree.into_expression().unwrap_err();
    assert_eq!(retained.hole(), Some(EmbedHole::Syntax));
    assert_eq!(retained.comments().count(), 2);
    assert_eq!(retained.diagnostics().count(), observed.diagnostics().len());
    drop(retained);
    drop(observed);
}

#[test]
fn clean_admission_borrows_actual_program_source_profile_and_options() {
    let allocator = Allocator::default();
    let text = "/* α */ export const value = 1;\r\n// β";
    let tree = parse_program_once(&allocator, source(text), ProgramOptions::module(Lang::Js));
    let admitted = tree.admitted_program().unwrap();
    assert_eq!(admitted.source(), text);
    assert_eq!(admitted.source().as_ptr(), text.as_ptr());
    assert_eq!(admitted.source_type(), SourceType::mjs());
    assert_eq!(admitted.options(), ParseOptions::default());
    assert!(core::ptr::eq(admitted.program(), tree.program().unwrap()));
    assert!(matches!(
        admitted.program().body[0],
        Statement::ExportNamedDeclaration(_)
    ));
    assert_eq!(tree.comments().count(), 2);
    assert_eq!(tree.diagnostics().count(), 0);
}

#[test]
fn nondefault_options_remain_actual_observations_and_not_default_authority() {
    let allocator = Allocator::default();
    for options in [
        ParseOptions {
            enable_ident_hashes: false,
            ..ParseOptions::default()
        },
        ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        },
    ] {
        let text = if options.allow_return_outside_function {
            "return 1;"
        } else {
            "const value = 1;"
        };
        let observed = Parser::new(allocator.as_oxc(), text, SourceType::cjs())
            .with_options(options)
            .parse_observed();
        let admitted = observed.admitted().unwrap();
        assert_eq!(admitted.options(), options);
        assert_ne!(admitted.options(), ParseOptions::default());
        assert_eq!(admitted.source_type(), SourceType::cjs());
        assert_eq!(admitted.source(), text);
    }
}

#[test]
fn unambiguous_inference_does_not_replace_original_profile_identity() {
    let allocator = Allocator::default();
    let profile = SourceType::unambiguous();
    let observed =
        Parser::new(allocator.as_oxc(), "export const value = 1;", profile).parse_observed();
    let admitted = observed.admitted().unwrap();
    assert_eq!(admitted.source_type(), profile);
    assert!(admitted.program().source_type.is_module());
    assert_ne!(admitted.source_type(), admitted.program().source_type);
}

#[test]
fn fatal_results_keep_owned_diagnostics_and_comments_without_admission() {
    let allocator = Allocator::default();
    let text = "/* before */ const value = ; // after";
    let observed = Parser::new(allocator.as_oxc(), text, SourceType::mjs()).parse_observed();
    assert!(observed.panicked());
    assert!(observed.admitted().is_none());
    assert!(observed.diagnostics().has_errors());
    assert_eq!(observed.comments().len(), 1);
}

#[test]
fn flow_results_keep_original_observations_and_never_mint_admission() {
    let allocator = Allocator::default();
    let text = "/* @flow */ super;";
    let observed = Parser::new(allocator.as_oxc(), text, SourceType::mjs()).parse_observed();
    assert!(observed.is_flow_language());
    assert!(observed.admitted().is_none());
    let tree = parse_program_once(&allocator, source(text), ProgramOptions::module(Lang::Js));
    assert_eq!(tree.hole(), Some(EmbedHole::UnsupportedFlow));
    assert!(tree.admitted_program().is_none());
    assert_eq!(tree.source().text(), text);
    assert_eq!(tree.comments().count(), observed.comments().len());
    drop(tree);
    drop(observed);
}

fn assert_stock_observation(observed: ProgramObservation<'_>, options: ParseOptions) {
    let admitted = observed.admitted().unwrap();
    assert_eq!(admitted.source(), "/* stock */ const value = 1;");
    assert_eq!(admitted.source_type(), SourceType::cjs());
    assert_eq!(admitted.options(), options);
    assert_eq!(admitted.program().body.len(), 1);
    assert_eq!(observed.comments().len(), 1);
    assert_eq!(observed.diagnostics().len(), 0);
    assert!(!observed.panicked());
}

#[test]
fn no_tokens_stock_config_retains_actual_profile_options_and_errors() {
    let allocator = Allocator::default();
    let options = ParseOptions {
        enable_ident_hashes: false,
        ..ParseOptions::default()
    };
    assert_stock_observation(
        Parser::new(
            allocator.as_oxc(),
            "/* stock */ const value = 1;",
            SourceType::cjs(),
        )
        .with_options(options)
        .with_config(NoTokensParserConfig)
        .parse_observed(),
        options,
    );
    let invalid = Parser::new(
        allocator.as_oxc(),
        "const value = /x/uv;",
        SourceType::cjs(),
    )
    .with_config(NoTokensParserConfig)
    .parse_observed();
    assert!(invalid.diagnostics().has_errors());
    assert!(invalid.admitted().is_none());
}

#[test]
fn tokens_stock_config_retains_actual_profile_options_and_errors() {
    let allocator = Allocator::default();
    let options = ParseOptions {
        preserve_parens: false,
        ..ParseOptions::default()
    };
    assert_stock_observation(
        Parser::new(
            allocator.as_oxc(),
            "/* stock */ const value = 1;",
            SourceType::cjs(),
        )
        .with_options(options)
        .with_config(TokensParserConfig)
        .parse_observed(),
        options,
    );
    let invalid = Parser::new(
        allocator.as_oxc(),
        "const value = /x/uv;",
        SourceType::cjs(),
    )
    .with_config(TokensParserConfig)
    .parse_observed();
    assert!(invalid.diagnostics().has_errors());
    assert!(invalid.admitted().is_none());
}

#[test]
fn runtime_stock_configs_retain_actual_profile_options_and_errors() {
    let allocator = Allocator::default();
    let options = ParseOptions {
        preserve_parens: false,
        ..ParseOptions::default()
    };
    for tokens in [false, true] {
        assert_stock_observation(
            Parser::new(
                allocator.as_oxc(),
                "/* stock */ const value = 1;",
                SourceType::cjs(),
            )
            .with_options(options)
            .with_config(RuntimeParserConfig::new(tokens))
            .parse_observed(),
            options,
        );
        let invalid = Parser::new(
            allocator.as_oxc(),
            "const value = /x/uv;",
            SourceType::cjs(),
        )
        .with_config(RuntimeParserConfig::new(tokens))
        .parse_observed();
        assert!(invalid.diagnostics().has_errors());
        assert!(invalid.admitted().is_none());
    }
}

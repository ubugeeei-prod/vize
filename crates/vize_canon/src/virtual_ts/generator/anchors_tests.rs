use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::String;
use vize_croquis::{Analyzer, AnalyzerOptions};

use crate::batch::{ImportRewriter, generate_vue_document_virtual_ts};
use crate::virtual_ts::VirtualTsOptions;

const CORPUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/quoted-prop-setup-anchors/"
);

fn parse_errors(source: &str) -> usize {
    Parser::new(&Allocator::default(), source, SourceType::ts())
        .parse()
        .diagnostics
        .len()
}

#[test]
fn complete_original_native_projections_retain_the_syntax_regression_control() {
    for name in ["popover", "tooltip"] {
        let before =
            std::fs::read_to_string(format!("{CORPUS}{name}.original-h.vue.ts.txt")).unwrap();
        assert!(
            parse_errors(&before) > 0,
            "literal original-H projection is the regression control"
        );
    }
    // The fresh full ElementPlus projections run through their original pinned
    // import graph in the required native integration lane, after App E2E has
    // hydrated it. A hand-built Croquis summary cannot stand in for that graph.
}

#[test]
fn quoted_prop_keys_keep_valid_unicode_and_dollar_identifiers() {
    let source =
        std::fs::read_to_string(format!("{CORPUS}quoted-properties-and-unicode.vue.txt")).unwrap();
    let descriptor = parse_sfc(&source, SfcParseOptions::default()).unwrap();
    let script = descriptor.script_setup.as_ref().unwrap().content.as_ref();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    assert_eq!(parsed.diagnostics.len(), 0);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    let summary = analyzer.finish();
    let usage = super::super::imports::collect_identifier_usage(&parsed.program);
    let mut anchors = String::default();
    super::emit_setup_binding_anchors(
        &mut anchors,
        &summary,
        Some(script),
        &usage,
        None,
        "Reference setup bindings (used in template/CSS v-bind)",
    );
    assert_eq!(
        anchors.as_str(),
        "\n  // Reference setup bindings (used in template/CSS v-bind)\n  void $setup; void $value; void _private; void café; void props; void résumé;\n"
    );
    assert_eq!(
        summary
            .macros
            .props()
            .iter()
            .map(|prop| prop.name.as_str())
            .collect::<Vec<_>>(),
        [
            "onUpdate:visible",
            "data-id",
            "class",
            "default",
            "café",
            "$value",
            "_private"
        ]
    );
    let output = generate_vue_document_virtual_ts(
        std::path::Path::new("Identifiers.vue"),
        &source,
        &VirtualTsOptions::default(),
        &ImportRewriter::new(),
        false,
    )
    .unwrap();
    assert_eq!(parse_errors(&output.pre_rewrite_code), 0);
}

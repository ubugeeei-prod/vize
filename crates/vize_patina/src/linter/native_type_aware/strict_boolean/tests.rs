//! Syntax contexts are fixed independently of checker type classification.
use super::queries::Collector;
use crate::{LintPreset, Linter};
use oxc_allocator::Allocator;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::config::StrictBooleanExpressionsOptions;

#[test]
fn syntax_contexts_include_logical_left_but_not_value_right_operands() {
    let source = "if (a && b) {} const value = c || d; const negated = !e; while (f) {} do {} while (g); for (; h ;) {} const choice = i ? 1 : 0;";
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let mut collector = Collector {
        conditions: Vec::new(),
    };
    collector.visit_program(&parsed.program);
    collector
        .conditions
        .sort_unstable_by_key(|condition| (condition.start, condition.end));
    collector
        .conditions
        .dedup_by_key(|condition| (condition.start, condition.end));
    let expressions: Vec<_> = collector
        .conditions
        .iter()
        .map(|condition| {
            source
                .get(condition.start as usize..condition.end as usize)
                .expect("authored range")
        })
        .collect();
    assert_eq!(expressions, vec!["a", "b", "c", "e", "f", "g", "h", "i"]);
}

#[test]
fn options_do_not_register_the_rule_in_a_preset() {
    for preset in [
        LintPreset::HappyPath,
        LintPreset::Opinionated,
        LintPreset::Essential,
        LintPreset::Incremental,
        LintPreset::Ecosystem,
        LintPreset::Nuxt,
    ] {
        let linter = Linter::with_preset(preset)
            .with_type_aware_lint(true)
            .with_strict_boolean_expressions_options(StrictBooleanExpressionsOptions::default());
        assert!(!linter.registry.has_rule(super::RULE_STRICT_BOOLEAN));
    }
}

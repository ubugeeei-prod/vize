use super::*;
use crate::linter::{LintResult, Linter};
use crate::rule::Rule;
use crate::rule::RuleRegistry;

fn lint(rule: Box<dyn Rule>, template: &str) -> LintResult {
    let mut registry = RuleRegistry::new();
    registry.register(rule);
    Linter::with_registry(registry).lint_template(template, "Component.vue")
}

#[test]
fn core_rules_read_retained_expression_ast() {
    let eqeqeq = lint(Box::new(Eqeqeq), "<p>{{ value == null }}</p>");
    assert_eq!(eqeqeq.warning_count, 1);
    let sparse = lint(Box::new(NoSparseArrays), "<p :data=\"[1,,2]\" />");
    assert_eq!(sparse.warning_count, 1);
    let console = lint(Box::new(NoConsole), "<p>{{ console.log(value) }}</p>");
    assert_eq!(console.warning_count, 1);
}

#[test]
fn template_specific_rules_report_compatibility_cases() {
    let literals = lint(
        Box::new(NoLiteralsInTemplate),
        "<Widget :options=\"{ eager: true }\" />",
    );
    assert_eq!(literals.warning_count, 1);
    let restricted = lint(Box::new(NoRestrictedVBind), "<div :v-test=\"value\" />");
    assert_eq!(restricted.warning_count, 1);
    let model = lint(
        Box::new(PreferVModel),
        "<Widget :model-value=\"value\" @update:model-value=\"value = $event\" />",
    );
    assert_eq!(model.warning_count, 1);
}

#[test]
fn repeated_conditional_components_require_keys() {
    let result = lint(
        Box::new(VIfElseKey),
        "<section><Widget v-if=\"first\" /><Widget v-else /></section>",
    );
    assert_eq!(result.warning_count, 2);
}

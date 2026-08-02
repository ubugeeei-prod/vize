use serde_json::Value;
use vize_patina::rules::vue::RequireVForKey;
use vize_patina::{LintResult, Linter, RuleRegistry};

const CASES: &str = include_str!("fixtures/require-v-for-key-object-bind.json");
const RULE: &str = "vue/require-v-for-key";

fn linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(RequireVForKey));
    Linter::with_registry(registry)
}

fn rule_diagnostics(result: &LintResult) -> usize {
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .count()
}

#[test]
fn object_bound_key_matrix_matches_the_recorded_contract() {
    let cases: Value = serde_json::from_str(CASES).unwrap();
    let linter = linter();

    for case in cases.as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let expected = case["patinaDiagnostics"].as_u64().unwrap() as usize;
        let result = linter.lint_sfc(source, id);

        assert_eq!(rule_diagnostics(&result), expected, "{id}");
    }
}

#[test]
fn pinned_elk_reproduction_has_zero_false_positives_and_false_negatives() {
    let cases: Value = serde_json::from_str(CASES).unwrap();
    let elk = &cases.as_array().unwrap()[0];
    assert_eq!(elk["id"], "elk-common-paginator");

    let expected = elk["eslintDiagnostics"].as_u64().unwrap() as usize;
    let result = linter().lint_sfc(elk["source"].as_str().unwrap(), "CommonPaginator.vue");
    let actual = rule_diagnostics(&result);

    assert_eq!(actual, expected, "false-positive or false-negative drift");
    assert_eq!(actual, 0, "the pinned Elk reproduction must stay clean");
}

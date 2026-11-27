//! A component's `for` prop may render as its control's `id` (Quasar's
//! `<QInput for>`), so it resolves references and is not itself checked.

use vize_patina::{LintPreset, Linter};

const RULE: &str = "a11y/no-refer-to-non-existent-id";

fn reported(source: &str, filename: &str) -> Vec<(u32, u32)> {
    let linter =
        Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![RULE.into()]));
    let result = linter.lint_sfc(source, filename);
    assert!(
        result.diagnostics.iter().all(|d| d.rule_name == RULE),
        "{result:#?}"
    );
    result
        .diagnostics
        .iter()
        .map(|d| (d.start, d.end))
        .collect()
}

fn span_of(source: &str, needle: &str) -> (u32, u32) {
    let start = source.find(needle).unwrap();
    (start as u32, (start + needle.len()) as u32)
}

#[test]
fn pinned_component_for_fixture_reports_only_unresolved_references() {
    let child = include_str!(
        "../../../tests/_fixtures/differential/linter/id-reference-component-for/MyInput.vue.txt"
    );
    let parent = include_str!(
        "../../../tests/_fixtures/differential/linter/id-reference-component-for/LabelForComponent.vue.txt"
    );
    let controls = include_str!(
        "../../../tests/_fixtures/differential/linter/id-reference-component-for/ComponentIdControls.vue.txt"
    );
    assert_eq!(reported(child, "MyInput.vue"), []);
    assert_eq!(reported(parent, "LabelForComponent.vue"), []);
    assert_eq!(
        reported(controls, "ComponentIdControls.vue"),
        [
            span_of(controls, r#"for="missing""#),
            span_of(controls, r#"aria-describedby="search-hint""#),
        ]
    );
}

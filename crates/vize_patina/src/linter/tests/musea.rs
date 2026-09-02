use super::Linter;
use crate::{LintPreset, Severity};

const ART_SOURCE: &str = r#"<art component="./Button.vue">
  <variant name="empty"></variant>
</art>
"#;

#[test]
fn default_presets_do_not_lint_art_files() {
    let result = Linter::new().lint_sfc(ART_SOURCE, "Button.art.vue");

    assert!(!result.has_diagnostics());
}

#[test]
fn explicitly_enabled_musea_rules_lint_art_files() {
    let linter = Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![
        "musea/require-title".into(),
        "musea/no-empty-variant".into(),
    ]));

    let result = linter.lint_sfc(ART_SOURCE, "Button.art.vue");
    let rules = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name)
        .collect::<Vec<_>>();

    assert_eq!(rules, ["musea/require-title", "musea/no-empty-variant"]);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn musea_rules_stay_scoped_to_art_vue_files() {
    let linter = Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![
        "musea/require-title".into(),
        "musea/no-empty-variant".into(),
    ]));

    let result = linter.lint_sfc(ART_SOURCE, "Button.vue");

    assert!(!result.has_diagnostics());
}

#[test]
fn musea_rule_severity_overrides_recount_results() {
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["musea/require-title".into()]))
        .with_rule_severity_overrides(vec![("musea/require-title".into(), Severity::Warning)]);

    let result = linter.lint_sfc(ART_SOURCE, "Button.art.vue");

    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.diagnostics[0].severity, Severity::Warning);
}

#[test]
fn musea_category_can_disable_enabled_musea_rules() {
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["musea/require-title".into()]))
        .with_disabled_categories(vec!["musea".into()]);

    let result = linter.lint_sfc(ART_SOURCE, "Button.art.vue");

    assert!(!result.has_diagnostics());
}

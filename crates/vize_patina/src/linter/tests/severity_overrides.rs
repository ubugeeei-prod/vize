use super::Linter;
use crate::{LintPreset, Severity};

#[test]
fn script_rule_severity_override_recounts_sfc_result() {
    let source = r#"<script setup lang="ts">
const emit = defineEmits(["update:current-step-index"])
emit("update:current-step-index")
</script>
"#;
    let result = Linter::with_preset(LintPreset::Ecosystem)
        .with_enabled_rules(Some(vec!["script/custom-event-name-casing".into()]))
        .with_rule_severity_overrides(vec![(
            "script/custom-event-name-casing".into(),
            Severity::Warning,
        )])
        .lint_sfc(source, "Stepper.vue");

    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
    assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].severity, Severity::Warning);
}

#[test]
fn css_rule_severity_override_recounts_sfc_result() {
    let source = r#"<style>
.button { color: red !important; }
</style>
"#;
    let result = Linter::with_preset(LintPreset::Ecosystem)
        .with_enabled_rules(Some(vec!["css/no-important".into()]))
        .with_rule_severity_overrides(vec![("css/no-important".into(), Severity::Error)])
        .lint_sfc(source, "Button.vue");

    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
}

#[test]
fn css_rule_category_severity_override_recounts_sfc_result() {
    let source = r#"<style>
.button { color: red !important; }
</style>
"#;
    let result = Linter::with_preset(LintPreset::Ecosystem)
        .with_enabled_rules(Some(vec!["css/no-important".into()]))
        .with_category_severity_overrides(vec![("style".into(), Severity::Error)])
        .lint_sfc(source, "Button.vue");

    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
}

#[test]
fn disabled_category_disables_classified_css_rule() {
    let source = r#"<style>
.button { color: red !important; }
</style>
"#;
    let result = Linter::with_preset(LintPreset::Ecosystem)
        .with_enabled_rules(Some(vec!["css/no-important".into()]))
        .with_disabled_categories(vec!["style".into()])
        .lint_sfc(source, "Button.vue");

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 0);
}

const NUXT_CONFIG_ORDER_SOURCE: &str = "export default { ssr: true, modules: [] }";

#[test]
fn script_rule_category_severity_override_recounts_sfc_result() {
    let result = Linter::with_preset(LintPreset::Nuxt)
        .with_category_severity_overrides(vec![("style".into(), Severity::Warning)])
        .lint_script(NUXT_CONFIG_ORDER_SOURCE, "nuxt.config.ts");

    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
    assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    assert_eq!(
        result.diagnostics[0].rule_name,
        "nuxt/nuxt-config-keys-order"
    );
}

#[test]
fn existing_perf_category_classification_applies_to_script_rule() {
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["script/no-async-in-computed".into()]))
        .with_category_severity_overrides(vec![("perf".into(), Severity::Warning)])
        .lint_script("const value = computed(async () => 1)", "component.ts");

    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
    assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].severity, Severity::Warning);
}

#[test]
fn unrelated_category_does_not_override_script_rule_severity() {
    let result = Linter::with_preset(LintPreset::Nuxt)
        .with_category_severity_overrides(vec![("correctness".into(), Severity::Warning)])
        .lint_script(NUXT_CONFIG_ORDER_SOURCE, "nuxt.config.ts");

    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
}

#[test]
fn disabled_category_disables_classified_script_rule() {
    let result = Linter::with_preset(LintPreset::Nuxt)
        .with_disabled_categories(vec!["style".into()])
        .lint_script(NUXT_CONFIG_ORDER_SOURCE, "nuxt.config.ts");

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 0);
}

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

use super::NoUndefinedRefs;
use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleCategory, RuleRegistry};

const RULE: &str = "vue/no-undefined-refs";

/// Lint a full SFC with only this rule enabled.
///
/// `vue/no-undefined-refs` is a member of `SEMANTIC_TEMPLATE_RULES`, so
/// enabling it alone still produces the croquis analysis the rule reads.
fn lint_sfc(sfc: &str) -> LintResult {
    Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_sfc(sfc, "Probe.vue")
}

fn findings(result: &LintResult) -> Vec<(&'static str, Severity, u32, u32, &str)> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name,
                diagnostic.severity,
                diagnostic.start,
                diagnostic.end,
                diagnostic.message.as_str(),
            )
        })
        .collect()
}

fn none() -> Vec<(&'static str, Severity, u32, u32, &'static str)> {
    Vec::new()
}

fn undefined_at(sfc: &str, name: &str) -> (&'static str, Severity, u32, u32, std::string::String) {
    let start = sfc.rfind(name).expect("undefined identifier") as u32;
    (
        RULE,
        Severity::Warning,
        start,
        start + name.len() as u32,
        format!("Variable '{name}' is not defined"),
    )
}

#[test]
fn test_meta() {
    let rule = NoUndefinedRefs;
    assert_eq!(rule.meta().name, RULE);
    assert_eq!(rule.meta().category, RuleCategory::Recommended);
    assert_eq!(rule.meta().default_severity, Severity::Warning);
}

#[test]
fn opt_in_registry_owns_the_rule_and_default_presets_do_not() {
    assert!(RuleRegistry::with_opt_in_rules().has_rule(RULE));
    assert!(!RuleRegistry::with_preset(crate::preset::LintPreset::HappyPath).has_rule(RULE));
    assert!(!RuleRegistry::with_preset(crate::preset::LintPreset::Ecosystem).has_rule(RULE));
    assert!(!RuleRegistry::with_preset(crate::preset::LintPreset::Opinionated).has_rule(RULE));
}

#[test]
fn default_preset_stays_silent_on_an_undefined_template_ref() {
    let sfc = r#"<script setup>
const known = 1
</script>
<template>
  <p>{{ known }} {{ missing }}</p>
</template>
"#;
    let result = Linter::new().lint_sfc(sfc, "Probe.vue");
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != RULE),
        "default preset must not fire the opt-in rule: {:#?}",
        result.diagnostics
    );
}

#[test]
fn reports_an_undefined_interpolation_identifier() {
    let sfc = r#"<script setup>
const known = 1
</script>
<template>
  <p>{{ known }} {{ missing }}</p>
</template>
"#;
    let expected = undefined_at(sfc, "missing");
    assert_eq!(
        findings(&lint_sfc(sfc))
            .into_iter()
            .map(|(rule, severity, start, end, message)| {
                (rule, severity, start, end, message.to_string())
            })
            .collect::<Vec<_>>(),
        vec![expected]
    );
}

#[test]
fn additional_rules_can_enable_the_rule_without_replacing_the_preset() {
    let sfc = r#"<script setup>
const known = 1
</script>
<template>
  <p>{{ known }} {{ missing }}</p>
</template>
"#;
    let result = Linter::new()
        .with_additional_rules(vec![RULE.into()])
        .lint_sfc(sfc, "Probe.vue");
    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.rule_name == RULE && diagnostic.message.as_str().contains("missing")
        }),
        "config-enable must instantiate the rule: {:#?}",
        result.diagnostics
    );
}

#[test]
fn ignores_a_script_setup_binding() {
    let sfc = r#"<script setup>
const known = 1
</script>
<template>
  <p>{{ known }}</p>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_v_for_alias() {
    let sfc = r#"<script setup>
const items = ['a']
</script>
<template>
  <span v-for="item in items" :key="item">{{ item }}</span>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_javascript_global() {
    let sfc = r#"<script setup>
</script>
<template>
  <span>{{ Math.max(1, 2) }}</span>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

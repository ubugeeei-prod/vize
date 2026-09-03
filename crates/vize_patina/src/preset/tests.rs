//! Preset and rule-map coverage for the built-in Patina lint presets.

use super::LintPreset;
use crate::Linter;
use crate::rule::RuleRegistry;
use std::collections::BTreeSet;

#[test]
fn parses_common_aliases() {
    assert_eq!(LintPreset::parse("default"), Some(LintPreset::HappyPath));
    assert_eq!(
        LintPreset::parse("recommended"),
        Some(LintPreset::HappyPath)
    );
    assert_eq!(
        LintPreset::parse("general-recommended"),
        Some(LintPreset::HappyPath)
    );
    assert_eq!(LintPreset::parse("all"), Some(LintPreset::Opinionated));
    assert_eq!(LintPreset::parse("strict"), Some(LintPreset::Opinionated));
    assert_eq!(
        LintPreset::parse("incremental"),
        Some(LintPreset::Incremental)
    );
    assert_eq!(LintPreset::parse("ecosystem"), Some(LintPreset::Ecosystem));
    assert_eq!(LintPreset::parse("nuxt"), Some(LintPreset::Nuxt));
    assert_eq!(LintPreset::parse("unknown"), None);
}

#[test]
fn preset_rule_membership_snapshot() {
    let snapshot = serde_json::json!({
        "happy_path": rule_names(LintPreset::HappyPath),
        "opinionated": rule_names(LintPreset::Opinionated),
        "essential": rule_names(LintPreset::Essential),
        "ecosystem": ecosystem_rule_names(),
        "incremental": rule_names(LintPreset::Incremental),
        "nuxt": rule_names(LintPreset::Nuxt),
        "opt_in": opt_in_rule_names(),
    });

    insta::assert_snapshot!(
        "lint_preset_rule_membership",
        serde_json::to_string_pretty(&snapshot).unwrap()
    );
}

#[test]
fn script_preset_membership_matches_builtin_rule_metadata() {
    for meta in crate::linter::script_rules::builtin_script_rules() {
        for preset_name in meta.presets {
            assert!(
                LintPreset::parse(preset_name).is_some(),
                "{} declares unknown preset {preset_name}",
                meta.name
            );
        }

        for preset in LintPreset::ALL {
            assert_eq!(
                super::builtin_script_rule_names(preset).contains(&meta.name),
                meta.presets.contains(&preset.as_str()),
                "{} has stale script-rule membership for {}",
                meta.name,
                preset.as_str()
            );
        }
    }
}

#[test]
fn happy_path_keeps_opinionated_rules_opt_in() {
    let happy_path = RuleRegistry::with_preset(LintPreset::HappyPath);
    let opinionated = RuleRegistry::with_preset(LintPreset::Opinionated);

    assert!(happy_path.has_rule("vue/attribute-order"));
    assert!(happy_path.has_rule("vue/component-definition-name-casing"));
    assert!(happy_path.has_rule("vue/html-quotes"));
    assert!(happy_path.has_rule("vue/mustache-interpolation-spacing"));
    assert!(happy_path.has_rule("vue/no-lone-template"));
    assert!(happy_path.has_rule("vue/no-multi-spaces"));
    assert!(happy_path.has_rule("vue/no-unused-properties"));
    assert!(happy_path.has_rule("vue/prop-name-casing"));
    assert!(happy_path.has_rule("vue/require-scoped-style"));
    assert!(happy_path.has_rule("vue/sfc-element-order"));
    assert!(happy_path.has_rule("vue/single-style-block"));
    assert!(happy_path.has_rule("vue/v-on-style"));
    assert!(happy_path.has_rule("vue/v-slot-style"));
    assert!(happy_path.has_rule("vapor/no-vue-lifecycle-events"));
    assert!(happy_path.has_rule("type/require-typed-props"));
    assert!(happy_path.has_rule("type/require-typed-emits"));
    assert!(!happy_path.has_rule("type/no-unsafe-template-binding"));
    assert!(!happy_path.has_rule("type/no-reactivity-loss"));
    assert!(happy_path.has_rule("html/no-empty-palpable-content"));
    assert!(!happy_path.has_rule("vue/multi-word-component-names"));
    assert!(!happy_path.has_rule("vue/no-undefined-refs"));
    assert!(!happy_path.has_rule("a11y/use-list"));
    assert!(RuleRegistry::with_opt_in_rules().has_rule("vue/no-undefined-refs"));
    assert!(opinionated.has_rule("vue/attribute-order"));
    assert!(opinionated.has_rule("vue/component-definition-name-casing"));
    assert!(opinionated.has_rule("vue/html-quotes"));
    assert!(opinionated.has_rule("vue/mustache-interpolation-spacing"));
    assert!(opinionated.has_rule("vue/no-lone-template"));
    assert!(opinionated.has_rule("vue/no-multi-spaces"));
    assert!(opinionated.has_rule("vue/no-unused-properties"));
    assert!(opinionated.has_rule("vue/prop-name-casing"));
    assert!(opinionated.has_rule("vue/require-scoped-style"));
    assert!(opinionated.has_rule("vue/sfc-element-order"));
    assert!(opinionated.has_rule("vue/single-style-block"));
    assert!(opinionated.has_rule("vue/v-on-style"));
    assert!(opinionated.has_rule("vue/v-slot-style"));
    assert!(opinionated.has_rule("vapor/no-vue-lifecycle-events"));
    assert!(opinionated.has_rule("type/require-typed-props"));
    assert!(opinionated.has_rule("type/require-typed-emits"));
    assert!(opinionated.has_rule("type/no-unsafe-template-binding"));
    assert!(opinionated.has_rule("type/no-reactivity-loss"));
    assert!(opinionated.has_rule("html/no-empty-palpable-content"));
    assert!(opinionated.has_rule("vue/multi-word-component-names"));
    assert!(opinionated.has_rule("a11y/use-list"));
    assert!(!opinionated.has_rule("ecosystem/router-link-require-to"));
    assert!(RuleRegistry::with_ecosystem().has_rule("ecosystem/router-link-require-to"));
    assert!(RuleRegistry::with_ecosystem().has_rule("ecosystem/vue-i18n-no-missing-key"));
    let prefer_nuxt_link = "ecosystem/nuxt-prefer-nuxt-link";
    assert!(RuleRegistry::with_preset(LintPreset::Nuxt).has_rule(prefer_nuxt_link));
    assert!(!RuleRegistry::with_ecosystem().has_rule(prefer_nuxt_link));
    assert!(RuleRegistry::with_all().has_rule(prefer_nuxt_link));
    assert!(RuleRegistry::with_opt_in_rules().has_rule("ecosystem/router-link-require-to"));
    assert!(RuleRegistry::with_opt_in_rules().has_rule("ecosystem/vue-i18n-no-missing-key"));
    let happy_path_script = super::builtin_script_rule_names(LintPreset::HappyPath);
    let essential_script = super::builtin_script_rule_names(LintPreset::Essential);
    assert!(happy_path_script.contains(&"script/valid-define-props"));
    assert!(happy_path_script.contains(&"script/no-import-compiler-macros"));
    assert!(happy_path_script.contains(&"script/no-ref-as-operand"));
    assert!(happy_path_script.contains(&"script/no-duplicate-attr-inheritance"));
    assert!(essential_script.contains(&"script/valid-define-props"));
    assert!(essential_script.contains(&"script/no-import-compiler-macros"));
    assert!(essential_script.contains(&"script/no-ref-as-operand"));
    assert!(!essential_script.contains(&"script/no-duplicate-attr-inheritance"));
    assert!(!happy_path_script.contains(&"script/no-unused-emit-declarations"));
    assert!(!essential_script.contains(&"script/no-unused-emit-declarations"));
    assert!(!happy_path_script.contains(&"script/no-reactive-destructure"));
    assert!(
        !super::builtin_script_rule_names(LintPreset::HappyPath).contains(&"script/no-options-api")
    );
    assert!(
        !super::builtin_script_rule_names(LintPreset::HappyPath)
            .contains(&"script/define-props-declaration")
    );
    assert!(
        !super::builtin_script_rule_names(LintPreset::HappyPath)
            .contains(&"script/no-with-defaults")
    );
    assert!(
        !super::builtin_script_rule_names(LintPreset::HappyPath)
            .contains(&"script/require-explicit-emits")
    );
    assert!(
        super::builtin_script_rule_names(LintPreset::Opinionated)
            .contains(&"script/no-options-api")
    );
    assert!(!super::builtin_script_rule_names(LintPreset::Nuxt).contains(&"script/no-options-api"));
    assert!(
        !super::builtin_script_rule_names(LintPreset::Nuxt)
            .contains(&"script/no-get-current-instance")
    );
    assert!(!super::builtin_script_rule_names(LintPreset::Nuxt).contains(&"script/no-next-tick"));
    assert!(
        !super::builtin_script_rule_names(LintPreset::Opinionated)
            .contains(&"script/no-export-in-script-setup")
    );
    assert!(
        !super::builtin_script_rule_names(LintPreset::Nuxt)
            .contains(&"script/no-export-in-script-setup")
    );
    assert!(
        super::builtin_script_rule_names(LintPreset::Nuxt).contains(&"nuxt/prefer-import-meta")
    );
    assert!(
        super::builtin_script_rule_names(LintPreset::Nuxt)
            .contains(&"nuxt/no-page-meta-runtime-values")
    );
    assert!(
        super::builtin_script_rule_names(LintPreset::Opinionated)
            .contains(&"script/no-get-current-instance")
    );
    assert!(
        !super::builtin_script_rule_names(LintPreset::Opinionated).contains(&"script/no-next-tick")
    );
    assert!(
        !super::builtin_script_rule_names(LintPreset::Opinionated)
            .contains(&"ecosystem/pinia-prefer-store-to-refs")
    );
    assert!(
        crate::linter::script_rules::opt_in_script_rule_names()
            .contains(&"ecosystem/pinia-prefer-store-to-refs")
    );
    assert!(
        super::ecosystem_builtin_script_rule_names()
            .contains(&"ecosystem/vue-router-prefer-named-push")
    );
}

#[test]
fn happy_path_runs_script_correctness_and_low_noise_warnings() {
    let source = r#"<script setup lang="ts">
import { defineProps, ref } from 'vue'

const localProps = { count: Number }
const emit = defineEmits<{ save: []; unused: [] }>()
const count = ref(0)

defineProps<{ count: number }>(localProps)
if (count) {}
emit('save')
</script>

<template>
  <button v-bind="$attrs">{{ count }}</button>
</template>
"#;

    let result = Linter::with_preset(LintPreset::HappyPath).lint_sfc(source, "App.vue");
    let rules = diagnostic_rule_names(&result);

    assert!(rules.contains("script/no-import-compiler-macros"));
    assert!(rules.contains("script/valid-define-props"));
    assert!(rules.contains("script/no-ref-as-operand"));
    assert!(rules.contains("script/no-duplicate-attr-inheritance"));
    assert!(!rules.contains("script/no-reactive-destructure"));
    assert!(!rules.contains("script/no-unused-emit-declarations"));
    assert!(!rules.contains("script/no-options-api"));
    assert!(!rules.contains("script/define-props-declaration"));
    assert!(!rules.contains("script/require-explicit-emits"));
}

#[test]
fn essential_runs_script_correctness_without_happy_path_warnings() {
    let source = r#"<script setup lang="ts">
import { defineProps, ref } from 'vue'

const localProps = { count: Number }
const emit = defineEmits<{ save: []; unused: [] }>()
const count = ref(0)

defineProps<{ count: number }>(localProps)
if (count) {}
emit('save')
</script>

<template>
  <button v-bind="$attrs">{{ count }}</button>
</template>
"#;

    let result = Linter::with_preset(LintPreset::Essential).lint_sfc(source, "App.vue");
    let rules = diagnostic_rule_names(&result);

    assert!(rules.contains("script/no-import-compiler-macros"));
    assert!(rules.contains("script/valid-define-props"));
    assert!(rules.contains("script/no-ref-as-operand"));
    assert!(!rules.contains("script/no-duplicate-attr-inheritance"));
    assert!(!rules.contains("script/no-unused-emit-declarations"));
}

#[test]
fn incremental_starts_empty() {
    let incremental = RuleRegistry::with_preset(LintPreset::Incremental);

    assert!(!incremental.has_rule("vue/require-v-for-key"));
    assert!(super::builtin_script_rule_names(LintPreset::Incremental).is_empty());
}

fn rule_names(preset: LintPreset) -> Vec<&'static str> {
    let mut rules: Vec<_> = RuleRegistry::with_preset(preset)
        .rules()
        .iter()
        .map(|rule| rule.meta().name)
        .collect();
    rules.extend_from_slice(super::builtin_script_rule_names(preset));
    rules.extend_from_slice(super::builtin_css_rule_names(preset));
    rules
}

fn opt_in_rule_names() -> Vec<&'static str> {
    let mut rules: Vec<_> = RuleRegistry::with_opt_in_rules()
        .rules()
        .iter()
        .map(|rule| rule.meta().name)
        .collect();
    rules.extend_from_slice(crate::linter::script_rules::opt_in_script_rule_names());
    rules.extend_from_slice(crate::linter::musea_rules::all_builtin_musea_rule_names());
    rules
}

fn ecosystem_rule_names() -> Vec<&'static str> {
    let mut rules: Vec<_> = RuleRegistry::with_ecosystem()
        .rules()
        .iter()
        .map(|rule| rule.meta().name)
        .collect();
    rules.extend_from_slice(super::ecosystem_builtin_script_rule_names());
    rules
}

fn diagnostic_rule_names(result: &crate::LintResult) -> BTreeSet<&'static str> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name)
        .collect()
}

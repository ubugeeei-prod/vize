//! The Oxlint plugin skips plain modules for every rule that
//! `rule_runs_on_plain_scripts` rules out, so those rules must never report on
//! one.

use vize_patina::{
    LintPreset, Linter, RuleRegistry, builtin_script_rules, rule_runs_on_plain_scripts,
};

const MODULE: &str = r#"import { defineComponent, getCurrentInstance, nextTick } from "vue";

export default defineComponent({
  data() {
    return { count: 0 };
  },
  methods: {
    tick() {
      nextTick(() => {});
    },
  },
  setup() {
    const instance = getCurrentInstance();
    return { instance };
  },
});
"#;

fn diagnostics(rules: &[&str]) -> Vec<(&'static str, u32, u32)> {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(rules.iter().map(|rule| (*rule).into()).collect()))
        .lint_script(MODULE, "component.ts")
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.rule_name, diagnostic.start, diagnostic.end))
        .collect()
}

#[test]
fn rules_ruled_out_for_plain_modules_never_report_on_one() {
    let registry = RuleRegistry::with_preset(LintPreset::Opinionated);
    let script_rules = builtin_script_rules();
    let mut every: Vec<&str> = registry
        .rules()
        .iter()
        .map(|rule| rule.meta().name)
        .collect();
    every.extend(script_rules.iter().map(|rule| rule.name));
    let plain: Vec<&str> = every
        .iter()
        .copied()
        .filter(|rule| rule_runs_on_plain_scripts(rule))
        .collect();

    assert!(rule_runs_on_plain_scripts("script/no-options-api"));
    assert!(!rule_runs_on_plain_scripts("vue/no-template-shadow"));
    let reported = diagnostics(&every);
    assert!(!reported.is_empty(), "the module trips no script rule");
    assert_eq!(diagnostics(&plain), reported);
}

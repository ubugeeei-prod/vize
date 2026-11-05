//! vue/comment-directive
//!
//! Support `eslint-disable` comments in `<template>`.
//!
//! eslint-plugin-vue enables this from the base config, so every Vue 3 preset
//! includes it. The template processor applies the suppressions. This rule
//! owns the unused-directive diagnostic: an explicit disable retires that
//! report. Reporting still requires `--report-unused-disable-directives`.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <!-- eslint-disable-next-line vue/no-v-html -->
//!   <div />
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <!-- eslint-disable-next-line vue/no-v-html -->
//!   <div v-html="html" />
//! </template>
//! ```

use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

pub const RULE: &str = "vue/comment-directive";

static META: RuleMeta = RuleMeta {
    name: RULE,
    description: "Support comment directives in `<template>`",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

pub struct CommentDirective;

impl Rule for CommentDirective {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }
}

#[cfg(test)]
mod tests {
    use super::RULE;
    use crate::linter::Linter;
    use crate::preset::LintPreset;
    use crate::rule::RuleRegistry;

    const UNUSED_LISTED: &str =
        "Unused eslint-disable directive (no problems were reported from 'vue/no-v-html').";

    fn source(body: &str) -> String {
        [
            "<script setup>\nconst html = '';\n</script>\n<template>\n",
            body,
            "\n</template>\n",
        ]
        .concat()
    }

    fn messages(body: &str, report: bool, disabled: bool) -> Vec<std::string::String> {
        let mut linter = Linter::with_preset(LintPreset::Essential)
            .with_report_unused_disable_directives(report);
        if disabled {
            linter = linter.with_disabled_rules(vec![RULE.into()]);
        }
        linter
            .lint_sfc(&source(body), "App.vue")
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == RULE)
            .map(|diagnostic| diagnostic.message.to_string())
            .collect()
    }

    #[test]
    fn reports_an_unused_disable_directive() {
        let body = "  <!-- eslint-disable-next-line vue/no-v-html -->\n  <div />";
        assert_eq!(messages(body, true, false), vec![UNUSED_LISTED.to_owned()]);
    }

    #[test]
    fn allows_a_directive_that_suppresses_a_report_and_skips_when_disabled() {
        let used = "  <!-- eslint-disable-next-line vue/no-v-html -->\n  <div v-html=\"html\" />";
        assert!(messages(used, true, false).is_empty());

        let unused = "  <!-- eslint-disable-next-line vue/no-v-html -->\n  <div />";
        assert!(messages(unused, false, false).is_empty());
        assert!(messages(unused, true, true).is_empty());
    }

    #[test]
    fn belongs_to_every_vue3_preset_that_includes_base() {
        assert!(RuleRegistry::with_essential().has_rule(RULE));
        assert!(RuleRegistry::with_happy_path().has_rule(RULE));
        assert!(RuleRegistry::with_ecosystem().has_rule(RULE));
        assert!(RuleRegistry::with_preset(LintPreset::Nuxt).has_rule(RULE));
        assert!(RuleRegistry::with_preset(LintPreset::Opinionated).has_rule(RULE));
    }
}

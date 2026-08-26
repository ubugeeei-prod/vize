//! vue/no-deprecated-slot-attribute
//!
//! Disallow the deprecated `slot` attribute (removed in Vue 3).
//!
//! Vue 2.6 deprecated the `slot="name"` attribute in favour of `v-slot`, and
//! Vue 3 removed it entirely. A lingering `slot` attribute is no longer
//! interpreted as a named-slot assignment.
//!
//! This mirrors eslint-plugin-vue's `vue/no-deprecated-slot-attribute`. It is an
//! Essential rule for Vue 3 projects and is disabled for legacy Vue versions.
//!
//! ## Examples
//!
//! ### Invalid (Vue 3)
//! ```vue
//! <template>
//!   <Foo>
//!     <template slot="header"><h1>Title</h1></template>
//!   </Foo>
//! </template>
//! ```
//!
//! ### Valid (Vue 3)
//! ```vue
//! <template>
//!   <Foo>
//!     <template v-slot:header><h1>Title</h1></template>
//!   </Foo>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, PropNode};
use vize_s0::dialect::VueDialect;

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-slot-attribute",
    description: "Disallow the deprecated `slot` attribute",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow the deprecated `slot` attribute.
pub struct NoDeprecatedSlotAttribute;

impl Rule for NoDeprecatedSlotAttribute {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Only the default Vue 3 dialect removed the `slot` attribute.
        if ctx.dialect() != VueDialect::Vue {
            return;
        }

        for prop in element.props.iter() {
            if let PropNode::Attribute(attr) = prop
                && attr.name == "slot"
            {
                ctx.error_with_help(
                    ctx.t("vue/no-deprecated-slot-attribute.message"),
                    &attr.name_loc,
                    ctx.t("vue/no-deprecated-slot-attribute.help"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedSlotAttribute;
    use crate::Severity;
    use crate::linter::Linter;
    use crate::preset::LintPreset;
    use crate::rule::RuleRegistry;
    use vize_s0::config::VueVersion;

    const RULE: &str = "vue/no-deprecated-slot-attribute";

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDeprecatedSlotAttribute));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_slot_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<Foo><template slot="header">x</template></Foo>"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn default_allows_supported_slot_syntax() {
        for source in [
            r#"<template><MyPanel><template v-slot:header>header</template></MyPanel></template>"#,
            r#"<template><MyPanel><template #header>header</template></MyPanel></template>"#,
        ] {
            let result = Linter::new().lint_sfc(source, "NamedSlots.vue");
            let findings = result
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.rule_name == RULE)
                .collect::<Vec<_>>();

            assert_eq!(result.error_count, 0);
            assert_eq!(findings.len(), 0);
        }
    }

    #[test]
    fn benchmark_fixture_is_reported_by_default() {
        let source =
            "<template>\n<MyPanel>\n  <span slot=\"header\">h</span>\n</MyPanel>\n</template>";
        let result = Linter::new().lint_sfc(source, "DeprecatedSlotAttr.vue");
        let diagnostics = result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.rule_name,
                    diagnostic.severity,
                    diagnostic.message.as_str(),
                    diagnostic.start,
                    diagnostic.end,
                    diagnostic.help.as_deref(),
                    diagnostic.labels.len(),
                    diagnostic.fix.is_some(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(result.error_count, 1);
        assert_eq!(result.warning_count, 0);
        assert_eq!(
            diagnostics,
            vec![(
                RULE,
                Severity::Error,
                "the `slot` attribute was deprecated in Vue 2.6 and removed in Vue 3",
                29,
                33,
                Some("Use `v-slot` instead (e.g. `<template v-slot:header>`)."),
                0,
                false,
            )]
        );
    }

    #[test]
    fn preset_membership_matches_vue_essential() {
        let membership = LintPreset::ALL.map(|preset| {
            (
                preset.as_str(),
                RuleRegistry::with_preset(preset).has_rule(RULE),
            )
        });

        assert_eq!(
            membership,
            [
                ("happy-path", true),
                ("opinionated", true),
                ("essential", true),
                ("incremental", false),
                ("ecosystem", true),
                ("nuxt", true),
            ]
        );
        assert!(!RuleRegistry::with_opt_in_rules().has_rule(RULE));
    }

    #[test]
    fn vue2_keeps_legacy_slot_attribute() {
        let result = create_linter()
            .with_vue_version(Some(VueVersion::V2))
            .lint_template(
                r#"<Foo><span slot="header">header</span></Foo>"#,
                "Legacy.vue",
            );

        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 0);
        assert_eq!(result.diagnostics.len(), 0);
    }
}

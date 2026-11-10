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
//!     <div :slot="name">Title</div>
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
use crate::ir::ByteRange;
use crate::markup::{MarkupAttribute, MarkupContext, MarkupDirective, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::dialect::VueDialect;
use vize_relief::{DirectiveNode, ElementNode};

mod native;

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-slot-attribute",
    description: "Disallow the deprecated `slot` attribute",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow the deprecated `slot` attribute.
pub struct NoDeprecatedSlotAttribute;

impl NoDeprecatedSlotAttribute {
    fn report(ctx: &mut LintContext<'_>, range: ByteRange) {
        ctx.error_at_with_help(
            ctx.t("vue/no-deprecated-slot-attribute.message"),
            range,
            ctx.t("vue/no-deprecated-slot-attribute.help"),
        );
    }

    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        if ctx.dialect() != VueDialect::Vue {
            return;
        }

        element.walk_attributes(&mut |attribute: MarkupAttribute<'_>| {
            if attribute.name() == "slot" {
                Self::report(ctx, attribute.name_range());
            }
        });
    }

    fn check_directive(ctx: &mut LintContext<'_>, directive: &MarkupDirective<'_>) {
        if ctx.dialect() != VueDialect::Vue
            || !directive.name_eq("bind")
            || !directive.is_static_arg_exact("slot")
        {
            return;
        }

        // Report the authored directive key, including its prefix/modifiers.
        // Dot shorthand has a synthesized prop modifier at the prefix only.
        let start = directive.range().start;
        let Some(argument) = directive.argument_range() else {
            return;
        };
        let mut end = argument.end;
        directive.walk_modifier_ranges(&mut |_modifier, range| {
            end = end.max(range.end);
        });
        Self::report(ctx, ByteRange::new(start, end));
    }
}

impl MarkupRule for NoDeprecatedSlotAttribute {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check_element(ctx.lint(), element);
    }

    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        _element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
        Self::check_directive(ctx.lint(), directive);
    }
}

impl Rule for NoDeprecatedSlotAttribute {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_native_sfc_template_rule(
        &self,
    ) -> Option<&dyn crate::native::sfc::NativeSfcTemplateRule> {
        Some(self)
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check_element(ctx, &MarkupElement::new(element));
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        Self::check_directive(ctx, &MarkupDirective::from_relief(directive));
    }
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedSlotAttribute;
    use crate::Severity;
    use crate::linter::Linter;
    use crate::preset::LintPreset;
    use crate::rule::RuleRegistry;
    use vize_l0::config::VueVersion;

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

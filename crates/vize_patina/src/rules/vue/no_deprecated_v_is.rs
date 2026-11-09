//! vue/no-deprecated-v-is
//!
//! Disallow the deprecated `v-is` directive.
//!
//! Vue 3.1 deprecated `v-is` in favor of the `is` attribute with the `vue:`
//! prefix.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <tr v-is="'MyRow'"></tr>
//! ```
//!
//! ### Valid
//! ```vue
//! <tr is="vue:MyRow"></tr>
//! <component :is="MyRow" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::markup::{MarkupContext, MarkupDirective, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::dialect::VueDialect;
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-v-is",
    description: "Disallow the deprecated `v-is` directive",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow the deprecated `v-is` directive.
pub struct NoDeprecatedVIs;

impl NoDeprecatedVIs {
    fn check(ctx: &mut LintContext<'_>, directive: &MarkupDirective<'_>) {
        // Only the default Vue 3 dialect ever shipped `v-is` as a built-in.
        if !directive.name_eq("is") || ctx.dialect() != VueDialect::Vue {
            return;
        }

        // Report the directive key (`v-is`, with any argument and modifiers)
        // rather than its value, as eslint-plugin-vue does: it ends at the last
        // modifier, else at the argument, else after the bare name.
        let range = directive.range();
        let mut end = None;
        directive.walk_modifier_ranges(&mut |_, modifier| end = Some(modifier.end));
        let end = end
            .or_else(|| directive.argument_range().map(|argument| argument.end))
            .unwrap_or(range.start + "v-is".len() as u32);
        ctx.report(
            LintDiagnostic::warn(
                META.name,
                ctx.t("vue/no-deprecated-v-is.message").as_ref(),
                range.start,
                end,
            )
            .with_help(ctx.t("vue/no-deprecated-v-is.help").as_ref()),
        );
    }
}

impl MarkupRule for NoDeprecatedVIs {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        _element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
        Self::check(ctx.lint(), directive);
    }
}

impl Rule for NoDeprecatedVIs {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        Self::check(ctx, &MarkupDirective::from_relief(directive));
    }
}

#[cfg(test)]
mod tests {
    //! Cases mirror eslint-plugin-vue's `no-deprecated-v-is` suite. Templates are
    //! written at the `lint_template` call so the markup differential battery
    //! picks them up as fixtures.

    use super::NoDeprecatedVIs;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDeprecatedVIs));
        Linter::with_registry(registry)
    }

    #[test]
    fn accepts_is_attribute_with_vue_prefix() {
        let result = linter().lint_template(r#"<div is="vue:MyComponent" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn accepts_dynamic_component_shorthand() {
        let result = linter().lint_template(r#"<component :is="MyComponent" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn accepts_dynamic_component_longhand() {
        let result = linter().lint_template(r#"<component v-bind:is="MyComponent" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_v_is_at_the_directive_key() {
        let source = r#"<div v-is="'MyComponent'" />"#;
        let result = linter().lint_template(r#"<div v-is="'MyComponent'" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        let diagnostic = &result.diagnostics[0];
        assert_eq!(diagnostic.rule_name, "vue/no-deprecated-v-is");
        assert_eq!(diagnostic.message, "`v-is` directive is deprecated.");
        assert_eq!(
            source.get(diagnostic.start as usize..diagnostic.end as usize),
            Some("v-is")
        );
    }

    #[test]
    fn reports_v_is_without_value() {
        let result = linter().lint_template(r#"<div v-is />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_each_v_is() {
        let result = linter().lint_template(
            r#"<table><tr v-is="'A'"></tr><tr v-is="'B'"></tr></table>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 2);
    }
}

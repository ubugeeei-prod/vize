//! vue/valid-v-is
//!
//! Enforce valid `v-is` directives.
//!
//! `v-is` (Vue 3.0, deprecated in 3.1) must:
//! - Have a value, and no argument or modifier
//! - Sit on a native HTML element, not on a component
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <tr v-is:arg="'MyRow'"></tr>
//! <tr v-is.mod="'MyRow'"></tr>
//! <tr v-is></tr>
//! <MyComponent v-is="'MyRow'" />
//! ```
//!
//! ### Valid
//! ```vue
//! <tr v-is="'MyRow'"></tr>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::dialect::VueDialect;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, Namespace};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-is",
    description: "Enforce valid `v-is` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-is directives
pub struct ValidVIs;

impl Rule for ValidVIs {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        // `v-is` is a built-in directive only in the default Vue 3 dialect.
        if directive.name != "is" || ctx.dialect() != VueDialect::Vue {
            return;
        }

        if let Some(arg) = &directive.arg {
            let loc = match arg {
                ExpressionNode::Simple(simple) => &simple.loc,
                ExpressionNode::Compound(compound) => &compound.loc,
            };
            ctx.warn_with_help(
                ctx.t("vue/valid-v-is.unexpected_argument"),
                loc,
                ctx.t("vue/valid-v-is.help"),
            );
        }

        if let (Some(first), Some(last)) = (directive.modifiers.first(), directive.modifiers.last())
        {
            ctx.report(
                LintDiagnostic::warn(
                    META.name,
                    ctx.t("vue/valid-v-is.unexpected_modifier").as_ref(),
                    first.loc.span.start,
                    last.loc.span.end,
                )
                .with_help(ctx.t("vue/valid-v-is.help").as_ref()),
            );
        }

        if !has_value(&directive.exp) {
            ctx.warn_with_help(
                ctx.t("vue/valid-v-is.expected_value"),
                &directive.loc,
                ctx.t("vue/valid-v-is.help"),
            );
        }

        // SVG and MathML owners are accepted; an HTML-namespace owner must be
        // a well-known HTML element rather than a component or custom element.
        if element.ns == Namespace::Html && !vize_l0::is_html_tag(element.tag) {
            let message = ctx.t_fmt(
                "vue/valid-v-is.owner_must_be_html_element",
                &[("name", element.tag)],
            );
            ctx.warn_with_help(message, &directive.loc, ctx.t("vue/valid-v-is.help"));
        }
    }
}

fn has_value(exp: &Option<ExpressionNode>) -> bool {
    match exp {
        Some(ExpressionNode::Simple(simple)) => !simple.content.trim().is_empty(),
        Some(ExpressionNode::Compound(compound)) => !compound.children.is_empty(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    //! Cases mirror eslint-plugin-vue's `valid-v-is` suite. Templates are written
    //! at the `lint_template` call so the markup differential battery picks them
    //! up as fixtures.

    use super::ValidVIs;
    use crate::linter::{LintResult, Linter};
    use crate::rule::RuleRegistry;

    fn linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVIs));
        Linter::with_registry(registry)
    }

    fn messages(result: &LintResult) -> Vec<&str> {
        result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                assert_eq!(diagnostic.rule_name, "vue/valid-v-is");
                diagnostic.message.as_str()
            })
            .collect()
    }

    fn reported<'a>(source: &'a str, result: &LintResult) -> Vec<&'a str> {
        result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                source
                    .get(diagnostic.start as usize..diagnostic.end as usize)
                    .unwrap_or_default()
            })
            .collect()
    }

    const EXPECTED_VALUE: &str = "'v-is' directives require that attribute value.";

    #[test]
    fn accepts_native_element_with_value() {
        let result = linter().lint_template(r#"<div v-is="foo" />"#, "test.vue");
        assert!(messages(&result).is_empty());
    }

    #[test]
    fn accepts_table_row_with_value() {
        let result =
            linter().lint_template(r#"<table><tr v-is="'MyRow'"></tr></table>"#, "test.vue");
        assert!(messages(&result).is_empty());
    }

    #[test]
    fn accepts_svg_owner() {
        let result = linter().lint_template(r#"<svg><g v-is="foo" /></svg>"#, "test.vue");
        assert!(messages(&result).is_empty());
    }

    #[test]
    fn ignores_the_is_attribute() {
        let result = linter().lint_template(r#"<MyComponent is="vue:Other" />"#, "test.vue");
        assert!(messages(&result).is_empty());
    }

    #[test]
    fn reports_argument_at_the_argument() {
        let source = r#"<div v-is:a="foo" />"#;
        let result = linter().lint_template(r#"<div v-is:a="foo" />"#, "test.vue");
        assert_eq!(
            messages(&result),
            ["'v-is' directives require no argument."]
        );
        assert_eq!(reported(source, &result), ["a"]);
    }

    #[test]
    fn reports_all_modifiers_once() {
        let source = r#"<div v-is.a.b="foo" />"#;
        let result = linter().lint_template(r#"<div v-is.a.b="foo" />"#, "test.vue");
        assert_eq!(
            messages(&result),
            ["'v-is' directives require no modifier."]
        );
        assert_eq!(reported(source, &result), ["a.b"]);
    }

    #[test]
    fn reports_missing_value() {
        let result = linter().lint_template(r#"<div v-is />"#, "test.vue");
        assert_eq!(messages(&result), [EXPECTED_VALUE]);
    }

    #[test]
    fn reports_empty_value() {
        let result = linter().lint_template(r#"<div v-is="" />"#, "test.vue");
        assert_eq!(messages(&result), [EXPECTED_VALUE]);
    }

    #[test]
    fn reports_component_owner() {
        let result = linter().lint_template(r#"<MyComponent v-is="foo" />"#, "test.vue");
        assert_eq!(
            messages(&result),
            ["'v-is' directive must be owned by a native HTML element, but 'MyComponent' is not."]
        );
    }

    #[test]
    fn reports_custom_element_owner() {
        let result = linter().lint_template(r#"<my-element v-is="foo" />"#, "test.vue");
        assert_eq!(
            messages(&result),
            ["'v-is' directive must be owned by a native HTML element, but 'my-element' is not."]
        );
    }
}

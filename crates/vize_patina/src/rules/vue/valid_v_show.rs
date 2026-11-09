//! vue/valid-v-show
//!
//! Enforce valid `v-show` directives.
//!
//! `v-show` must:
//! - Have an expression
//! - Not be on `<template>` element
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-show></div>
//! <template v-show="foo"><div></div></template>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-show="foo"></div>
//! <div v-show="foo > 0"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupDirective, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-show",
    description: "Enforce valid `v-show` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-show directives
pub struct ValidVShow;

impl ValidVShow {
    fn check(ctx: &mut LintContext<'_>, tag: &str, directive: &MarkupDirective<'_>) {
        if !directive.name_eq("show") {
            return;
        }
        let help = ctx.t("vue/valid-v-show.help");
        if !directive.has_value() {
            ctx.error_at_with_help(
                ctx.t("vue/valid-v-show.missing_expression"),
                directive.range(),
                help,
            );
            return;
        }
        if let Some(argument) = directive.argument_range() {
            ctx.error_at_with_help(
                ctx.t("vue/valid-v-show.unexpected_argument"),
                argument,
                help.clone(),
            );
        }
        directive.walk_modifier_ranges(&mut |_, modifier| {
            ctx.error_at_with_help(
                ctx.t("vue/valid-v-show.unexpected_modifier"),
                modifier,
                help.clone(),
            );
        });
        if tag == "template" {
            ctx.error_at_with_help(
                ctx.t("vue/valid-v-show.on_template"),
                directive.range(),
                help,
            );
        }
    }
}

impl MarkupRule for ValidVShow {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
        Self::check(ctx.lint(), element.tag(), directive);
    }
}

impl Rule for ValidVShow {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        Self::check(ctx, element.tag, &MarkupDirective::from_relief(directive));
    }
}

#[cfg(test)]
mod tests {
    use super::ValidVShow;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVShow));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_show() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-show="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_show_no_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-show></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_show_with_argument() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-show:foo="visible"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_show_with_modifier() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-show.foo="visible"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_show_on_template() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<template v-show="foo"><div></div></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }
}

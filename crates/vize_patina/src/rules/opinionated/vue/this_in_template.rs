//! vue/this-in-template
//!
//! Disallow `this.` in template expressions.
//!
//! Vue resolves template identifiers against the component instance
//! automatically, so writing `this.` in a template expression is unnecessary
//! and is frequently a mistake (for example, copy-pasting code out of a method
//! body). This rule flags member access on `this` inside both interpolations
//! and directive expressions.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div>{{ this.message }}</div>
//! <div :class="this.className"></div>
//! <button @click="this.handleClick()"></button>
//! ```
//!
//! ### Valid
//! ```vue
//! <div>{{ message }}</div>
//! <div :class="className"></div>
//! <button @click="handleClick()"></button>
//! <div>{{ 'this.is.a.string' }}</div>
//! ```

mod expressions;
mod origin;

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l1::markup::entity::EntityContext;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, InterpolationNode};

static META: RuleMeta = RuleMeta {
    name: "vue/this-in-template",
    description: "Disallow `this.` in template expressions",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow `this.` in template expressions.
pub struct ThisInTemplate;

impl Rule for ThisInTemplate {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
        expressions::check(ctx, &interpolation.content, None, EntityContext::Text);
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if let Some(exp) = &directive.exp {
            expressions::check(ctx, exp, Some(directive.name), EntityContext::Attribute);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ThisInTemplate;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ThisInTemplate));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_interpolation_without_this() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ message }}</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_directive_without_this() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="className"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_this_inside_string_literal() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ 'this.is.a.string' }}</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_identifier_containing_this_substring() {
        let linter = create_linter();
        // `things` and `myThis` contain "this" but are not the `this` keyword.
        let result = linter.lint_template(r#"<div>{{ things.length + myThis }}</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_this_in_interpolation() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ this.message }}</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_this_in_bind_directive() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="this.className"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_this_in_event_handler() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<button @click="this.handleClick()"></button>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_this_with_whitespace_before_dot() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ this .message }}</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }
}

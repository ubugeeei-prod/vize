//! vue/valid-v-html
//!
//! Enforce valid `v-html` directives.
//!
//! `v-html` must have an expression and must not have an argument or a
//! modifier. This validates the directive's *syntax*; the security concern
//! of using `v-html` at all is handled separately by `vue/no-v-html`.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-html></div>
//! <div v-html:arg="foo"></div>
//! <div v-html.mod="foo"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-html="html"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupDirective, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::vue::directive_shape::{Shape, Value};
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-html",
    description: "Enforce valid `v-html` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

static SHAPE: Shape = Shape {
    name: "html",
    value: Value::Required,
    value_message: "vue/valid-v-html.missing_expression",
    argument_message: "vue/valid-v-html.unexpected_argument",
    modifier_message: "vue/valid-v-html.unexpected_modifier",
    help: "vue/valid-v-html.help",
};

/// Enforce valid v-html directives
pub struct ValidVHtml;

impl MarkupRule for ValidVHtml {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        _element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
        SHAPE.check(ctx.lint(), directive);
    }
}

impl Rule for ValidVHtml {
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
        SHAPE.check(ctx, &MarkupDirective::from_relief(directive));
    }
}

#[cfg(test)]
mod tests {
    use super::ValidVHtml;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVHtml));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_html() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-html="html"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_html_missing_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-html></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_html_with_argument() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-html:foo="html"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_html_with_modifier() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-html.foo="html"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}

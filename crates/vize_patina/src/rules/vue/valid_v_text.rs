//! vue/valid-v-text
//!
//! Enforce valid `v-text` directives.
//!
//! `v-text` must have an expression and must not have an argument or a
//! modifier.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-text></div>
//! <div v-text:arg="foo"></div>
//! <div v-text.mod="foo"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-text="msg"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupDirective, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::vue::directive_shape::{Shape, Value};
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-text",
    description: "Enforce valid `v-text` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

static SHAPE: Shape = Shape {
    name: "text",
    value: Value::Required,
    value_message: "vue/valid-v-text.missing_expression",
    argument_message: "vue/valid-v-text.unexpected_argument",
    modifier_message: "vue/valid-v-text.unexpected_modifier",
    help: "vue/valid-v-text.help",
};

/// Enforce valid v-text directives
pub struct ValidVText;

impl MarkupRule for ValidVText {
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

impl Rule for ValidVText {
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
    use super::ValidVText;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVText));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_text() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-text="msg"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_text_missing_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-text></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_text_with_argument() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-text:foo="msg"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_text_with_modifier() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-text.foo="msg"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}

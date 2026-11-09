//! vue/valid-v-once
//!
//! Enforce valid `v-once` directives.
//!
//! `v-once` is a bare marker directive: it must not have an expression,
//! an argument, or a modifier.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-once="foo"></div>
//! <div v-once:arg></div>
//! <div v-once.mod></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-once></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupDirective, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::vue::directive_shape::{Shape, Value};
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-once",
    description: "Enforce valid `v-once` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

static SHAPE: Shape = Shape {
    name: "once",
    value: Value::Forbidden,
    value_message: "vue/valid-v-once.unexpected_value",
    argument_message: "vue/valid-v-once.unexpected_argument",
    modifier_message: "vue/valid-v-once.unexpected_modifier",
    help: "vue/valid-v-once.help",
};

/// Enforce valid v-once directives
pub struct ValidVOnce;

impl MarkupRule for ValidVOnce {
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

impl Rule for ValidVOnce {
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
    use super::ValidVOnce;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVOnce));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_once() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-once></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_once_with_value() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-once="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_once_with_argument() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-once:foo></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_once_with_modifier() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-once.foo></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}

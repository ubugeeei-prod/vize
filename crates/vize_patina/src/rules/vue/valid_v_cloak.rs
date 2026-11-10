//! vue/valid-v-cloak
//!
//! Enforce valid `v-cloak` directives.
//!
//! `v-cloak` is a bare marker directive: it must not have an expression,
//! an argument, or a modifier.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-cloak="foo"></div>
//! <div v-cloak:arg></div>
//! <div v-cloak.mod></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-cloak></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupDirective, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::vue::directive_shape::{Shape, Value};
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-cloak",
    description: "Enforce valid `v-cloak` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

static SHAPE: Shape = Shape {
    name: "cloak",
    value: Value::Forbidden,
    value_message: "vue/valid-v-cloak.unexpected_value",
    argument_message: "vue/valid-v-cloak.unexpected_argument",
    modifier_message: "vue/valid-v-cloak.unexpected_modifier",
    help: "vue/valid-v-cloak.help",
};

/// Enforce valid v-cloak directives
pub struct ValidVCloak;

impl MarkupRule for ValidVCloak {
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

impl Rule for ValidVCloak {
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
    use super::ValidVCloak;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVCloak));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_cloak() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-cloak></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_cloak_with_value() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-cloak="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_cloak_with_argument() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-cloak:foo></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_cloak_with_modifier() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-cloak.foo></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}

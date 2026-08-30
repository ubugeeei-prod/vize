//! vue/no-textarea-mustache
//!
//! Disallow mustache interpolation in `<textarea>`.
//!
//! Mustache interpolation in `<textarea>` doesn't work correctly in Vue.
//! Use `v-model` instead.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <textarea>{{ message }}</textarea>
//! ```
//!
//! ### Valid
//! ```vue
//! <textarea v-model="message"></textarea>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupElement, MarkupNode, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-textarea-mustache",
    description: "Disallow mustache interpolation in `<textarea>`",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow mustache in textarea
pub struct NoTextareaMustache;

impl NoTextareaMustache {
    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        // Preserve the legacy exact lowercase tag check. In JSX, capitalized
        // `<Textarea>` remains a component and must not be treated as native.
        if !element.is_unqualified_tag_exact("textarea") {
            return;
        }

        element.walk_children(&mut |child| {
            if let MarkupNode::Interpolation(range) = child {
                ctx.error_at_with_help(
                    ctx.t("vue/no-textarea-mustache.message"),
                    range,
                    ctx.t("vue/no-textarea-mustache.help"),
                );
            }
        });
    }
}

impl MarkupRule for NoTextareaMustache {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check_element(ctx.lint(), element);
    }
}

impl Rule for NoTextareaMustache {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn jsx_needs_lowering(&self) -> bool {
        true
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check_element(ctx, &MarkupElement::new(element));
    }
}

#[cfg(test)]
mod tests {
    use super::NoTextareaMustache;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoTextareaMustache));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_model() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<textarea v-model="message"></textarea>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_mustache() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<textarea>{{ message }}</textarea>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_mustache_in_div() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ message }}</div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }
}

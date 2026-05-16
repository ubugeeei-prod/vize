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
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{ElementNode, TemplateChildNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-textarea-mustache",
    description: "Disallow mustache interpolation in `<textarea>`",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow mustache in textarea
pub struct NoTextareaMustache;

impl Rule for NoTextareaMustache {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Only check <textarea> elements
        if element.tag.as_str() != "textarea" {
            return;
        }

        // Check for interpolation in children
        for child in element.children.iter() {
            if let TemplateChildNode::Interpolation(interp) = child {
                ctx.error_with_help(
                    ctx.t("vue/no-textarea-mustache.message"),
                    &interp.loc,
                    ctx.t("vue/no-textarea-mustache.help"),
                );
            }
        }
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

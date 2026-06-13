//! vue/no-v-text
//!
//! Disallow the `v-text` directive; prefer mustache interpolation `{{ }}` for
//! text content.
//!
//! `v-text` sets an element's `textContent`, which is exactly what mustache
//! interpolation does. Mustache is the idiomatic Vue way to render text: it
//! reads more naturally in the template, supports filters/expressions inline,
//! and keeps the element's existing children visible in source.
//!
//! This is distinct from `vue/valid-v-text`, which only validates the syntax of
//! `v-text` *when it is used*. This rule discourages using it at all.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-text="message"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div>{{ message }}</div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-v-text",
    description: "Disallow the v-text directive; prefer mustache interpolation",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow the `v-text` directive.
pub struct NoVText;

impl Rule for NoVText {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "text" {
            return;
        }

        ctx.warn_with_help(
            ctx.t("vue/no-v-text.message"),
            &directive.loc,
            ctx.t("vue/no-v-text.help"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::NoVText;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoVText));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_v_text_warns() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-text="message"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_mustache_is_allowed() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ message }}</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_other_directive_is_allowed() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-html="message"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_plain_element_is_allowed() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="box"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}

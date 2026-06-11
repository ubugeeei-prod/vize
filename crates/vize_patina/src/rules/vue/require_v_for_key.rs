//! vue/require-v-for-key
//!
//! Require `v-bind:key` with `v-for` directives.
//!
//! This rule reports elements using `v-for` without a `:key` attribute.
//! The key attribute is essential for Vue's virtual DOM diffing algorithm
//! to efficiently update the DOM when the list changes.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <li v-for="item in items">{{ item }}</li>
//! ```
//!
//! ### Valid
//! ```vue
//! <li v-for="item in items" :key="item.id">{{ item }}</li>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{DirectiveNode, ElementNode, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/require-v-for-key",
    description: "Require `v-bind:key` with `v-for` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Require v-bind:key with v-for directives
pub struct RequireVForKey;

impl Rule for RequireVForKey {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        // Only check v-for directives
        if directive.name.as_str() != "for" {
            return;
        }

        // petite-vue does not require a :key on v-for, so this Vue-3-only rule
        // must not fire on petite-vue documents.
        if ctx.is_petite_vue() {
            return;
        }

        // Skip <template> tags - key should be on children instead
        // (though on <template v-for>, the key can be on the template itself)
        if element.tag.as_str() == "template" {
            // For <template v-for>, we still require a key if it has meaningful content
            // But we'll be lenient here since the pattern varies
            return;
        }

        // Check if element has :key or key attribute
        let has_key = element.props.iter().any(|prop| match prop {
            PropNode::Attribute(attr) => attr.name.as_str() == "key",
            PropNode::Directive(dir) => {
                // Check for v-bind:key or :key
                if dir.name.as_str() == "bind"
                    && let Some(ExpressionNode::Simple(s)) = &dir.arg
                {
                    return s.content.as_str() == "key";
                }
                false
            }
        });

        if !has_key {
            ctx.error_with_help(
                ctx.t_fmt(
                    "vue/require-v-for-key.message",
                    &[("tag", element.tag.as_str())],
                ),
                &directive.loc,
                ctx.t("vue/require-v-for-key.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RequireVForKey;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(RequireVForKey));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_for_with_key() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<ul><li v-for="item in items" :key="item.id">{{ item.name }}</li></ul>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_for_without_key() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<ul><li v-for="item in items">{{ item.name }}</li></ul>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_valid_v_for_with_static_key() {
        let linter = create_linter();
        // Static key is unusual but technically valid
        let result = linter.lint_template(
            r#"<div v-for="item in items" key="static"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_petite_vue_keyless_v_for_allowed() {
        let linter = create_linter();
        // Structurally detected petite-vue document (script src resolves to the
        // petite-vue package). petite-vue allows keyless v-for.
        let result = linter.lint_standalone_html(
            r#"<!DOCTYPE html>
<html>
  <body>
    <ul v-scope="{ items: [1, 2, 3] }">
      <li v-for="item in items">{{ item }}</li>
    </ul>
    <script src="https://unpkg.com/petite-vue" init></script>
  </body>
</html>"#,
            "index.html",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_non_petite_html_keyless_v_for_still_reports() {
        let linter = create_linter();
        // A plain HTML document (no petite-vue) keeps the Vue-3 requirement.
        let result = linter.lint_standalone_html(
            r#"<!DOCTYPE html>
<html>
  <body>
    <ul>
      <li v-for="item in items">{{ item }}</li>
    </ul>
    <script src="https://unpkg.com/vue"></script>
  </body>
</html>"#,
            "index.html",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_template_v_for_ignored() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<template v-for="item in items"><div :key="item.id">{{ item }}</div></template>"#,
            "test.vue",
        );
        // <template> itself doesn't need key, but children should
        assert_eq!(result.error_count, 0);
    }
}

//! vue/no-v-for-template-key-on-child
//!
//! Disallow `key` on the child of a `<template v-for>`.
//!
//! In Vue 3 the `key` for a `<template v-for>` must live on the `<template>`
//! element itself, not on its child. Vue 2 placed the key on the child; Vue 3
//! reversed this. A `:key`/`v-bind:key` left on the child is therefore a bug.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template v-for="item in items">
//!   <div :key="item.id">{{ item }}</div>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template v-for="item in items" :key="item.id">
//!   <div>{{ item }}</div>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ExpressionNode, PropNode, TemplateChildNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-v-for-template-key-on-child",
    description: "Disallow `key` on the child of a `<template v-for>`",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow `:key` on the child of a `<template v-for>`.
pub struct NoVForTemplateKeyOnChild;

impl NoVForTemplateKeyOnChild {
    /// Whether `element` carries a `v-for` directive.
    fn has_v_for(element: &ElementNode) -> bool {
        element.props.iter().any(|prop| match prop {
            PropNode::Directive(dir) => dir.name.as_str() == "for",
            PropNode::Attribute(_) => false,
        })
    }

    /// Return the `:key`/`v-bind:key` directive on `element`, if any.
    ///
    /// Only the bound form is considered: the Vue 3 breaking change is about the
    /// dynamic `key` binding migrating from the child up to the `<template>`.
    fn key_binding<'a, 'b>(element: &'b ElementNode<'a>) -> Option<&'b PropNode<'a>> {
        element.props.iter().find(|prop| match prop {
            PropNode::Directive(dir) => {
                dir.name.as_str() == "bind"
                    && matches!(
                        dir.arg.as_ref(),
                        Some(ExpressionNode::Simple(s)) if s.content.as_str() == "key"
                    )
            }
            PropNode::Attribute(_) => false,
        })
    }
}

impl Rule for NoVForTemplateKeyOnChild {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Only `<template v-for>` carries its key on itself; everything else is
        // free to key its children however it likes.
        if element.tag.as_str() != "template" || !Self::has_v_for(element) {
            return;
        }

        for child in &element.children {
            let TemplateChildNode::Element(child) = child else {
                continue;
            };
            if let Some(prop) = Self::key_binding(child) {
                ctx.error_with_help(
                    ctx.t("vue/no-v-for-template-key-on-child.message"),
                    prop.loc(),
                    ctx.t("vue/no-v-for-template-key-on-child.help"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoVForTemplateKeyOnChild;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoVForTemplateKeyOnChild));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_invalid_key_on_child() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<template v-for="item in items"><div :key="item.id">{{ item }}</div></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_v_bind_key_on_child() {
        let linter = create_linter();
        // The long-hand `v-bind:key` form is equivalent to `:key`.
        let result = linter.lint_template(
            r#"<template v-for="item in items"><div v-bind:key="item.id">{{ item }}</div></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_key_on_template() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<template v-for="item in items" :key="item.id"><div>{{ item }}</div></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_key_on_element_v_for() {
        let linter = create_linter();
        // A non-template `v-for` keeps its key on the repeated element itself.
        let result = linter.lint_template(
            r#"<div v-for="item in items" :key="item.id">{{ item }}</div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_template_without_v_for() {
        let linter = create_linter();
        // A plain `<template>` (no v-for) does not own its child's key.
        let result = linter.lint_template(
            r#"<template v-if="show"><div :key="item.id">{{ item }}</div></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_no_key_anywhere() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<template v-for="item in items"><div>{{ item }}</div></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_static_key_on_child_ignored() {
        let linter = create_linter();
        // Only the bound `:key` is the migration target; a static `key="..."`
        // attribute is left to other rules.
        let result = linter.lint_template(
            r#"<template v-for="item in items"><div key="static">{{ item }}</div></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }
}

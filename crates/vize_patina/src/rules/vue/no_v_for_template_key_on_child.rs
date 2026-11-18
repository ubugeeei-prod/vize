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
use crate::markup::{
    MarkupBindingKind, MarkupContext, MarkupElement, MarkupNode, MarkupRule, MarkupTraversal,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

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
    /// Whether `element` carries an authored directive called `name`.
    fn has_directive(element: &MarkupElement<'_>, name: &str) -> bool {
        let mut found = false;
        element.walk_authored_directive_ranges(name, &mut |_| found = true);
        found
    }

    /// Vue's compiler checks this rule after `v-if` / `v-for` transforms, so a
    /// child with its own structural directive is no longer a plain direct
    /// element child for this diagnostic.
    fn has_child_control_flow(element: &MarkupElement<'_>) -> bool {
        ["if", "else", "else-if", "for"]
            .into_iter()
            .any(|name| Self::has_directive(element, name))
    }

    /// The range of the first `:key`/`v-bind:key` binding on `element`.
    ///
    /// Only the bound form is considered: the Vue 3 breaking change is about the
    /// dynamic `key` binding migrating from the child up to the `<template>`.
    fn key_binding(element: &MarkupElement<'_>) -> Option<crate::ir::ByteRange> {
        let mut key = None;
        element.walk_bindings(&mut |binding| {
            if key.is_none()
                && binding.kind() == MarkupBindingKind::Bind
                && binding.is_unqualified_arg_exact("key")
            {
                key = Some(binding.range());
            }
        });
        key
    }

    fn check(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        // Only `<template v-for>` carries its key on itself; everything else is
        // free to key its children however it likes.
        if !element.is_unqualified_tag_exact("template") || !Self::has_directive(element, "for") {
            return;
        }

        // Once the repeated template owns its key, a keyed dynamic child is
        // not the Vue 2 placement mistake this migration rule targets.
        // eslint-plugin-vue and the Vue compiler both restrict the diagnostic
        // to the case where the key was left on the child instead of moved.
        if Self::key_binding(element).is_some() {
            return;
        }

        element.walk_children(&mut |child| {
            let MarkupNode::Element(child) = child else {
                return;
            };
            if Self::has_child_control_flow(&child) {
                return;
            }
            if let Some(range) = Self::key_binding(&child) {
                ctx.error_at_with_help(
                    ctx.t("vue/no-v-for-template-key-on-child.message"),
                    range,
                    ctx.t("vue/no-v-for-template-key-on-child.help"),
                );
            }
        });
    }
}

impl MarkupRule for NoVForTemplateKeyOnChild {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check(ctx.lint(), element);
    }
}

impl Rule for NoVForTemplateKeyOnChild {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check(ctx, &MarkupElement::new(element));
    }
}

#[cfg(test)]
mod tests {
    use super::NoVForTemplateKeyOnChild;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;
    use vize_l0::config::VueVersion;

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
    fn test_valid_key_on_template_and_dynamic_component_child() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<template v-for="name in names" :key="name"><component :is="name" :key="name" /></template>"#,
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
    fn test_valid_key_on_child_with_own_control_flow() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"
            <template v-for="row in rows" :key="row">
              <li v-for="col in cols" :key="col">{{ col }}</li>
            </template>
            <template v-for="item in items">
              <span v-if="item.visible" :key="item.id" />
            </template>
            <template v-for="x in list">
              <span v-if="x.a" :key="x.k1" />
              <span v-else-if="x.b" :key="x.k2" />
              <span v-else :key="x.k3" />
            </template>
            "#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_reports_only_plain_child_key_when_mixed_with_control_flow_children() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"
            <template v-for="item in items">
              <span v-for="child in item.children" :key="child.id" />
              <span v-if="item.visible" :key="item.id" />
              <span :key="item.fallbackId" />
            </template>
            "#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
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

    #[test]
    fn vue2_allows_key_on_template_v_for_child() {
        let linter = create_linter().with_vue_version(Some(VueVersion::V2));
        let result = linter.lint_template(
            r#"<template v-for="item in items"><div :key="item.id">{{ item }}</div></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }
}

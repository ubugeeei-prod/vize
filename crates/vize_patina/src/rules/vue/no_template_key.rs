//! vue/no-template-key
//!
//! Disallow `key` attribute on `<template>`.
//!
//! Vue does not allow `key` attribute on `<template>` elements.
//! Use `key` on real elements or components instead.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template :key="section">
//!   <div>{{ item }}</div>
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
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-template-key",
    description: "Disallow `key` attribute on `<template>`",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow a `key` attribute on `<template>`.
pub struct NoTemplateKey;

fn check(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
    // Only check <template> elements
    if !element.is_unqualified_tag_exact("template") {
        return;
    }

    let mut has_v_for = false;
    element.walk_authored_directive_ranges("for", &mut |_| has_v_for = true);
    if has_v_for {
        return;
    }

    // Check for key attribute or :key directive
    element.walk_bindings(&mut |binding| {
        let is_key = match binding.kind() {
            MarkupBindingKind::Attribute => binding.is_unqualified_arg_exact("key"),
            MarkupBindingKind::Bind => binding.is_static_unqualified_arg_exact("key"),
            MarkupBindingKind::On | MarkupBindingKind::Model | MarkupBindingKind::Custom => false,
        };
        if is_key {
            ctx.error_at_with_help(
                ctx.t("vue/no-template-key.message"),
                binding.range(),
                ctx.t("vue/no-template-key.help"),
            );
        }
    });
}

impl MarkupRule for NoTemplateKey {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        check(ctx.lint(), element);
    }
}

impl Rule for NoTemplateKey {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        check(ctx, &MarkupElement::new(element));
    }
}

#[cfg(test)]
mod tests {
    use super::NoTemplateKey;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoTemplateKey));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_no_key_on_template() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<template v-for="item in items"><div :key="item.id">{{ item }}</div></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_key_on_template() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<template :key="section"><div>{{ item }}</div></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_key_on_template_v_for() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<template v-for="item in items" :key="item.id"><div>{{ item }}</div></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_key_on_div() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item in items" :key="item.id">{{ item }}</div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn dynamic_argument_is_not_a_key() {
        let result = create_linter().lint_template(
            r#"<div><template v-if="show" :[key]="value"><p /></template></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
    }
}

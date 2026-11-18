//! vue/no-lone-template
//!
//! Disallow unnecessary `<template>` elements.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <div>content</div>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template v-if="condition">
//!   <div>content</div>
//! </template>
//! <template v-for="item in items">
//!   <div>{{ item }}</div>
//! </template>
//! <template #header>
//!   <div>Header</div>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-lone-template",
    description: "Disallow unnecessary `<template>` elements",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// No lone template rule
#[derive(Default)]
pub struct NoLoneTemplate;

impl NoLoneTemplate {
    /// Check if the template has a valid directive that justifies its existence
    fn has_valid_directive(element: &MarkupElement<'_>) -> bool {
        let mut valid = false;
        for name in [
            "if",
            "else-if",
            "else",
            "for",
            "slot",
            "slot-scope",
            "scope",
        ] {
            element.walk_authored_directive_ranges(name, &mut |_| valid = true);
        }
        element.walk_bindings(&mut |binding| {
            valid |= match binding.kind() {
                MarkupBindingKind::Bind => binding.is_static_unqualified_arg_exact("slot"),
                MarkupBindingKind::Attribute => ["slot", "slot-scope", "scope"]
                    .into_iter()
                    .any(|name| binding.is_unqualified_arg_exact(name)),
                MarkupBindingKind::On | MarkupBindingKind::Model | MarkupBindingKind::Custom => {
                    false
                }
            };
        });
        valid
    }

    fn check(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        if !element.is_unqualified_tag_exact("template") {
            return;
        }

        if !Self::has_valid_directive(element) {
            ctx.warn_at_with_help(
                ctx.t("vue/no-lone-template.message"),
                element.range(),
                ctx.t("vue/no-lone-template.help"),
            );
        }
    }
}

impl MarkupRule for NoLoneTemplate {
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

impl Rule for NoLoneTemplate {
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
    use super::NoLoneTemplate;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoLoneTemplate));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_with_v_if() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div><template v-if="show"><span>Hi</span></template></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_v_for() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div><template v-for="item in items" :key="item"><span>{{ item }}</span></template></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_slot() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent><template #header><h1>Title</h1></template></MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_legacy_slot_attributes() {
        let linter = create_linter();
        for source in [
            r#"<MyComponent><template slot="header"><h1>Title</h1></template></MyComponent>"#,
            r#"<MyComponent><template slot-scope="props">{{ props.title }}</template></MyComponent>"#,
            r#"<MyComponent><template scope="props">{{ props.title }}</template></MyComponent>"#,
            r#"<MyComponent><template v-bind:slot="name"><h1>Title</h1></template></MyComponent>"#,
            r#"<MyComponent><template :slot="name"><h1>Title</h1></template></MyComponent>"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(result.warning_count, 0, "{source}");
        }
    }

    #[test]
    fn test_invalid_dynamic_bound_slot() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent><template v-bind:[slot]="name"><h1>Title</h1></template></MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_lone_template() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div><template><span>Hi</span></template></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_lone_template_at_fragment_root() {
        let result = create_linter().lint_template(
            r#"<template><svg aria-hidden="true" /></template>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    }
}

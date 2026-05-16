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
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{ElementNode, PropNode};

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
    fn has_valid_directive(element: &ElementNode) -> bool {
        for prop in &element.props {
            if let PropNode::Directive(dir) = prop {
                let name = dir.name.as_str();
                // Template is valid if it has v-if, v-else-if, v-else, v-for, v-slot, or #slot
                if matches!(name, "if" | "else-if" | "else" | "for" | "slot") {
                    return true;
                }
            }
        }
        false
    }
}

impl Rule for NoLoneTemplate {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag.as_str() != "template" {
            return;
        }

        // Root template element is allowed
        if ctx.parent_element().is_none() {
            return;
        }

        if !Self::has_valid_directive(element) {
            ctx.warn_with_help(
                ctx.t("vue/no-lone-template.message"),
                &element.loc,
                ctx.t("vue/no-lone-template.help"),
            );
        }
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
    fn test_invalid_lone_template() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div><template><span>Hi</span></template></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }
}

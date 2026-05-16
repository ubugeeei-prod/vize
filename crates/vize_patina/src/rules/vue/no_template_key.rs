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
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_relief::ast::{ElementNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-template-key",
    description: "Disallow `key` attribute on `<template>`",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow key attribute on <template>
pub struct NoTemplateKey;

impl Rule for NoTemplateKey {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Only check <template> elements
        if element.tag.as_str() != "template" {
            return;
        }

        let has_v_for = element.props.iter().any(|prop| match prop {
            PropNode::Directive(dir) => dir.name.as_str() == "for",
            _ => false,
        });

        // Check for key attribute or :key directive
        for prop in element.props.iter() {
            match prop {
                PropNode::Attribute(attr) => {
                    if attr.name.as_str() == "key" && !has_v_for {
                        ctx.error_with_help(
                            ctx.t("vue/no-template-key.message"),
                            &attr.loc,
                            ctx.t("vue/no-template-key.help"),
                        );
                    }
                }
                PropNode::Directive(dir) => {
                    if dir.name.as_str() == "bind" {
                        if let Some(ref arg) = dir.arg {
                            if get_expression_content(arg) == "key" && !has_v_for {
                                ctx.error_with_help(
                                    ctx.t("vue/no-template-key.message"),
                                    &dir.loc,
                                    ctx.t("vue/no-template-key.help"),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Get content from ExpressionNode
fn get_expression_content(expr: &vize_relief::ast::ExpressionNode) -> String {
    match expr {
        vize_relief::ast::ExpressionNode::Simple(s) => s.content.to_compact_string(),
        vize_relief::ast::ExpressionNode::Compound(_) => String::default(),
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
}

//! vue/no-use-v-if-with-v-for
//!
//! Disallow using `v-if` on the same element that has `v-for`.
//!
//! When v-if and v-for are on the same element, v-if has higher priority.
//! This means the v-if condition won't have access to variables from v-for.
//! This is confusing and often not what developers intend.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <li v-for="item in items" v-if="item.active">{{ item }}</li>
//! ```
//!
//! ### Valid
//! ```vue
//! <!-- Use computed property to filter -->
//! <li v-for="item in activeItems" :key="item.id">{{ item }}</li>
//!
//! <!-- Or wrap with template -->
//! <template v-for="item in items" :key="item.id">
//!   <li v-if="item.active">{{ item }}</li>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::visitor::parse_v_for_variables;
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_relief::ast::{ElementNode, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-use-v-if-with-v-for",
    description: "Disallow using `v-if` on the same element as `v-for`",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow using v-if with v-for on the same element
pub struct NoUseVIfWithVFor;

impl Rule for NoUseVIfWithVFor {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        let mut v_if_info = None;
        let mut v_for_info = None;

        // Collect v-if and v-for info
        for prop in element.props.iter() {
            if let PropNode::Directive(dir) = prop {
                match dir.name.as_str() {
                    "if" | "else-if" => {
                        v_if_info = Some((dir.loc.clone(), dir.exp.as_ref()));
                    }
                    "for" => {
                        v_for_info = Some((dir.loc.clone(), dir.exp.as_ref()));
                    }
                    _ => {}
                }
            }
        }

        // Check if both exist
        if let (Some((v_if_loc, v_if_exp)), Some((v_for_loc, v_for_exp))) = (v_if_info, v_for_info)
        {
            // Extract v-for variables
            let v_for_vars: Vec<String> = v_for_exp
                .map(|exp| {
                    parse_v_for_variables(exp)
                        .into_iter()
                        .map(|s| s.to_compact_string())
                        .collect()
                })
                .unwrap_or_default();

            // Check if v-if uses any v-for variables
            let v_if_uses_v_for_var = if let Some(exp) = v_if_exp {
                let v_if_content = match exp {
                    ExpressionNode::Simple(s) => s.content.as_str(),
                    ExpressionNode::Compound(_) => "",
                };
                v_for_vars
                    .iter()
                    .any(|var| v_if_content.contains(var.as_str()))
            } else {
                false
            };

            // If v-if uses v-for variables, it's a filtering pattern which is less problematic
            // but still not recommended. We warn in both cases.
            let message = if v_if_uses_v_for_var {
                ctx.t("vue/no-use-v-if-with-v-for.message_perf")
            } else {
                ctx.t("vue/no-use-v-if-with-v-for.message_access")
            };

            let help = ctx.t("vue/no-use-v-if-with-v-for.help");

            let diagnostic = LintDiagnostic::warn(
                META.name,
                message.as_ref(),
                v_if_loc.start.offset,
                v_if_loc.end.offset,
            )
            .with_help(help.as_ref())
            .with_label(
                "v-for is here",
                v_for_loc.start.offset,
                v_for_loc.end.offset,
            );

            ctx.report(diagnostic);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoUseVIfWithVFor;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoUseVIfWithVFor));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_no_v_if_with_v_for() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item in items" :key="item.id">{{ item }}</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_v_if_on_nested_element() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<template v-for="item in items" :key="item.id"><div v-if="item.active">{{ item }}</div></template>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_v_if_with_v_for_same_element() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item in items" v-if="item.active" :key="item.id">{{ item }}</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_v_if_not_using_v_for_var() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item in items" v-if="showAll" :key="item.id">{{ item }}</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_v_else_if_with_v_for() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item in items" v-else-if="condition" :key="item.id">{{ item }}</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }
}

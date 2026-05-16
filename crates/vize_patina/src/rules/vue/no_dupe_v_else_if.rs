//! vue/no-dupe-v-else-if
//!
//! Disallow duplicate conditions in `v-if` / `v-else-if` chains.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-if="foo"></div>
//! <div v-else-if="foo"></div>
//!
//! <div v-if="a === 1"></div>
//! <div v-else-if="a === 2"></div>
//! <div v-else-if="a === 1"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-if="foo"></div>
//! <div v-else-if="bar"></div>
//! <div v-else></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_relief::ast::{
    ElementNode, ExpressionNode, PropNode, RootNode, SourceLocation, TemplateChildNode,
};

static META: RuleMeta = RuleMeta {
    name: "vue/no-dupe-v-else-if",
    description: "Disallow duplicate conditions in `v-if` / `v-else-if` chains",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow duplicate v-else-if conditions
pub struct NoDupeVElseIf;

/// Info about v-if directive on an element
struct IfDirectiveInfo {
    has_v_if: bool,
    has_v_else_if: bool,
    has_v_else: bool,
    condition: Option<String>,
    loc: Option<SourceLocation>,
}

impl Rule for NoDupeVElseIf {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        // Check root children
        check_element_children(ctx, &root.children);
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Check element children
        check_element_children(ctx, &element.children);
    }
}

/// Check children for duplicate v-else-if conditions
fn check_element_children(ctx: &mut LintContext, children: &[TemplateChildNode]) {
    let mut seen_conditions: FxHashSet<String> = FxHashSet::default();
    let mut in_if_chain = false;

    for child in children.iter() {
        if let TemplateChildNode::Element(el) = child {
            let info = get_if_directive_info(el);

            if info.has_v_if {
                // Start a new chain
                seen_conditions.clear();
                in_if_chain = true;
                if let Some(cond) = info.condition {
                    let normalized = normalize_condition(&cond);
                    seen_conditions.insert(normalized);
                }
            } else if info.has_v_else_if && in_if_chain {
                // Continue the chain
                if let Some(cond) = info.condition {
                    let normalized = normalize_condition(&cond);
                    if seen_conditions.contains(&normalized) {
                        // Report the error
                        if let Some(loc) = info.loc {
                            ctx.error_with_help(
                                ctx.t("vue/no-dupe-v-else-if.message"),
                                &loc,
                                ctx.t("vue/no-dupe-v-else-if.help"),
                            );
                        }
                    } else {
                        seen_conditions.insert(normalized);
                    }
                }
            } else if info.has_v_else && in_if_chain {
                // End the chain
                in_if_chain = false;
                seen_conditions.clear();
            } else {
                // Not part of an if chain, reset
                in_if_chain = false;
                seen_conditions.clear();
            }
        }
        // Non-element nodes (text, comment, etc.) are ignored but chain continues
    }
}

/// Get v-if/v-else-if/v-else directive info from an element
fn get_if_directive_info(el: &ElementNode) -> IfDirectiveInfo {
    let mut info = IfDirectiveInfo {
        has_v_if: false,
        has_v_else_if: false,
        has_v_else: false,
        condition: None,
        loc: None,
    };

    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop {
            match dir.name.as_str() {
                "if" => {
                    info.has_v_if = true;
                    if let Some(ref exp) = dir.exp {
                        info.condition = Some(get_expression_content(exp));
                    }
                    info.loc = Some(dir.loc.clone());
                }
                "else-if" => {
                    info.has_v_else_if = true;
                    if let Some(ref exp) = dir.exp {
                        info.condition = Some(get_expression_content(exp));
                    }
                    info.loc = Some(dir.loc.clone());
                }
                "else" => {
                    info.has_v_else = true;
                }
                _ => {}
            }
        }
    }

    info
}

/// Get content from ExpressionNode
fn get_expression_content(expr: &ExpressionNode) -> String {
    match expr {
        ExpressionNode::Simple(s) => s.content.to_compact_string(),
        ExpressionNode::Compound(_) => "<compound>".to_compact_string(),
    }
}

/// Normalize a condition for comparison (remove whitespace differences)
fn normalize_condition(condition: &str) -> String {
    condition
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .into()
}

#[cfg(test)]
mod tests {
    use super::NoDupeVElseIf;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDupeVElseIf));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_no_duplicates() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="foo"></div><div v-else-if="bar"></div><div v-else></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_duplicate_condition() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="foo"></div><div v-else-if="foo"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_duplicate_in_chain() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="a === 1"></div><div v-else-if="a === 2"></div><div v-else-if="a === 1"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }
}

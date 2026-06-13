//! vue/no-negated-v-if-condition
//!
//! Disallow a negated `v-if` condition (`v-if="!x"`) when the same conditional
//! chain has a following `v-else`.
//!
//! When a `v-if` whose condition is a logical negation is paired with a
//! `v-else`, the two branches read backwards: the "happy path" lives in the
//! `v-else`. Swapping the condition (drop the `!`) and the branch bodies makes
//! the template easier to follow.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-if="!ok">A</div>
//! <div v-else>B</div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-if="ok">A</div>
//! <div v-else>B</div>
//!
//! <div v-if="!ok">A</div>
//!
//! <div v-if="a !== b">A</div>
//! <div v-else>B</div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{
    ElementNode, ExpressionNode, PropNode, RootNode, SourceLocation, TemplateChildNode,
};

static META: RuleMeta = RuleMeta {
    name: "vue/no-negated-v-if-condition",
    description: "Disallow a negated v-if condition when the chain has a v-else",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow a negated `v-if` condition paired with a `v-else`.
pub struct NoNegatedVIfCondition;

impl Rule for NoNegatedVIfCondition {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        check_children(ctx, &root.children);
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        check_children(ctx, &element.children);
    }
}

/// What kind of conditional directive (if any) an element carries.
enum BranchKind {
    /// `v-if`, with the location of the directive and its negation status.
    If {
        negated: bool,
        loc: SourceLocation,
    },
    ElseIf,
    Else,
    None,
}

/// Walk a sibling list. A negated `v-if` is reported only when the conditional
/// chain it starts is later closed by a `v-else`.
fn check_children(ctx: &mut LintContext, children: &[TemplateChildNode]) {
    // Pending negated `v-if`: its directive location, awaiting a chain-closing
    // `v-else` somewhere after it (possibly through `v-else-if` links).
    let mut pending_negated: Option<SourceLocation> = None;
    // Whether we are currently inside a conditional chain at all (so a stray
    // `v-else` without a matching `v-if` does not affect detection).
    let mut in_chain = false;

    for child in children.iter() {
        let TemplateChildNode::Element(el) = child else {
            // Text / comment / interpolation siblings do not break a chain.
            continue;
        };

        match branch_kind(el) {
            BranchKind::If { negated, loc } => {
                // A new chain starts here; any previous pending `v-if` had no
                // `v-else` and is therefore fine.
                in_chain = true;
                pending_negated = if negated { Some(loc) } else { None };
            }
            BranchKind::ElseIf if in_chain => {
                // The chain continues; keep waiting for a possible `v-else`.
            }
            BranchKind::Else if in_chain => {
                if let Some(loc) = pending_negated.take() {
                    ctx.warn_with_help(
                        ctx.t("vue/no-negated-v-if-condition.message"),
                        &loc,
                        ctx.t("vue/no-negated-v-if-condition.help"),
                    );
                }
                in_chain = false;
            }
            _ => {
                // A non-conditional element (or a dangling else) ends the chain.
                in_chain = false;
                pending_negated = None;
            }
        }
    }
}

/// Classify an element by its conditional directive.
fn branch_kind(el: &ElementNode) -> BranchKind {
    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop {
            match dir.name.as_str() {
                "if" => {
                    let negated = dir.exp.as_ref().map(expression_is_negated).unwrap_or(false);
                    return BranchKind::If {
                        negated,
                        loc: dir.loc.clone(),
                    };
                }
                "else-if" => return BranchKind::ElseIf,
                "else" => return BranchKind::Else,
                _ => {}
            }
        }
    }
    BranchKind::None
}

/// Whether the expression text begins with a logical NOT, distinguishing a
/// leading `!` from the inequality operators `!=` and `!==`.
fn expression_is_negated(exp: &ExpressionNode) -> bool {
    let content = match exp {
        ExpressionNode::Simple(s) => s.content.as_str(),
        // Compound expressions (e.g. with interpolation) are not a simple
        // negation we can reason about; leave them alone.
        ExpressionNode::Compound(_) => return false,
    };
    let trimmed = content.trim_start();
    let rest = match trimmed.strip_prefix('!') {
        Some(rest) => rest,
        None => return false,
    };
    // `!=` / `!==` are comparisons, not a negated condition.
    !rest.starts_with('=')
}

#[cfg(test)]
mod tests {
    use super::NoNegatedVIfCondition;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoNegatedVIfCondition));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_negated_v_if_with_v_else() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div v-if="!ok">A</div><div v-else>B</div>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_negated_v_if_with_else_if_then_else() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="!ok">A</div><div v-else-if="other">B</div><div v-else>C</div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_negated_v_if_without_v_else() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-if="!ok">A</div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_negated_v_if_with_only_else_if() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="!ok">A</div><div v-else-if="other">B</div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_plain_v_if_with_v_else() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div v-if="ok">A</div><div v-else>B</div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_strict_inequality_condition() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="a !== b">A</div><div v-else>B</div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_loose_inequality_condition() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="a != b">A</div><div v-else>B</div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_negation_with_surrounding_whitespace() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="  !ok ">A</div><div v-else>B</div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn dangling_v_else_does_not_panic() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-else>B</div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }
}

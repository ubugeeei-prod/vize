use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{is_comparable_index_search, is_negative_one_literal, is_zero_literal};

/// Rule that prefers `.includes()` over index search comparisons.
#[derive(Clone, Copy, Debug, Default)]
pub struct PreferIncludesRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unexpected",
    description: "Use .includes() instead of comparing an index result.",
}];
const LISTENERS: &[&str] = &["BinaryExpression"];

impl RustLintRule for PreferIncludesRule {
    fn name(&self) -> &'static str {
        "prefer-includes"
    }

    fn docs_description(&self) -> &'static str {
        "Prefer includes over indexOf/lastIndexOf comparisons."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn requires_type_texts(&self) -> bool {
        false
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "BinaryExpression" {
            return;
        }
        let Some(left) = node.child("left") else {
            return;
        };
        let Some(right) = node.child("right") else {
            return;
        };
        if (is_comparable_index_search(left) || is_comparable_index_search(right))
            && (is_negative_one_literal(left)
                || is_negative_one_literal(right)
                || is_zero_literal(left)
                || is_zero_literal(right))
        {
            ctx.report("unexpected", node.range);
        }
    }
}

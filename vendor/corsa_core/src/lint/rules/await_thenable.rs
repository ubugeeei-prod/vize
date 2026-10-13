use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{is_obviously_promise_like, is_promise_like_node};

/// Type-aware rule that rejects `await` on values without a thenable shape.
#[derive(Clone, Copy, Debug, Default)]
pub struct AwaitThenableRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unexpected",
    description: "Unexpected await of a non-thenable value.",
}];
const LISTENERS: &[&str] = &["AwaitExpression"];

impl RustLintRule for AwaitThenableRule {
    fn name(&self) -> &'static str {
        "await-thenable"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow awaiting non-thenable values."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "AwaitExpression" {
            return;
        }
        let Some(argument) = node.child("argument") else {
            return;
        };
        if !is_promise_like_node(argument) && !is_obviously_promise_like(argument) {
            ctx.report("unexpected", node.range);
        }
    }
}

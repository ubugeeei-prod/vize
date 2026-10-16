use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{is_number_or_bigint_literal, is_unary_minus_type_safe};

/// Type-aware rule that rejects unary minus on non-number-like values.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnsafeUnaryMinusRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unaryMinus",
    description: "Argument of unary negation should be assignable to number | bigint.",
}];
const LISTENERS: &[&str] = &["UnaryExpression"];

impl RustLintRule for NoUnsafeUnaryMinusRule {
    fn name(&self) -> &'static str {
        "no-unsafe-unary-minus"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow unary negation on non-number and non-bigint values."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "UnaryExpression" || node.field_str("operator") != Some("-") {
            return;
        }
        let Some(argument) = node.child("argument") else {
            return;
        };
        if is_number_or_bigint_literal(argument) || is_unary_minus_type_safe(&argument.type_texts) {
            return;
        }
        ctx.report("unaryMinus", node.range);
    }
}

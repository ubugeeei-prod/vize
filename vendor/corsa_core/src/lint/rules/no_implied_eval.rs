use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{
    child_list, first_child_list_item, identifier_name, is_identifier_named, is_literal_string,
    member_property_name, strip_chain_expression,
};
use crate::utils::is_string_like_type_texts;

/// Type-aware rule that rejects string-based dynamic code execution APIs.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoImpliedEvalRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unexpected",
    description: "Do not pass a string to an implied eval API.",
}];
const LISTENERS: &[&str] = &["CallExpression", "NewExpression"];

impl RustLintRule for NoImpliedEvalRule {
    fn name(&self) -> &'static str {
        "no-implied-eval"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow string-based dynamic code execution APIs."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        match node.kind.as_str() {
            "CallExpression" => {
                let Some(callee) = node.child("callee") else {
                    return;
                };
                let callee = strip_chain_expression(callee);
                let callee_name = member_property_name(callee).or_else(|| identifier_name(callee));
                if !matches!(
                    callee_name.as_deref(),
                    Some("execScript" | "setInterval" | "setTimeout")
                ) {
                    return;
                }
                let Some(first_argument) = first_child_list_item(node, "arguments") else {
                    return;
                };
                if !first_argument.kind.contains("Function")
                    && (is_literal_string(first_argument)
                        || is_string_like_type_texts(&first_argument.type_texts))
                {
                    ctx.report("unexpected", node.range);
                }
            }
            "NewExpression" => {
                let Some(callee) = node.child("callee") else {
                    return;
                };
                if !is_identifier_named(callee, "Function") {
                    return;
                }
                if child_list(node, "arguments").iter().any(is_literal_string) {
                    ctx.report("unexpected", node.range);
                }
            }
            _ => {}
        }
    }
}

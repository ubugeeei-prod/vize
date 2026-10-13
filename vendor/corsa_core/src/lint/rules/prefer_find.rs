use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{
    callee_property_name, first_child_list_item, is_zero_literal, member_object,
    member_property_name,
};

/// Rule that prefers `.find()` over filtering and reading the first result.
#[derive(Clone, Copy, Debug, Default)]
pub struct PreferFindRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unexpected",
    description: "Use .find() instead of filtering and taking the first match.",
}];
const LISTENERS: &[&str] = &["CallExpression", "MemberExpression"];

impl RustLintRule for PreferFindRule {
    fn name(&self) -> &'static str {
        "prefer-find"
    }

    fn docs_description(&self) -> &'static str {
        "Prefer find over filtering and taking the first element."
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
        match node.kind.as_str() {
            "MemberExpression"
                if member_property_name(node).as_deref() == Some("0")
                    && callee_property_name(node.child("object")).as_deref() == Some("filter") =>
            {
                ctx.report("unexpected", node.range);
            }
            "CallExpression" => {
                if callee_property_name(Some(node)).as_deref() != Some("at")
                    || !first_child_list_item(node, "arguments").is_some_and(is_zero_literal)
                {
                    return;
                }
                let Some(callee) = node.child("callee") else {
                    return;
                };
                if callee_property_name(member_object(callee)).as_deref() == Some("filter") {
                    ctx.report("unexpected", node.range);
                }
            }
            _ => {}
        }
    }
}

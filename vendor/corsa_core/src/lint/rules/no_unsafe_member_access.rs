use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{
    is_any_like_node, member_object, rule_option_bool, strip_chain_expression,
};

/// Type-aware rule that rejects member access through unsafe values.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnsafeMemberAccessRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "unsafeComputedMemberAccess",
        description: "Computed name resolves to an `any` value.",
    },
    RuleMessage {
        id: "unsafeMemberExpression",
        description: "Unsafe member access on an `any` value.",
    },
    RuleMessage {
        id: "unsafeThisMemberExpression",
        description: "Unsafe member access on an `any` value. `this` is typed as `any`.",
    },
];
const LISTENERS: &[&str] = &["MemberExpression"];

impl RustLintRule for NoUnsafeMemberAccessRule {
    fn name(&self) -> &'static str {
        "no-unsafe-member-access"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow member access on values typed as any."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "MemberExpression" {
            return;
        }
        if allow_optional_chaining(node) && node.field_bool("optional") == Some(true) {
            return;
        }

        if let Some(object) = node.child("object") {
            report_unsafe_object_access(ctx, node, object);
        }
        if node.field_bool("computed").unwrap_or(false) {
            report_unsafe_computed_key(ctx, node);
        }
    }
}

fn allow_optional_chaining(node: &LintNode) -> bool {
    rule_option_bool(node, "allowOptionalChaining").unwrap_or(false)
}

fn report_unsafe_object_access(ctx: &mut RuleContext<'_>, node: &LintNode, object: &LintNode) {
    if !is_any_like_node(object) || is_reported_by_inner_member(object) {
        return;
    }
    let report_range = node
        .child("property")
        .map(|property| property.range)
        .unwrap_or(node.range);
    ctx.report("unsafeMemberExpression", report_range);
}

fn report_unsafe_computed_key(ctx: &mut RuleContext<'_>, node: &LintNode) {
    let Some(property) = node.child("property") else {
        return;
    };
    if is_safe_computed_key(property) || !is_any_like_node(property) {
        return;
    }
    ctx.report("unsafeComputedMemberAccess", property.range);
}

fn is_reported_by_inner_member(object: &LintNode) -> bool {
    let current = strip_chain_expression(object);
    current.kind == "MemberExpression" && member_object(current).is_some_and(is_any_like_node)
}

fn is_safe_computed_key(property: &LintNode) -> bool {
    let current = strip_chain_expression(property);
    current.kind == "Literal"
        || (current.kind == "UpdateExpression"
            && matches!(current.field_str("operator"), Some("++" | "--")))
}

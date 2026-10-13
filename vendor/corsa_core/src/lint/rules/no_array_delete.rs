use super::super::{
    LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule, TextRange,
};
use crate::utils::is_array_like_type_texts;

/// Type-aware rule that rejects `delete` against array-like elements.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoArrayDeleteRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "unexpected",
        description: "Do not delete elements from an array-like value.",
    },
    RuleMessage {
        id: "useSplice",
        description: "Use array.splice(index, 1) instead.",
    },
];
const LISTENERS: &[&str] = &["UnaryExpression"];

impl RustLintRule for NoArrayDeleteRule {
    fn name(&self) -> &'static str {
        "no-array-delete"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow deleting elements from array-like values."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn has_suggestions(&self) -> bool {
        true
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "UnaryExpression" || node.field_str("operator") != Some("delete") {
            return;
        }
        let Some(argument) = node.child("argument") else {
            return;
        };
        if argument.kind != "MemberExpression" || argument.field_bool("computed") != Some(true) {
            return;
        }
        let Some(object) = argument.child("object") else {
            return;
        };
        if object.kind != "ArrayExpression" && !is_array_like_type_texts(&object.type_texts) {
            return;
        }
        let suggestions = splice_suggestion(node, argument, object)
            .into_iter()
            .collect();
        ctx.report_with_suggestions("unexpected", node.range, suggestions);
    }
}

fn splice_suggestion(
    node: &LintNode,
    argument: &LintNode,
    object: &LintNode,
) -> Option<LintSuggestion> {
    let property = argument.child("property")?;
    let delete_range = TextRange::new(node.range.start, object.range.start);
    let left_bracket_range = TextRange::new(object.range.end, property.range.start);
    let right_bracket_range = TextRange::new(property.range.end, argument.range.end);
    if !delete_range.is_valid() || !left_bracket_range.is_valid() || !right_bracket_range.is_valid()
    {
        return None;
    }
    Some(LintSuggestion {
        message_id: "useSplice".to_owned(),
        message: "Use array.splice(index, 1) instead.".to_owned(),
        fixes: vec![
            LintFix::remove_range(delete_range),
            LintFix::replace_range(left_bracket_range, ".splice("),
            LintFix::replace_range(right_bracket_range, ", 1)"),
        ],
    })
}

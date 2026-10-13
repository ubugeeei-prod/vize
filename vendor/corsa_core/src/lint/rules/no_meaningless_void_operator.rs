use super::super::{
    LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule, TextRange,
};
use crate::{lint::helpers::rule_option_bool, utils::split_type_text};

/// Type-aware rule that rejects `void` when the expression is already void-like.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoMeaninglessVoidOperatorRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "meaninglessVoidOperator",
        description: "void operator should only be used to ignore a non-void return value.",
    },
    RuleMessage {
        id: "removeVoid",
        description: "Remove 'void'.",
    },
];
const LISTENERS: &[&str] = &["UnaryExpression"];

impl RustLintRule for NoMeaninglessVoidOperatorRule {
    fn name(&self) -> &'static str {
        "no-meaningless-void-operator"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow void operators on expressions that are already void-like."
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
        if node.kind != "UnaryExpression" || node.field_str("operator") != Some("void") {
            return;
        }
        let Some(argument) = node.child("argument") else {
            return;
        };
        if is_always_void_like(&argument.type_texts)
            || (check_never(node) && is_always_void_like_or_never(&argument.type_texts))
        {
            ctx.report_with_suggestions(
                "meaninglessVoidOperator",
                node.range,
                remove_void_suggestion(node, argument).into_iter().collect(),
            );
        }
    }
}

fn check_never(node: &LintNode) -> bool {
    rule_option_bool(node, "checkNever").unwrap_or(false)
}

fn is_always_void_like(type_texts: &[String]) -> bool {
    !type_texts.is_empty()
        && type_texts
            .iter()
            .flat_map(|text| split_type_text(text))
            .all(|text| matches!(text.trim(), "void" | "undefined"))
}

fn is_always_void_like_or_never(type_texts: &[String]) -> bool {
    !type_texts.is_empty()
        && type_texts
            .iter()
            .flat_map(|text| split_type_text(text))
            .all(|text| matches!(text.trim(), "void" | "undefined" | "never"))
}

fn remove_void_suggestion(node: &LintNode, argument: &LintNode) -> Option<LintSuggestion> {
    let remove_range = TextRange::new(node.range.start, argument.range.start);
    if !remove_range.is_valid() {
        return None;
    }
    Some(LintSuggestion {
        message_id: "removeVoid".to_owned(),
        message: "Remove 'void'.".to_owned(),
        fixes: vec![LintFix::remove_range(remove_range)],
    })
}

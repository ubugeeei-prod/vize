use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{callee_property_name, child_list, member_object, rule_option_bool};
use crate::utils::{is_array_like_type_texts, is_string_array_like_type_texts};

/// Type-aware rule that requires compare callbacks for array sorting calls.
#[derive(Clone, Copy, Debug, Default)]
pub struct RequireArraySortCompareRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "requireCompare",
    description: "Require a compare argument for array sorting.",
}];
const LISTENERS: &[&str] = &["CallExpression"];

impl RustLintRule for RequireArraySortCompareRule {
    fn name(&self) -> &'static str {
        "require-array-sort-compare"
    }

    fn docs_description(&self) -> &'static str {
        "Require compare callbacks for array sorting calls."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "CallExpression" || !child_list(node, "arguments").is_empty() {
            return;
        }
        if !matches!(
            callee_property_name(Some(node)).as_deref(),
            Some("sort" | "toSorted")
        ) {
            return;
        }
        let Some(callee) = node.child("callee") else {
            return;
        };
        let Some(object) = member_object(callee) else {
            return;
        };
        if object.kind != "ArrayExpression" && !is_array_like_type_texts(&object.type_texts) {
            return;
        }
        if ignore_string_arrays(node) && is_string_array_like_type_texts(&object.type_texts) {
            return;
        }
        ctx.report("requireCompare", node.range);
    }
}

fn ignore_string_arrays(node: &LintNode) -> bool {
    rule_option_bool(node, "ignoreStringArrays").unwrap_or(true)
}

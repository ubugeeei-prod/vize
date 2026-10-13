use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{callee_property_name, first_child_list_item, regex_flags};

/// Rule that prefers `RegExp#exec` for non-global regular-expression matches.
#[derive(Clone, Copy, Debug, Default)]
pub struct PreferRegexpExecRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unexpected",
    description: "Use a RegExp exec() call instead of String match().",
}];
const LISTENERS: &[&str] = &["CallExpression"];

impl RustLintRule for PreferRegexpExecRule {
    fn name(&self) -> &'static str {
        "prefer-regexp-exec"
    }

    fn docs_description(&self) -> &'static str {
        "Prefer RegExp#exec over String#match for single matches."
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
        if node.kind != "CallExpression"
            || callee_property_name(Some(node)).as_deref() != Some("match")
        {
            return;
        }
        let Some(first_argument) = first_child_list_item(node, "arguments") else {
            return;
        };
        let Some(flags) = regex_flags(first_argument) else {
            return;
        };
        if !flags.contains('g') {
            ctx.report("unexpected", node.range);
        }
    }
}

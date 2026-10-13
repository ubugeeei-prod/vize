use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{callee_property_name, child_list, has_unknown_type_annotation};

/// Rule that requires Promise catch callback variables to be typed `unknown`.
#[derive(Clone, Copy, Debug, Default)]
pub struct UseUnknownInCatchCallbackVariableRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unexpected",
    description: "Catch callback variables should be explicitly typed as unknown.",
}];
const LISTENERS: &[&str] = &["CallExpression"];

impl RustLintRule for UseUnknownInCatchCallbackVariableRule {
    fn name(&self) -> &'static str {
        "use-unknown-in-catch-callback-variable"
    }

    fn docs_description(&self) -> &'static str {
        "Require Promise catch callback variables to use an explicit unknown annotation."
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
        if node.kind != "CallExpression" {
            return;
        }
        let property_name = callee_property_name(Some(node));
        let callback = match property_name.as_deref() {
            Some("catch") => child_list(node, "arguments").first(),
            Some("then") => child_list(node, "arguments").get(1),
            _ => None,
        };
        let Some(callback) = callback else {
            return;
        };
        if !callback.kind.contains("Function") {
            return;
        }
        let Some(parameter) = callback
            .child_list("params")
            .and_then(|params| params.first())
        else {
            return;
        };
        if !has_unknown_type_annotation(parameter) {
            ctx.report("unexpected", parameter.range);
        }
    }
}

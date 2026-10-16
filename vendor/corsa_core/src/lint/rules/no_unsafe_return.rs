use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::utils::is_unsafe_return;

/// Type-aware rule that rejects returning any-typed values from functions.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnsafeReturnRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unsafe",
    description: "Unsafe return of an any-typed value.",
}];
const LISTENERS: &[&str] = &["ArrowFunctionExpression", "ReturnStatement"];

impl RustLintRule for NoUnsafeReturnRule {
    fn name(&self) -> &'static str {
        "no-unsafe-return"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow returning any-typed values from functions."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        match node.kind.as_str() {
            "ArrowFunctionExpression" => {
                let Some(body) = node.child("body") else {
                    return;
                };
                if body.kind == "BlockStatement" {
                    return;
                }
                report_if_unsafe(ctx, node, body);
            }
            "ReturnStatement" => {
                let Some(argument) = node.child("argument") else {
                    return;
                };
                report_if_unsafe(ctx, node, argument);
            }
            _ => {}
        }
    }
}

fn report_if_unsafe(ctx: &mut RuleContext<'_>, owner: &LintNode, source: &LintNode) {
    let target_type_texts = return_type_texts(owner);
    if is_unsafe_return(&source.type_texts, &target_type_texts) {
        ctx.report("unsafe", source.range);
    }
}

fn return_type_texts(node: &LintNode) -> Vec<String> {
    node.fields
        .get("__returnTypeTexts")
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

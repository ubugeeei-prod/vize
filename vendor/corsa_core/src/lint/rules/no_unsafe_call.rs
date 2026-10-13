use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{is_any_like_node, strip_chain_expression};

/// Type-aware rule that rejects calling values with unsafe callable types.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnsafeCallRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "unsafeCall",
        description: "Unsafe call of a(n) `any` typed value.",
    },
    RuleMessage {
        id: "unsafeCallThis",
        description: "Unsafe call of a(n) `any` typed value. `this` is typed as `any`.",
    },
    RuleMessage {
        id: "unsafeNew",
        description: "Unsafe construction of a(n) `any` typed value.",
    },
    RuleMessage {
        id: "unsafeTemplateTag",
        description: "Unsafe use of a(n) `any` typed template tag.",
    },
];
const LISTENERS: &[&str] = &[
    "CallExpression",
    "NewExpression",
    "TaggedTemplateExpression",
];

impl RustLintRule for NoUnsafeCallRule {
    fn name(&self) -> &'static str {
        "no-unsafe-call"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow calling values typed as any."
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
                if is_import_expression(callee) {
                    return;
                }
                report_if_unsafe(ctx, callee, "unsafeCall", callee);
            }
            "NewExpression" => {
                let Some(callee) = node.child("callee") else {
                    return;
                };
                report_if_unsafe(ctx, callee, "unsafeNew", node);
            }
            "TaggedTemplateExpression" => {
                let Some(tag) = node.child("tag") else {
                    return;
                };
                report_if_unsafe(ctx, tag, "unsafeTemplateTag", tag);
            }
            _ => {}
        }
    }
}

fn report_if_unsafe(
    ctx: &mut RuleContext<'_>,
    callee: &LintNode,
    message_id: &'static str,
    report_node: &LintNode,
) {
    if is_any_like_node(callee) || is_function_type(callee) {
        ctx.report(message_id, report_node.range);
    }
}

fn is_import_expression(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    matches!(
        current.kind.as_str(),
        "Import" | "ImportExpression" | "ImportKeyword"
    )
}

fn is_function_type(node: &LintNode) -> bool {
    node.type_texts.iter().any(|text| {
        let text = text.trim();
        text == "Function" || text.ends_with(" & Function") || text.ends_with(" extends Function")
    })
}

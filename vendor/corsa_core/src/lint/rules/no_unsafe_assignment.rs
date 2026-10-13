use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::utils::is_unsafe_assignment;

/// Type-aware rule that rejects assigning any-typed values to specific targets.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnsafeAssignmentRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unsafe",
    description: "Unsafe assignment of an any-typed value.",
}];
const LISTENERS: &[&str] = &[
    "AssignmentExpression",
    "PropertyDefinition",
    "VariableDeclarator",
];

impl RustLintRule for NoUnsafeAssignmentRule {
    fn name(&self) -> &'static str {
        "no-unsafe-assignment"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow assigning any-typed values to more specific targets."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        match node.kind.as_str() {
            "AssignmentExpression" if node.field_str("operator") == Some("=") => {
                let Some(source) = node.child("right") else {
                    return;
                };
                let target_type_texts = node
                    .child("left")
                    .map(|target| target.type_texts.as_slice())
                    .unwrap_or(&[]);
                report_if_unsafe(ctx, source, target_type_texts, node);
            }
            "PropertyDefinition" => {
                let Some(source) = node.child("value") else {
                    return;
                };
                let target_type_texts = annotated_target_type_texts(node);
                report_if_unsafe(ctx, source, &target_type_texts, node);
            }
            "VariableDeclarator" => {
                let Some(source) = node.child("init") else {
                    return;
                };
                let target_type_texts = node
                    .child("id")
                    .map(annotated_target_type_texts)
                    .unwrap_or_default();
                report_if_unsafe(ctx, source, &target_type_texts, node);
            }
            _ => {}
        }
    }
}

fn report_if_unsafe(
    ctx: &mut RuleContext<'_>,
    source: &LintNode,
    target_type_texts: &[String],
    report_node: &LintNode,
) {
    if is_unsafe_assignment(&source.type_texts, target_type_texts) {
        ctx.report("unsafe", report_node.range);
    }
}

fn annotated_target_type_texts(node: &LintNode) -> Vec<String> {
    if !has_type_annotation(node) {
        return Vec::new();
    }
    if let Some(text) = node.field_str("__typeAnnotationText") {
        if !text.is_empty() {
            return vec![text.to_owned()];
        }
    }
    node.type_texts.clone()
}

fn has_type_annotation(node: &LintNode) -> bool {
    node.child("typeAnnotation").is_some()
}

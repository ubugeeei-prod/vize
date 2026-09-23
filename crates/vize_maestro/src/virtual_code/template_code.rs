//! Template expression extraction for script-export decisions.
//!
//! The editor template document is the checker virtual TypeScript. This module
//! only lists the expressions a `<script setup>` export set needs.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use vize_armature::RootNode;
use vize_relief::{DirectiveNode, ExpressionNode, PropNode, SourceLocation, TemplateChildNode};

use super::SourceRange;

/// Extract expressions from a template for quick analysis.
#[derive(Debug, Clone)]
pub struct TemplateExpression {
    /// Expression text
    pub text: String,
    /// Source range in template
    pub range: SourceRange,
    /// Expression kind
    pub kind: ExpressionKind,
}

/// Kind of template expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionKind {
    /// Interpolation {{ expr }}
    Interpolation,
    /// v-bind expression
    VBind,
    /// v-on expression
    VOn,
    /// v-if/v-else-if condition
    VIf,
    /// v-for source
    VFor,
    /// v-model value
    VModel,
    /// v-show condition
    VShow,
    /// v-slot binding
    VSlot,
    /// Other directive expression
    Other,
}

/// Quick extraction of expressions from template.
pub fn extract_expressions<'a>(ast: &'a RootNode<'a>) -> Vec<TemplateExpression> {
    let mut expressions = Vec::new();
    extract_from_children(&ast.children, &mut expressions);
    expressions
}

fn extract_from_children<'a>(
    children: &'a [TemplateChildNode<'a>],
    expressions: &mut Vec<TemplateExpression>,
) {
    for child in children {
        extract_from_child(child, expressions);
    }
}

fn extract_from_child<'a>(
    node: &'a TemplateChildNode<'a>,
    expressions: &mut Vec<TemplateExpression>,
) {
    match node {
        TemplateChildNode::Element(el) => {
            for prop in &el.props {
                if let PropNode::Directive(dir) = prop {
                    extract_from_directive(dir, expressions);
                }
            }
            extract_from_children(&el.children, expressions);
        }
        TemplateChildNode::Interpolation(interp) => {
            if let Some((text, loc)) = get_expression_content(&interp.content) {
                expressions.push(TemplateExpression {
                    text,
                    range: SourceRange::from(loc),
                    kind: ExpressionKind::Interpolation,
                });
            }
        }
        TemplateChildNode::If(if_node) => {
            for branch in &if_node.branches {
                if let Some(ref cond) = branch.condition
                    && let Some((text, loc)) = get_expression_content(cond)
                {
                    expressions.push(TemplateExpression {
                        text,
                        range: SourceRange::from(loc),
                        kind: ExpressionKind::VIf,
                    });
                }
                extract_from_children(&branch.children, expressions);
            }
        }
        TemplateChildNode::For(for_node) => {
            if let Some((text, loc)) = get_expression_content(&for_node.source) {
                expressions.push(TemplateExpression {
                    text,
                    range: SourceRange::from(loc),
                    kind: ExpressionKind::VFor,
                });
            }
            extract_from_children(&for_node.children, expressions);
        }
        TemplateChildNode::IfBranch(branch) => {
            if let Some(ref cond) = branch.condition
                && let Some((text, loc)) = get_expression_content(cond)
            {
                expressions.push(TemplateExpression {
                    text,
                    range: SourceRange::from(loc),
                    kind: ExpressionKind::VIf,
                });
            }
            extract_from_children(&branch.children, expressions);
        }
        _ => {}
    }
}

fn extract_from_directive<'a>(
    dir: &'a DirectiveNode<'a>,
    expressions: &mut Vec<TemplateExpression>,
) {
    if let Some(ref exp) = dir.exp {
        let kind = match dir.name {
            "bind" => ExpressionKind::VBind,
            "on" => ExpressionKind::VOn,
            "if" | "else-if" => ExpressionKind::VIf,
            "for" => ExpressionKind::VFor,
            "model" => ExpressionKind::VModel,
            "show" => ExpressionKind::VShow,
            "slot" => ExpressionKind::VSlot,
            _ => ExpressionKind::Other,
        };
        if let Some((text, loc)) = get_expression_content(exp) {
            expressions.push(TemplateExpression {
                text,
                range: SourceRange::from(loc),
                kind,
            });
        }
    }
}

/// Helper to get content and location from an ExpressionNode.
fn get_expression_content<'a>(
    expr: &'a ExpressionNode<'a>,
) -> Option<(String, &'a SourceLocation)> {
    match expr {
        ExpressionNode::Simple(simple) => {
            if simple.content.is_empty() {
                None
            } else {
                Some((simple.content.to_string(), &simple.loc))
            }
        }
        ExpressionNode::Compound(compound) => {
            // For compound expressions, we return the full location
            // but can't easily get a single content string
            Some(("<compound>".to_string(), &compound.loc))
        }
    }
}

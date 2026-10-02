//! Conditions use their own template AST spans and the checker projection.
use super::queries::{Collector, Condition};
use oxc_allocator::Allocator;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_relief::{ExpressionNode, PropNode, RootNode, TemplateChildNode};

pub(super) fn collect(root: &RootNode<'_>, offset: u32, out: &mut Vec<Condition>) {
    children(&root.children, root.source, offset, out);
}

fn expression(
    expression: &ExpressionNode<'_>,
    source: &str,
    offset: u32,
    out: &mut Vec<Condition>,
) {
    let span = expression.loc().span;
    let Some(text) = source.get(span.start as usize..span.end as usize) else {
        return;
    };
    let allocator = Allocator::default();
    let Ok(parsed) = Parser::new(&allocator, text, SourceType::ts()).parse_expression() else {
        return;
    };
    let mut collector = Collector {
        conditions: Vec::new(),
    };
    collector.condition(&parsed);
    collector.visit_expression(&parsed);
    out.extend(collector.conditions.into_iter().map(|condition| Condition {
        start: offset + span.start + condition.start,
        end: offset + span.start + condition.end,
    }));
}

fn children(nodes: &[TemplateChildNode<'_>], source: &str, offset: u32, out: &mut Vec<Condition>) {
    for node in nodes {
        match node {
            TemplateChildNode::Element(element) => {
                for prop in &element.props {
                    if let PropNode::Directive(directive) = prop
                        && matches!(directive.name, "if" | "else-if" | "show")
                        && let Some(expr) = &directive.exp
                    {
                        expression(expr, source, offset, out);
                    }
                }
                children(&element.children, source, offset, out);
            }
            TemplateChildNode::If(node) => {
                for branch in &node.branches {
                    if let Some(expr) = &branch.condition {
                        expression(expr, source, offset, out);
                    }
                    children(&branch.children, source, offset, out);
                }
            }
            TemplateChildNode::IfBranch(branch) => {
                if let Some(expr) = &branch.condition {
                    expression(expr, source, offset, out);
                }
                children(&branch.children, source, offset, out);
            }
            TemplateChildNode::For(node) => children(&node.children, source, offset, out),
            _ => {}
        }
    }
}

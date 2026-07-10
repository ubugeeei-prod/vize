mod control_flow;
mod expressions;
mod syntax;

use vize_carton::{Box, Bump};

use crate::{
    ExpressionNode, InterpolationNode, Position, SimpleExpressionNode, SourceLocation, TextNode,
};

fn location(start: u32, end: u32, source: &str) -> SourceLocation {
    SourceLocation::new(
        Position::new(start, 1, start + 1),
        Position::new(end, 1, end + 1),
        source,
    )
}

fn expression<'a>(allocator: &'a Bump, content: &str, start: u32, end: u32) -> ExpressionNode<'a> {
    ExpressionNode::Simple(Box::new_in(
        SimpleExpressionNode::new(content, false, location(start, end, content)),
        allocator,
    ))
}

fn interpolation<'a>(
    allocator: &'a Bump,
    content: &str,
    start: u32,
    end: u32,
) -> InterpolationNode<'a> {
    InterpolationNode {
        content: expression(allocator, content, start + 2, end - 2),
        loc: location(start, end, content),
        #[cfg(feature = "_legacy")]
        raw: false,
    }
}

fn text(content: &str, start: u32, end: u32) -> TextNode {
    TextNode::new(content, location(start, end, content))
}

fn assert_owned_product<T: Send + Sync + 'static>(_: &T) {}

use vize_s0::{Allocator, Box, String, Vec};

use crate::{DirectiveNode, ExpressionNode, PropNode, SimpleExpressionNode, SourceLocation};

pub(super) fn rewrite_case_directive<'a>(
    allocator: &'a Allocator,
    dir: &mut DirectiveNode<'a>,
    name: &'static str,
    raw_name: &'static str,
    condition: Option<String>,
) {
    let loc = dir.loc.clone();
    dir.name = name;
    dir.raw_name = Some(raw_name);
    dir.exp = condition.map(|condition| {
        ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new(allocator.alloc_str(&condition), false, loc),
            &allocator,
        ))
    });
    dir.arg = None;
    dir.modifiers = Vec::new_in(&allocator);
    dir.for_parse_result = None;
    dir.shorthand = false;
}

pub(super) fn create_directive<'a>(
    allocator: &'a Allocator,
    name: &'static str,
    raw_name: &'static str,
    exp: Option<String>,
    loc: SourceLocation,
) -> PropNode<'a> {
    let mut dir = DirectiveNode::new(allocator, name, loc.clone());
    dir.raw_name = Some(raw_name);
    dir.exp = exp.map(|exp| {
        ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new(allocator.alloc_str(&exp), false, loc),
            &allocator,
        ))
    });
    PropNode::Directive(Box::new_in(dir, &allocator))
}

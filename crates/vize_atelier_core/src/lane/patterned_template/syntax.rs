use vize_s0::String;

use crate::ExpressionNode;

pub(super) fn expression_source(exp: &ExpressionNode<'_>, source: &str) -> String {
    match exp {
        ExpressionNode::Simple(simple) => simple.content.into(),
        ExpressionNode::Compound(compound) => String::new(compound.loc.span.slice(source)),
    }
}

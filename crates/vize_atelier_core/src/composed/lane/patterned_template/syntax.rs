// DO NOT EDIT. Generated from crates/vize_atelier_core/src/lane/patterned_template/syntax.rs by `node tools/commands/generate/composed-copies.ts`.
// Edit the source and run the command again.
use vize_l0::String;

use crate::composed::ExpressionNode;

pub(super) fn expression_source(exp: &ExpressionNode<'_>, source: &str) -> String {
    match exp {
        ExpressionNode::Simple(simple) => simple.content.into(),
        ExpressionNode::Compound(compound) => String::new(compound.loc.span.slice(source)),
    }
}

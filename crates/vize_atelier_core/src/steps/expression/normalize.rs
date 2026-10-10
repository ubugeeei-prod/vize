//! Byte-identical normalization of template expression nodes.

use vize_l0::{Allocator, Box};

use crate::{ConstantType, ExpressionNode, SimpleExpressionNode};

pub(crate) fn normalize_expression<'a>(
    exp: &ExpressionNode<'a>,
    allocator: &'a Allocator,
    source: &'a str,
) -> Box<'a, SimpleExpressionNode<'a>> {
    match exp {
        ExpressionNode::Simple(simple) => Box::new_in(
            SimpleExpressionNode {
                content: simple.content,
                is_static: simple.is_static,
                const_type: simple.const_type,
                loc: simple.loc.clone(),
                // Byte-identical clone: the retained parse still applies (P1-7).
                js_ast: simple.js_ast,
                hoisted: None,
                identifiers: None,
                is_handler_key: simple.is_handler_key,
                is_ref_transformed: simple.is_ref_transformed,
            },
            &allocator,
        ),
        ExpressionNode::Compound(compound) => Box::new_in(
            SimpleExpressionNode {
                content: compound.loc.span.slice(source),
                is_static: false,
                const_type: ConstantType::NotConstant,
                loc: compound.loc.clone(),
                js_ast: None,
                hoisted: None,
                identifiers: None,
                is_handler_key: compound.is_handler_key,
                is_ref_transformed: false,
            },
            &allocator,
        ),
    }
}

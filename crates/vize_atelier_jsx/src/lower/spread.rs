//! `{...items}` as a JSX child (#6888).
//!
//! `@vue/babel-plugin-jsx` spreads the value into the children array. The
//! optimized VDOM output tracks a parent's dynamic children, so a bare spread
//! of raw values would neither mount primitives nor patch length changes.
//! The spread therefore becomes its own Fragment block with the `BAIL` patch
//! flag: the block registers with its parent, and `BAIL` makes Vue normalize
//! and fully diff the raw children, exactly as for Babel's unoptimized vnodes.

use oxc_ast::ast::JSXSpreadChild;
use oxc_span::GetSpan;
use vize_relief::{CompoundExpressionChild, CompoundExpressionNode, TemplateChildNode};

use super::Lowerer;
use crate::spread_children::{SPREAD_BLOCK_CLOSE, SPREAD_BLOCK_OPEN};

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    /// `(_openBlock(), _createBlock(_Fragment, null, [...items], -2 /* BAIL */))`.
    pub(super) fn spread_block(&self, spread: &JSXSpreadChild<'_>) -> TemplateChildNode<'a> {
        let argument = self.dyn_simple_expr(spread.expression.span());
        let mut compound =
            CompoundExpressionNode::new(self.bump(), self.mapper().location(spread.span));
        compound
            .children
            .push(CompoundExpressionChild::String(SPREAD_BLOCK_OPEN));
        compound
            .children
            .push(CompoundExpressionChild::Simple(argument));
        compound
            .children
            .push(CompoundExpressionChild::String(SPREAD_BLOCK_CLOSE));
        TemplateChildNode::CompoundExpression(self.boxed(compound))
    }
}

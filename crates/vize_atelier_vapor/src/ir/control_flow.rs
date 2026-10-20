//! Control-flow operations: `v-if`, `v-for` and patterned-template scopes.

use vize_atelier_core::SimpleExpressionNode;
use vize_carton::Box;

use super::{BlockIRNode, InsertionAnchor};

/// If operation
#[derive(Debug)]
pub struct IfIRNode<'a> {
    pub id: usize,
    pub condition: Box<'a, SimpleExpressionNode<'a>>,
    pub positive: BlockIRNode<'a>,
    pub negative: Option<NegativeBranch<'a>>,
    pub once: bool,
    pub parent: Option<usize>,
    pub anchor: Option<InsertionAnchor>,
}

/// Negative branch of if
#[derive(Debug)]
pub enum NegativeBranch<'a> {
    Block(BlockIRNode<'a>),
    If(Box<'a, IfIRNode<'a>>),
}

/// For operation
#[derive(Debug)]
pub struct ForIRNode<'a> {
    pub id: usize,
    pub source: Box<'a, SimpleExpressionNode<'a>>,
    pub value: Option<Box<'a, SimpleExpressionNode<'a>>>,
    pub key: Option<Box<'a, SimpleExpressionNode<'a>>>,
    pub index: Option<Box<'a, SimpleExpressionNode<'a>>>,
    pub key_prop: Option<Box<'a, SimpleExpressionNode<'a>>>,
    pub render: BlockIRNode<'a>,
    pub once: bool,
    pub component: bool,
    pub only_child: bool,
    /// A `<template v-for>` row that is not one plain element: the server
    /// wraps each row in a fragment, which hydration must claim.
    pub wrapped_rows: bool,
    pub parent: Option<usize>,
    pub anchor: Option<InsertionAnchor>,
    /// The lexical scope a patterned-template `v-match` lowers to (RFC 823):
    /// the block runs once against a computed of the source instead of being
    /// rendered per list item, so it introduces no list fragment.
    pub match_scope: bool,
}

/// An authored reactive key owns a fresh block scope for each key value.
#[derive(Debug)]
pub struct KeyIRNode<'a> {
    pub id: usize,
    pub value: Box<'a, SimpleExpressionNode<'a>>,
    pub render: BlockIRNode<'a>,
    pub parent: Option<usize>,
    pub anchor: Option<InsertionAnchor>,
}

/// A lowered `v-for` alias without its TypeScript annotation.
pub(crate) fn untype_alias<'a>(
    mut alias: Box<'a, SimpleExpressionNode<'a>>,
) -> Box<'a, SimpleExpressionNode<'a>> {
    let untyped = vize_atelier_core::steps::untyped_for_alias(alias.content);
    if untyped.len() != alias.content.len() {
        alias.content = untyped;
        alias.js_ast = None;
    }
    alias
}

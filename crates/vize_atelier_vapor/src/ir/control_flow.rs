//! Control-flow operations: `v-if`, `v-for` and patterned-template scopes.

use vize_atelier_core::SimpleExpressionNode;
use vize_carton::Box;

use super::BlockIRNode;

/// If operation
#[derive(Debug)]
pub struct IfIRNode<'a> {
    pub id: usize,
    pub condition: Box<'a, SimpleExpressionNode<'a>>,
    pub positive: BlockIRNode<'a>,
    pub negative: Option<NegativeBranch<'a>>,
    pub once: bool,
    pub parent: Option<usize>,
    pub anchor: Option<usize>,
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
    pub parent: Option<usize>,
    pub anchor: Option<usize>,
    /// The lexical scope a patterned-template `v-match` lowers to (RFC 823):
    /// the block runs once against a computed of the source instead of being
    /// rendered per list item, so it introduces no list fragment.
    pub match_scope: bool,
}

//! SSR codegen statement nodes.
//!
//! Block/if statements, template literals, and assignment/sequence/return
//! statements used by the SSR codegen path. Split out of `render_ir` to keep
//! that file focused on the VNode and JS-expression IR.

use vize_carton::{Box, Bump, String, Vec};

use vize_relief::{
    ExpressionNode, NodeType, SimpleExpressionNode, SourceLocation, TemplateChildNode,
};

use crate::render_ir::JsChildNode;

/// Block statement
#[derive(Debug)]
pub struct BlockStatement<'a> {
    pub body: Vec<'a, BlockStatementBody<'a>>,
    pub loc: SourceLocation,
}

impl<'a> BlockStatement<'a> {
    pub fn new(allocator: &'a Bump, loc: SourceLocation) -> Self {
        Self {
            body: Vec::new_in(allocator),
            loc,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::JsBlockStatement
    }
}

/// Block statement body item
#[derive(Debug)]
pub enum BlockStatementBody<'a> {
    JsChild(JsChildNode<'a>),
    If(Box<'a, IfStatement<'a>>),
}

/// Template literal
#[derive(Debug)]
pub struct TemplateLiteral<'a> {
    pub elements: Vec<'a, TemplateLiteralElement<'a>>,
    pub loc: SourceLocation,
}

impl<'a> TemplateLiteral<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::JsTemplateLiteral
    }
}

/// Template literal element
#[derive(Debug)]
pub enum TemplateLiteralElement<'a> {
    String(String),
    JsChild(JsChildNode<'a>),
}

/// If statement (SSR)
#[derive(Debug)]
pub struct IfStatement<'a> {
    pub test: ExpressionNode<'a>,
    pub consequent: Box<'a, BlockStatement<'a>>,
    pub alternate: Option<IfStatementAlternate<'a>>,
    pub loc: SourceLocation,
}

impl<'a> IfStatement<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::JsIfStatement
    }
}

/// If statement alternate
#[derive(Debug)]
pub enum IfStatementAlternate<'a> {
    If(Box<'a, IfStatement<'a>>),
    Block(Box<'a, BlockStatement<'a>>),
    Return(Box<'a, ReturnStatement<'a>>),
}

/// Assignment expression
#[derive(Debug)]
pub struct AssignmentExpression<'a> {
    pub left: Box<'a, SimpleExpressionNode<'a>>,
    pub right: JsChildNode<'a>,
    pub loc: SourceLocation,
}

impl<'a> AssignmentExpression<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::JsAssignmentExpression
    }
}

/// Sequence expression
#[derive(Debug)]
pub struct SequenceExpression<'a> {
    pub expressions: Vec<'a, JsChildNode<'a>>,
    pub loc: SourceLocation,
}

impl<'a> SequenceExpression<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::JsSequenceExpression
    }
}

/// Return statement
#[derive(Debug)]
pub struct ReturnStatement<'a> {
    pub returns: ReturnValue<'a>,
    pub loc: SourceLocation,
}

impl<'a> ReturnStatement<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::JsReturnStatement
    }
}

/// Return value type
#[derive(Debug)]
pub enum ReturnValue<'a> {
    Single(TemplateChildNode<'a>),
    Multiple(Vec<'a, TemplateChildNode<'a>>),
    JsChild(JsChildNode<'a>),
}

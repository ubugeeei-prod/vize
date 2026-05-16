//! Expression AST node types.
//!
//! Contains simple and compound expression nodes used in
//! template bindings, directives, and interpolations.

use vize_carton::{Box, Bump, String, Vec};

use super::{
    codegen::JsChildNode,
    core::{ConstantType, NodeType, SourceLocation},
    elements::{InterpolationNode, TextNode},
    RuntimeHelper,
};

/// Expression node types
#[derive(Debug)]
pub enum ExpressionNode<'a> {
    Simple(Box<'a, SimpleExpressionNode<'a>>),
    Compound(Box<'a, CompoundExpressionNode<'a>>),
}

impl<'a> ExpressionNode<'a> {
    pub fn loc(&self) -> &SourceLocation {
        match self {
            Self::Simple(n) => &n.loc,
            Self::Compound(n) => &n.loc,
        }
    }
}

/// Simple expression node
#[derive(Debug)]
pub struct SimpleExpressionNode<'a> {
    pub content: String,
    pub is_static: bool,
    pub const_type: ConstantType,
    pub loc: SourceLocation,
    /// Parsed JavaScript AST (None = simple identifier, Some = parsed expression)
    pub js_ast: Option<JsExpression<'a>>,
    /// Hoisted node reference
    pub hoisted: Option<Box<'a, JsChildNode<'a>>>,
    /// Identifiers declared in this expression
    pub identifiers: Option<Vec<'a, String>>,
    /// Whether this is a handler key
    pub is_handler_key: bool,
    /// Whether this expression has been processed for ref .value transformation
    pub is_ref_transformed: bool,
}

impl<'a> SimpleExpressionNode<'a> {
    pub fn new(content: impl Into<String>, is_static: bool, loc: SourceLocation) -> Self {
        Self {
            content: content.into(),
            is_static,
            const_type: if is_static {
                ConstantType::CanStringify
            } else {
                ConstantType::NotConstant
            },
            loc,
            js_ast: None,
            hoisted: None,
            identifiers: None,
            is_handler_key: false,
            is_ref_transformed: false,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::SimpleExpression
    }
}

/// Placeholder for JavaScript expression AST from OXC
#[derive(Debug)]
pub struct JsExpression<'a> {
    /// Raw expression content (will be replaced with OXC AST)
    pub raw: String,
    _marker: std::marker::PhantomData<&'a ()>,
}

/// Compound expression node (mixed content)
#[derive(Debug)]
pub struct CompoundExpressionNode<'a> {
    pub children: Vec<'a, CompoundExpressionChild<'a>>,
    pub loc: SourceLocation,
    pub identifiers: Option<Vec<'a, String>>,
    pub is_handler_key: bool,
}

impl<'a> CompoundExpressionNode<'a> {
    pub fn new(allocator: &'a Bump, loc: SourceLocation) -> Self {
        Self {
            children: Vec::new_in(allocator),
            loc,
            identifiers: None,
            is_handler_key: false,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::CompoundExpression
    }
}

/// Child of a compound expression
#[derive(Debug)]
pub enum CompoundExpressionChild<'a> {
    Simple(Box<'a, SimpleExpressionNode<'a>>),
    Compound(Box<'a, CompoundExpressionNode<'a>>),
    Interpolation(Box<'a, InterpolationNode<'a>>),
    Text(Box<'a, TextNode>),
    String(String),
    Symbol(RuntimeHelper),
}

//! Root and template child node types.
//!
//! Contains the top-level AST nodes including RootNode and
//! the TemplateChildNode enum that represents all template children.

use vize_carton::{Box, Bump, String, Vec};

use super::{
    codegen::CacheExpression,
    core::{NodeType, SourceLocation, STUB_LOCATION},
    elements::{CommentNode, ElementNode, InterpolationNode, TextNode},
    expressions::CompoundExpressionNode,
    CodegenNode, ForNode, IfBranchNode, IfNode, ImportItem, JsChildNode, RuntimeHelper,
    TextCallNode,
};

/// Root AST node
#[derive(Debug)]
pub struct RootNode<'a> {
    pub children: Vec<'a, TemplateChildNode<'a>>,
    pub helpers: Vec<'a, RuntimeHelper>,
    pub components: Vec<'a, String>,
    pub directives: Vec<'a, String>,
    pub hoists: Vec<'a, Option<JsChildNode<'a>>>,
    pub imports: Vec<'a, ImportItem<'a>>,
    pub cached: Vec<'a, Option<Box<'a, CacheExpression<'a>>>>,
    pub temps: u32,
    pub source: String,
    pub loc: SourceLocation,
    pub codegen_node: Option<CodegenNode<'a>>,
    pub transformed: bool,
}

impl<'a> RootNode<'a> {
    pub fn new(allocator: &'a Bump, source: impl Into<String>) -> Self {
        Self {
            children: Vec::new_in(allocator),
            helpers: Vec::new_in(allocator),
            components: Vec::new_in(allocator),
            directives: Vec::new_in(allocator),
            hoists: Vec::new_in(allocator),
            imports: Vec::new_in(allocator),
            cached: Vec::new_in(allocator),
            temps: 0,
            source: source.into(),
            loc: SourceLocation::STUB,
            codegen_node: None,
            transformed: false,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::Root
    }
}

/// All template child node types
#[derive(Debug)]
pub enum TemplateChildNode<'a> {
    Element(Box<'a, ElementNode<'a>>),
    Text(Box<'a, TextNode>),
    Comment(Box<'a, CommentNode>),
    Interpolation(Box<'a, InterpolationNode<'a>>),
    If(Box<'a, IfNode<'a>>),
    IfBranch(Box<'a, IfBranchNode<'a>>),
    For(Box<'a, ForNode<'a>>),
    TextCall(Box<'a, TextCallNode<'a>>),
    CompoundExpression(Box<'a, CompoundExpressionNode<'a>>),
    /// Reference to a hoisted node (index into root.hoists array)
    Hoisted(usize),
}

impl<'a> TemplateChildNode<'a> {
    pub fn node_type(&self) -> NodeType {
        match self {
            Self::Element(_) => NodeType::Element,
            Self::Text(_) => NodeType::Text,
            Self::Comment(_) => NodeType::Comment,
            Self::Interpolation(_) => NodeType::Interpolation,
            Self::If(_) => NodeType::If,
            Self::IfBranch(_) => NodeType::IfBranch,
            Self::For(_) => NodeType::For,
            Self::TextCall(_) => NodeType::TextCall,
            Self::CompoundExpression(_) => NodeType::CompoundExpression,
            Self::Hoisted(_) => NodeType::SimpleExpression, // Hoisted refs are like expressions
        }
    }

    pub fn loc(&self) -> &SourceLocation {
        match self {
            Self::Element(n) => &n.loc,
            Self::Text(n) => &n.loc,
            Self::Comment(n) => &n.loc,
            Self::Interpolation(n) => &n.loc,
            Self::If(n) => &n.loc,
            Self::IfBranch(n) => &n.loc,
            Self::For(n) => &n.loc,
            Self::TextCall(n) => &n.loc,
            Self::CompoundExpression(n) => &n.loc,
            Self::Hoisted(_) => &STUB_LOCATION, // Hoisted refs don't have a real location
        }
    }
}

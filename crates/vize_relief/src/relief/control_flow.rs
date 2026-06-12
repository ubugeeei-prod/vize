//! Control flow AST node types.
//!
//! Contains if (v-if/v-else-if/v-else), for (v-for),
//! and text call node definitions.

use vize_carton::{Box, Bump, Vec};

use super::{
    codegen::{CacheExpression, CallExpression, ConditionalExpression, VNodeCall},
    core::{NodeType, SourceLocation},
    elements::PropNode,
    expressions::{CompoundExpressionNode, ExpressionNode, SimpleExpressionNode},
    nodes::TemplateChildNode,
};

/// If node (v-if)
#[derive(Debug)]
pub struct IfNode<'a> {
    pub branches: Vec<'a, IfBranchNode<'a>>,
    pub loc: SourceLocation,
    pub codegen_node: Option<IfCodegenNode<'a>>,
}

impl<'a> IfNode<'a> {
    pub fn new(allocator: &'a Bump, loc: SourceLocation) -> Self {
        Self {
            branches: Vec::new_in(allocator),
            loc,
            codegen_node: None,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::If
    }
}

/// If codegen node type
#[derive(Debug)]
pub enum IfCodegenNode<'a> {
    Conditional(Box<'a, ConditionalExpression<'a>>),
    Cache(Box<'a, CacheExpression<'a>>),
}

/// If branch node (v-if, v-else-if, v-else)
#[derive(Debug)]
pub struct IfBranchNode<'a> {
    pub condition: Option<ExpressionNode<'a>>,
    pub children: Vec<'a, TemplateChildNode<'a>>,
    pub user_key: Option<PropNode<'a>>,
    pub is_template_if: bool,
    pub loc: SourceLocation,
}

impl<'a> IfBranchNode<'a> {
    pub fn new(
        allocator: &'a Bump,
        condition: Option<ExpressionNode<'a>>,
        loc: SourceLocation,
    ) -> Self {
        Self {
            condition,
            children: Vec::new_in(allocator),
            user_key: None,
            is_template_if: false,
            loc,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::IfBranch
    }
}

/// For node (v-for)
#[derive(Debug)]
pub struct ForNode<'a> {
    pub source: ExpressionNode<'a>,
    pub value_alias: Option<ExpressionNode<'a>>,
    pub key_alias: Option<ExpressionNode<'a>>,
    pub object_index_alias: Option<ExpressionNode<'a>>,
    pub parse_result: ForParseResult<'a>,
    pub children: Vec<'a, TemplateChildNode<'a>>,
    pub loc: SourceLocation,
    pub codegen_node: Option<Box<'a, VNodeCall<'a>>>,
}

impl<'a> ForNode<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::For
    }
}

/// Parsed result for v-for expression
#[derive(Debug)]
pub struct ForParseResult<'a> {
    pub source: ExpressionNode<'a>,
    pub value: Option<ExpressionNode<'a>>,
    pub key: Option<ExpressionNode<'a>>,
    pub index: Option<ExpressionNode<'a>>,
    pub finalized: bool,
}

/// Text call node
#[derive(Debug)]
pub struct TextCallNode<'a> {
    pub content: TextCallContent<'a>,
    pub loc: SourceLocation,
    pub codegen_node: Option<TextCallCodegenNode<'a>>,
}

impl<'a> TextCallNode<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::TextCall
    }
}

/// Text call content
#[derive(Debug)]
pub enum TextCallContent<'a> {
    Text(Box<'a, super::elements::TextNode>),
    Interpolation(Box<'a, super::elements::InterpolationNode<'a>>),
    Compound(Box<'a, CompoundExpressionNode<'a>>),
}

/// Text call codegen node
#[derive(Debug)]
pub enum TextCallCodegenNode<'a> {
    Call(Box<'a, CallExpression<'a>>),
    Simple(Box<'a, SimpleExpressionNode<'a>>),
}

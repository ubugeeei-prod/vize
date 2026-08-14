//! Expression AST node types.
//!
//! Contains simple and compound expression nodes used in
//! template bindings, directives, and interpolations.

use vize_carton::{Box, Bump, String, Vec};

use super::{
    RuntimeHelper,
    codegen::JsChildNode,
    core::{ConstantType, NodeType, SourceLocation},
    elements::{InterpolationNode, TextNode},
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

/// A template expression's JavaScript AST, parsed once per compile into the
/// shared oxc arena pool (Davinci P1-5).
///
/// `ast` covers the whole of `raw`: the parser only retains complete
/// single-expression parses, so text a lone [`oxc_ast::ast::Expression`]
/// cannot represent (v-for values such as `item of items`, v-on
/// multi-statement bodies such as `a++; b++`, invalid expressions) leaves
/// [`SimpleExpressionNode::js_ast`] as `None` and consumers keep their own
/// handling for those shapes.
///
/// Lifetime contract: `'a` is the compile's arena lifetime, so retained
/// references are per-compile ephemera. Anything crossing a compile boundary
/// (caches, folios, summaries) must convert to an owned form (the expression
/// text) first — the arena/cache contract; never store this reference.
#[derive(Debug)]
pub struct JsExpression<'a> {
    /// Retained oxc AST, allocated in the compile's oxc arena pool.
    pub ast: &'a oxc_ast::ast::Expression<'a>,
    /// The exact text `ast` was parsed from (display slice): the template
    /// source slice where the node content equals it, otherwise an arena
    /// copy of the decoded content (attribute values with entities,
    /// camelized same-name shorthand arguments).
    pub raw: &'a str,
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

//! Expression AST node types.
//!
//! Contains simple and compound expression nodes used in
//! template bindings, directives, and interpolations.

use vize_s0::{Allocator, Box, Vec};

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
    /// Expression text: the template source slice when the content is
    /// verbatim, an arena copy when a transform computed it (Davinci P1-10).
    pub content: &'a str,
    pub is_static: bool,
    pub const_type: ConstantType,
    pub loc: SourceLocation,
    /// Parsed JavaScript AST (None = simple identifier, Some = parsed expression)
    pub js_ast: Option<JsExpression<'a>>,
    /// Hoisted node reference
    pub hoisted: Option<Box<'a, JsChildNode<'a>>>,
    /// Identifiers declared in this expression
    pub identifiers: Option<Vec<'a, &'a str>>,
    /// Whether this is a handler key
    pub is_handler_key: bool,
    /// Whether this expression has been processed for ref .value transformation
    pub is_ref_transformed: bool,
}

/// 120 -> 88: `content` and the `identifiers` vector both shrank.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<SimpleExpressionNode<'_>>() == 88);

impl<'a> SimpleExpressionNode<'a> {
    pub fn new(content: &'a str, is_static: bool, loc: SourceLocation) -> Self {
        Self {
            content,
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

    /// Clone `node`'s content/staticness/location exactly as
    /// [`SimpleExpressionNode::new`] would, carrying the retained AST along
    /// (Davinci P1-7): the clone's content is byte-identical to the
    /// original's, so the parse-once `js_ast` describes it verbatim.
    pub fn from_node(node: &SimpleExpressionNode<'a>) -> Self {
        Self {
            js_ast: node.js_ast,
            ..Self::new(node.content, node.is_static, node.loc.clone())
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
///
/// `Copy`: both fields are shared references, so consumers (P1-6/P1-7) can
/// propagate the retained parse through byte-identical clones of a node
/// without touching the arena.
#[derive(Debug, Clone, Copy)]
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
    pub identifiers: Option<Vec<'a, &'a str>>,
    pub is_handler_key: bool,
}

/// 88 -> 64.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<CompoundExpressionNode<'_>>() == 64);

impl<'a> CompoundExpressionNode<'a> {
    pub fn new(allocator: &'a Allocator, loc: SourceLocation) -> Self {
        Self {
            children: Vec::new_in(&allocator),
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
    Text(Box<'a, TextNode<'a>>),
    String(&'a str),
    Symbol(RuntimeHelper),
}

//! Root and template child node types.
//!
//! Contains the top-level AST nodes including RootNode and
//! the TemplateChildNode enum that represents all template children.

use vize_l0::{Allocator, Box, Vec};

use super::{
    ForNode, IfBranchNode, IfNode, ImportItem, JsChildNode, RuntimeHelper, TextCallNode,
    codegen::CacheExpression,
    core::{NodeType, STUB_LOCATION, SourceLocation},
    elements::{CommentNode, ElementNode, InterpolationNode, TextNode},
    expressions::CompoundExpressionNode,
};

/// Root AST node
pub struct RootNode<'a> {
    pub children: Vec<'a, TemplateChildNode<'a>>,
    pub comments: Vec<'a, CommentNode<'a>>,
    pub helpers: Vec<'a, RuntimeHelper>,
    pub components: Vec<'a, &'a str>,
    pub directives: Vec<'a, &'a str>,
    /// Vue 2 pipe filters referenced by the template (e.g. `capitalize` in
    /// `{{ msg | capitalize }}`), in first-seen order. Each becomes a
    /// `const _filter_<name> = _resolveFilter("<name>")` asset declaration in
    /// the render preamble, mirroring `@vue/compiler-core`'s Vue-2-compat
    /// filter output.
    ///
    /// Legacy-only: filters were removed in Vue 3, so this field is compiled
    /// only behind the internal `_legacy` cargo feature (enabled transitively
    /// by `vize_atelier_core/legacy`). The default Vue 3 build never sees it,
    /// keeping the public AST surface — and `cargo-semver-checks`, whose
    /// feature heuristic skips `_`-prefixed features — byte-identical.
    #[cfg(feature = "_legacy")]
    pub filters: Vec<'a, &'a str>,
    pub hoists: Vec<'a, Option<JsChildNode<'a>>>,
    pub imports: Vec<'a, ImportItem<'a>>,
    pub cached: Vec<'a, Option<Box<'a, CacheExpression<'a>>>>,
    pub temps: u32,
    /// The template text this tree was parsed from.
    pub source: &'a str,
    pub loc: SourceLocation,
    pub transformed: bool,
    /// An attribute or directive name in `source` holds a backslash, as the
    /// lexer saw while reading names. Codegen writes names between double
    /// quotes and escapes them only when this is set.
    pub name_backslash: bool,
}

/// 296 -> 224: eight arena vectors at -8 bytes each plus `source`. The
/// legacy build carries one more arena vector (`filters`), 320 -> 248.
#[cfg(all(target_pointer_width = "64", not(feature = "_legacy")))]
const _: () = assert!(size_of::<RootNode<'_>>() == 224);
#[cfg(all(target_pointer_width = "64", feature = "_legacy"))]
const _: () = assert!(size_of::<RootNode<'_>>() == 248);

/// Reviewed fixtures pin this output byte for byte, so the lexer fact prints
/// only when it is set.
impl core::fmt::Debug for RootNode<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut out = f.debug_struct("RootNode");
        out.field("children", &self.children)
            .field("comments", &self.comments)
            .field("helpers", &self.helpers)
            .field("components", &self.components)
            .field("directives", &self.directives);
        #[cfg(feature = "_legacy")]
        out.field("filters", &self.filters);
        out.field("hoists", &self.hoists)
            .field("imports", &self.imports)
            .field("cached", &self.cached)
            .field("temps", &self.temps)
            .field("source", &self.source)
            .field("loc", &self.loc)
            .field("transformed", &self.transformed);
        if self.name_backslash {
            out.field("name_backslash", &self.name_backslash);
        }
        out.finish()
    }
}

impl<'a> RootNode<'a> {
    pub fn new(allocator: &'a Allocator, source: &'a str) -> Self {
        Self {
            children: Vec::new_in(&allocator),
            comments: Vec::new_in(&allocator),
            helpers: Vec::new_in(&allocator),
            components: Vec::new_in(&allocator),
            directives: Vec::new_in(&allocator),
            #[cfg(feature = "_legacy")]
            filters: Vec::new_in(&allocator),
            hoists: Vec::new_in(&allocator),
            imports: Vec::new_in(&allocator),
            cached: Vec::new_in(&allocator),
            temps: 0,
            source,
            loc: SourceLocation::STUB,
            transformed: false,
            name_backslash: false,
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
    Text(Box<'a, TextNode<'a>>),
    Comment(Box<'a, CommentNode<'a>>),
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

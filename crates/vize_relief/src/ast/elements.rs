//! Element-related AST node types.
//!
//! Contains element, attribute, directive, text, comment,
//! and interpolation node definitions.

use vize_carton::{directive::DirectiveKind, Box, Bump, String, Vec};

use super::{
    codegen::{CacheExpression, VNodeCall},
    control_flow::ForParseResult,
    core::{ElementType, Namespace, NodeType, SourceLocation},
    expressions::{ExpressionNode, SimpleExpressionNode},
};

/// Element node
#[derive(Debug)]
pub struct ElementNode<'a> {
    pub ns: Namespace,
    pub tag: String,
    pub tag_type: ElementType,
    pub props: Vec<'a, PropNode<'a>>,
    pub children: Vec<'a, super::TemplateChildNode<'a>>,
    pub is_self_closing: bool,
    pub loc: SourceLocation,
    pub inner_loc: Option<SourceLocation>,
    pub codegen_node: Option<ElementCodegenNode<'a>>,
    /// If props are hoisted, this is the index into the hoists array (1-based for _hoisted_N)
    pub hoisted_props_index: Option<usize>,
}

impl<'a> ElementNode<'a> {
    pub fn new(allocator: &'a Bump, tag: impl Into<String>, loc: SourceLocation) -> Self {
        Self {
            ns: Namespace::Html,
            tag: tag.into(),
            tag_type: ElementType::Element,
            props: Vec::new_in(allocator),
            children: Vec::new_in(allocator),
            is_self_closing: false,
            loc,
            inner_loc: None,
            codegen_node: None,
            hoisted_props_index: None,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::Element
    }
}

/// Element codegen node (VNodeCall, SimpleExpression, CacheExpression, etc.)
#[derive(Debug)]
pub enum ElementCodegenNode<'a> {
    VNodeCall(Box<'a, VNodeCall<'a>>),
    SimpleExpression(Box<'a, SimpleExpressionNode<'a>>),
    CacheExpression(Box<'a, CacheExpression<'a>>),
}

/// Prop node (attribute or directive)
#[derive(Debug)]
pub enum PropNode<'a> {
    Attribute(Box<'a, AttributeNode>),
    Directive(Box<'a, DirectiveNode<'a>>),
}

impl<'a> PropNode<'a> {
    pub fn loc(&self) -> &SourceLocation {
        match self {
            Self::Attribute(n) => &n.loc,
            Self::Directive(n) => &n.loc,
        }
    }
}

/// Attribute node
#[derive(Debug)]
pub struct AttributeNode {
    pub name: String,
    pub name_loc: SourceLocation,
    pub value: Option<TextNode>,
    pub loc: SourceLocation,
}

impl AttributeNode {
    pub fn new(name: impl Into<String>, loc: SourceLocation) -> Self {
        Self {
            name: name.into(),
            name_loc: loc.clone(),
            value: None,
            loc,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::Attribute
    }
}

/// Directive node (v-if, v-for, v-bind, etc.)
#[derive(Debug)]
pub struct DirectiveNode<'a> {
    /// Normalized directive name without prefix (e.g., "if", "for", "bind")
    pub name: String,
    /// Raw attribute name including shorthand (e.g., "@click", ":class")
    pub raw_name: Option<String>,
    /// Directive expression
    pub exp: Option<ExpressionNode<'a>>,
    /// Directive argument (e.g., "click" in @click)
    pub arg: Option<ExpressionNode<'a>>,
    /// Directive modifiers (e.g., ["stop", "prevent"] in @click.stop.prevent)
    pub modifiers: Vec<'a, SimpleExpressionNode<'a>>,
    /// Parsed result for v-for
    pub for_parse_result: Option<ForParseResult<'a>>,
    /// Whether this is a Vue 3.4+ same-name shorthand (`:foo` without value)
    pub shorthand: bool,
    pub loc: SourceLocation,
}

impl<'a> DirectiveNode<'a> {
    pub fn new(allocator: &'a Bump, name: impl Into<String>, loc: SourceLocation) -> Self {
        Self {
            name: name.into(),
            raw_name: None,
            exp: None,
            arg: None,
            modifiers: Vec::new_in(allocator),
            for_parse_result: None,
            shorthand: false,
            loc,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::Directive
    }
}

/// Text node
#[derive(Debug)]
pub struct TextNode {
    pub content: String,
    pub loc: SourceLocation,
}

impl TextNode {
    pub fn new(content: impl Into<String>, loc: SourceLocation) -> Self {
        Self {
            content: content.into(),
            loc,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::Text
    }
}

/// Comment node
#[derive(Debug)]
pub struct CommentNode {
    pub content: String,
    pub loc: SourceLocation,
    /// Parsed `@vize:` directive, if this comment contains one.
    pub directive: Option<DirectiveKind>,
}

impl CommentNode {
    pub fn new(content: impl Into<String>, loc: SourceLocation) -> Self {
        Self {
            content: content.into(),
            loc,
            directive: None,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::Comment
    }
}

/// Interpolation node ({{ expr }})
#[derive(Debug)]
pub struct InterpolationNode<'a> {
    pub content: ExpressionNode<'a>,
    pub loc: SourceLocation,
}

impl<'a> InterpolationNode<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::Interpolation
    }
}

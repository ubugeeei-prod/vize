//! Element-related AST node types.
//!
//! Contains element, attribute, directive, text, comment,
//! and interpolation node definitions.

use vize_s0::{Allocator, Box, Vec, directive::DirectiveKind};

use super::{
    control_flow::ForParseResult,
    core::{ElementType, Namespace, NodeType, SourceLocation},
    expressions::{ExpressionNode, SimpleExpressionNode},
};

/// Element node
#[derive(Debug)]
pub struct ElementNode<'a> {
    pub ns: Namespace,
    /// Tag text exactly as authored: a slice of the template source, so the
    /// common case allocates nothing (Davinci P1-10).
    pub tag: &'a str,
    pub tag_type: ElementType,
    pub props: Vec<'a, PropNode<'a>>,
    pub children: Vec<'a, super::TemplateChildNode<'a>>,
    pub is_self_closing: bool,
    pub loc: SourceLocation,
    pub inner_loc: Option<SourceLocation>,
    /// If props are hoisted, this is the index into the hoists array (1-based for _hoisted_N)
    pub hoisted_props_index: Option<usize>,
}

/// Node footprints are pinned: the P1-10 string diet traded every owned
/// `CompactString` field (24 bytes) for an `&'a str` (16) and every arena
/// container for oxc's (32 -> 24 bytes per `Vec`). `ElementNode` 128 -> 104.
///
/// Every pinned figure in this crate is a 64-bit footprint, so the assertions
/// only apply where pointers are 8 bytes wide (the wasm32 build is 32-bit).
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<ElementNode<'_>>() == 104);

impl<'a> ElementNode<'a> {
    pub fn new(allocator: &'a Allocator, tag: &'a str, loc: SourceLocation) -> Self {
        Self {
            ns: Namespace::Html,
            tag,
            tag_type: ElementType::Element,
            props: Vec::new_in(&allocator),
            children: Vec::new_in(&allocator),
            is_self_closing: false,
            loc,
            inner_loc: None,
            hoisted_props_index: None,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::Element
    }
}

/// Prop node (attribute or directive)
#[derive(Debug)]
pub enum PropNode<'a> {
    Attribute(Box<'a, AttributeNode<'a>>),
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
pub struct AttributeNode<'a> {
    /// Attribute name exactly as authored: a slice of the template source.
    pub name: &'a str,
    pub name_loc: SourceLocation,
    pub value: Option<TextNode<'a>>,
    pub loc: SourceLocation,
}

/// 80 -> 56 (name + the nested text node's content).
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<AttributeNode<'_>>() == 56);

impl<'a> AttributeNode<'a> {
    pub fn new(name: &'a str, loc: SourceLocation) -> Self {
        Self {
            name,
            name_loc: SourceLocation::default(),
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
    /// Normalized directive name without prefix (e.g., "if", "for", "bind").
    /// An atom: the shorthand forms normalize to `'static` names and `v-x`
    /// forms slice the source, so this never allocates (Davinci P1-10).
    pub name: &'a str,
    /// Raw attribute name including shorthand (e.g., "@click", ":class"):
    /// a slice of the template source.
    pub raw_name: Option<&'a str>,
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

/// 208 -> 176 (`name`, `raw_name`, and the modifiers vector).
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<DirectiveNode<'_>>() == 176);

impl<'a> DirectiveNode<'a> {
    pub fn new(allocator: &'a Allocator, name: &'a str, loc: SourceLocation) -> Self {
        Self {
            name,
            raw_name: None,
            exp: None,
            arg: None,
            modifiers: Vec::new_in(&allocator),
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
pub struct TextNode<'a> {
    /// Text content: the template source slice when the run is verbatim, an
    /// arena copy when entity decoding or whitespace condensing rewrote it.
    pub content: &'a str,
    pub loc: SourceLocation,
}

/// 32 -> 24.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<TextNode<'_>>() == 24);

impl<'a> TextNode<'a> {
    pub fn new(content: &'a str, loc: SourceLocation) -> Self {
        Self { content, loc }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::Text
    }
}

/// Comment node
#[derive(Debug)]
pub struct CommentNode<'a> {
    /// Comment body: a slice of the template source.
    pub content: &'a str,
    pub loc: SourceLocation,
    pub kind: CommentKind,
    /// Parsed `@vize:` directive, if this comment contains one.
    pub directive: Option<DirectiveKind>,
}

/// Source-level comment forms recognized by the template parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    Html,
    InTag,
}

/// 40 -> 32.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<CommentNode<'_>>() == 32);

impl<'a> CommentNode<'a> {
    pub fn new(content: &'a str, loc: SourceLocation) -> Self {
        Self {
            content,
            loc,
            kind: CommentKind::Html,
            directive: None,
        }
    }

    pub fn new_in_tag(content: &'a str, loc: SourceLocation) -> Self {
        Self {
            content,
            loc,
            kind: CommentKind::InTag,
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
    /// Raw-HTML (unescaped) interpolation produced by Vue 1.x triple-mustache
    /// syntax (`{{{ html }}}`), the pre-Vue-2 equivalent of `v-html`. When set,
    /// codegen emits the expression directly instead of escaping it through
    /// `_toDisplayString`, matching Vue 1's unescaped output.
    ///
    /// Legacy-only: triple-mustache was removed in Vue 2, so this field is
    /// compiled only behind the internal `_legacy` cargo feature (enabled
    /// transitively by `vize_atelier_core/legacy` / `vize_armature/legacy`). The
    /// default Vue 3 build never sees it, keeping the public AST surface — and
    /// `cargo-semver-checks`, whose feature heuristic skips `_`-prefixed
    /// features — byte-identical.
    #[cfg(feature = "_legacy")]
    pub raw: bool,
}

impl<'a> InterpolationNode<'a> {
    pub fn node_type(&self) -> NodeType {
        NodeType::Interpolation
    }
}

//! Element-related AST node types.
//!
//! Contains element, attribute, directive, text, comment,
//! and interpolation node definitions.

use vize_carton::{Box, Bump, String, Vec, directive::DirectiveKind, ensure_sufficient_stack};

use super::{
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
    /// Preserve an upstream compiler's explicit custom-element classification.
    pub is_custom_element: bool,
    pub loc: SourceLocation,
    pub inner_loc: Option<SourceLocation>,
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
            is_custom_element: false,
            loc,
            inner_loc: None,
            hoisted_props_index: None,
        }
    }

    pub fn node_type(&self) -> NodeType {
        NodeType::Element
    }
}

/// Tearing an element down is itself a recursive walk of its subtree.
///
/// Without this, the compiler-generated drop glue chains
/// `Vec<TemplateChildNode>` -> `Box<ElementNode>` -> `Vec<TemplateChildNode>`
/// once per nesting level, on the machine stack, with no guard — so a template
/// deep enough would abort the process on the way *out* of a compile that had
/// just succeeded. Dropping the children here, inside a checked frame, puts the
/// teardown under the same stack-growth guarantee as the passes that built the
/// tree (`vize_carton::recursion`).
///
/// Leaf elements — the overwhelming majority — pay one branch and nothing else.
impl Drop for ElementNode<'_> {
    fn drop(&mut self) {
        if self.children.is_empty() {
            return;
        }
        // `clear` runs the children's destructors here; the implicit field drop
        // that follows this function then sees an empty vector and recurses no
        // further.
        ensure_sufficient_stack(|| self.children.clear());
    }
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

impl CommentNode {
    pub fn new(content: impl Into<String>, loc: SourceLocation) -> Self {
        Self {
            content: content.into(),
            loc,
            kind: CommentKind::Html,
            directive: None,
        }
    }

    pub fn new_in_tag(content: impl Into<String>, loc: SourceLocation) -> Self {
        Self {
            content: content.into(),
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

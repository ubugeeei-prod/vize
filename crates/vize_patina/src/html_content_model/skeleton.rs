//! The render skeleton: the statically known element structure one template
//! renders, with Vue's transparent constructs erased.
//!
//! `<template v-if/v-for/v-slot>` wrappers, `v-if` chains, `v-for`, and the
//! render-in-place built-ins (`Transition`, `KeepAlive`, `Suspense`, …) are
//! erased: their children sit directly under the nearest rendered ancestor,
//! which is where the HTML serializer puts them. Everything whose rendered
//! position or content the template does not determine becomes an explicit
//! node kind — never a silent omission — so the checker can answer `unknown`
//! for it instead of guessing.

use vize_s0::{CompactString, Span};

use super::facts::{Attr, ElemId, Ns};
use super::tri::Tri;

/// Why a subtree's rendered context is not determined by this template.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoundaryKind {
    /// `<Teleport>`: children render at the teleport target.
    Teleport,
    /// `<component :is>`, a `vue:`-prefixed `is`, or a dynamic
    /// `TransitionGroup` tag: the rendered element is not static.
    DynamicComponent,
    /// A native `<template>` element: its children form a separate
    /// `DocumentFragment` parsed in the "in template" insertion mode.
    TemplateContents,
    /// `<noscript>`: tokenized as raw text only when scripting is enabled,
    /// which the template does not determine.
    ScriptingDependent,
    /// Children of `html`, `head`, `body` and `frameset`: document-level
    /// insertion modes outside the checker's body-content domain.
    DocumentStructure,
}

/// Static knowledge of the attributes the fact table's conditions read.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AttrFacts {
    yes: u8,
    maybe: u8,
}

impl AttrFacts {
    /// Record a statically present attribute.
    pub fn set_yes(&mut self, attr: Attr) {
        self.yes |= 1 << attr as u8;
    }

    /// Record an attribute whose presence depends on a dynamic binding.
    pub fn set_maybe(&mut self, attr: Attr) {
        self.maybe |= 1 << attr as u8;
    }

    /// Record an object spread (`v-bind="attrs"`): every attribute unknown.
    pub fn set_all_maybe(&mut self) {
        self.maybe = u8::MAX;
    }

    /// The three-valued presence of `attr`.
    pub fn get(&self, attr: Attr) -> Tri {
        let bit = 1 << attr as u8;
        if self.yes & bit != 0 {
            Tri::Yes
        } else if self.maybe & bit != 0 {
            Tri::Maybe
        } else {
            Tri::No
        }
    }
}

/// A statically known element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    /// The tag as authored.
    pub tag: CompactString,
    /// The tag name the HTML tokenizer produces: ASCII-lowercased.
    pub name: CompactString,
    /// Fact-table ids of the tag in the HTML, SVG and MathML namespaces.
    pub ids: [Option<ElemId>; 3],
    /// Attribute facts.
    pub attrs: AttrFacts,
    /// Span of the tag name in the start tag (`div` in `<div class>`).
    pub name_span: Span,
    /// Whether `v-html`/`v-text` (or a bound `innerHTML`/`textContent`)
    /// replaces the children with content this template does not show.
    pub dynamic_content: bool,
    /// The namespace Vue's compiler creates the element in. The checker
    /// derives namespaces from the parser's rules wherever the chain is
    /// known; where it is not (a template root, slot content) this is the
    /// declared mount assumption: the element sits where its compiled
    /// namespace holds.
    pub compiler_ns: Ns,
}

impl Element {
    /// A new element with the given tag, looking up its fact-table ids.
    pub fn new(tag: &str, name_span: Span) -> Self {
        Self {
            tag: CompactString::new(tag),
            name: if tag.bytes().any(|byte| byte.is_ascii_uppercase()) {
                CompactString::new(tag.to_ascii_lowercase())
            } else {
                CompactString::new(tag)
            },
            ids: super::tag_ids::tag_ids(tag),
            attrs: AttrFacts::default(),
            name_span,
            dynamic_content: false,
            compiler_ns: Ns::Html,
        }
    }

    /// The fact-table id of this tag in `ns`.
    #[inline]
    pub fn id(&self, ns: Ns) -> Option<ElemId> {
        self.ids[ns as usize]
    }
}

/// One skeleton node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    /// A statically known element.
    Element(Element),
    /// Static text; `whitespace_only` when it is inter-element whitespace.
    Text {
        /// Whether every character is ASCII whitespace.
        whitespace_only: bool,
    },
    /// Text whose content is an expression (`{{ }}`): its emptiness is unknown.
    DynamicText,
    /// A subtree rendered at a place this template does not determine.
    Boundary(BoundaryKind),
    /// A component usage; its children are [`NodeKind::SlotContent`] groups.
    Component {
        /// The component tag as authored.
        name: CompactString,
    },
    /// Content passed to a component slot (child of [`NodeKind::Component`]).
    SlotContent {
        /// Slot name (`default` for the default slot, `None` when dynamic).
        name: Option<CompactString>,
    },
    /// A `<slot>` outlet; its children are the fallback content, rendered in
    /// place when the parent passes nothing.
    SlotOutlet {
        /// Slot name (`default` when omitted, `None` when dynamic).
        name: Option<CompactString>,
    },
}

/// A skeleton node in pre-order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// What this node is.
    pub kind: NodeKind,
    /// Source span of the whole node.
    pub span: Span,
    /// Index one past the last node of this node's subtree.
    pub end: u32,
}

/// What composition may prune with (Davinci FP-3); empty unless the skeleton
/// was built composable.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PropFacts {
    /// `(node, identifier)`: the node renders only when the identifier — the
    /// component's own binding, no scope variable — is truthy (`v-if="copy"`).
    pub guards: Vec<(u32, CompactString)>,
    /// `(component node, prop)`: a prop the usage passes, camelized.
    pub passed: Vec<(u32, CompactString)>,
    /// Component usages whose passed props are not all known.
    pub opaque: Vec<u32>,
}

/// A template's render skeleton, in pre-order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Skeleton {
    /// Nodes in pre-order; a node's subtree is `index + 1 .. node.end`.
    pub nodes: Vec<Node>,
    /// Guards and passed props, for composition.
    pub props: PropFacts,
}

impl Skeleton {
    /// Whether the usage at `usage` passes `prop`: `None` when it may.
    pub fn passes(&self, usage: u32, prop: &str) -> Option<bool> {
        if self.props.opaque.contains(&usage) {
            return None;
        }
        Some(
            self.props
                .passed
                .iter()
                .any(|(node, name)| *node == usage && name == prop),
        )
    }

    /// Indices of the root nodes.
    pub fn roots(&self) -> Children<'_> {
        Children {
            skeleton: self,
            next: 0,
            end: self.nodes.len() as u32,
        }
    }

    /// Indices of `index`'s children.
    pub fn children(&self, index: u32) -> Children<'_> {
        Children {
            skeleton: self,
            next: index + 1,
            end: self.nodes[index as usize].end,
        }
    }

    /// The node at `index`.
    #[inline]
    pub fn node(&self, index: u32) -> &Node {
        &self.nodes[index as usize]
    }

    /// Open a node; its children are the nodes pushed until [`Self::close`].
    pub fn open(&mut self, kind: NodeKind, span: Span) -> u32 {
        let index = self.nodes.len() as u32;
        self.nodes.push(Node {
            kind,
            span,
            end: index + 1,
        });
        index
    }

    /// Close the node opened at `index`.
    pub fn close(&mut self, index: u32) {
        self.nodes[index as usize].end = self.nodes.len() as u32;
    }

    /// Push a childless node.
    pub fn leaf(&mut self, kind: NodeKind, span: Span) -> u32 {
        let index = self.open(kind, span);
        self.close(index);
        index
    }
}

/// Iterator over sibling node indices.
#[derive(Debug, Clone)]
pub struct Children<'s> {
    skeleton: &'s Skeleton,
    next: u32,
    end: u32,
}

impl Iterator for Children<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.next >= self.end {
            return None;
        }
        let current = self.next;
        self.next = self.skeleton.node(current).end;
        Some(current)
    }
}

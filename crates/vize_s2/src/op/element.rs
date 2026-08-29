//! `ui.element` / `ui.component` payloads and the attribute type they (and
//! [`super::ModelOp`]) share.

use vize_s0::{Span, Vec};

use super::{BindingOp, Region};

/// Markup namespace of a native element.
///
/// Semantic, not syntactic: the namespace changes how a target creates the
/// node (`createElementNS`, SSR serialization), so it survives into S2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Namespace {
    /// The HTML namespace (the default; elided from folio lines).
    #[default]
    Html,
    /// The SVG namespace.
    Svg,
    /// The MathML namespace.
    MathMl,
}

/// A static attribute: authored name, optional authored value.
///
/// Dynamic bindings are ops, not attributes - the normalized one-way ops
/// (`ui.bind`, `ui.on`) land with the transform that needs them (P2-9).
/// Attributes also carry [`super::ModelOp`]'s element kind and dialect
/// modifiers, which is why the type is shared rather than element-private.
#[derive(Debug)]
pub struct Attribute<'a> {
    /// Attribute name, a slice of the source (or of a lowering's arena for
    /// synthesized attributes such as `ui.model`'s `element-kind`).
    pub name: &'a str,
    /// The authored value; `None` for a bare boolean attribute.
    pub value: Option<&'a str>,
    /// Where the attribute came from; synthesized attributes carry their
    /// owner op's span.
    pub span: Span,
}

/// `ui.element` - a native element.
///
/// Grouping is fixed: attributes, then attached bindings, then the owned
/// children region. The folio page prints exactly that order.
#[derive(Debug)]
pub struct ElementOp<'a> {
    /// Tag exactly as authored, a slice of the source.
    pub tag: &'a str,
    /// Markup namespace.
    pub namespace: Namespace,
    /// Static attributes, in authored order.
    pub attributes: Vec<'a, Attribute<'a>>,
    /// Attached bindings, in authored order.
    pub bindings: Vec<'a, BindingOp<'a>>,
    /// The children region this op owns.
    pub children: Region<'a>,
    /// The element's full source range.
    pub span: Span,
}

/// `ui.component` - a component reference.
///
/// Same neutral surface as [`ElementOp`] minus the markup namespace; what
/// the name resolves to is a lowering concern, never S2's.
#[derive(Debug)]
pub struct ComponentOp<'a> {
    /// Component name exactly as authored, a slice of the source.
    pub name: &'a str,
    /// Static attributes, in authored order.
    pub attributes: Vec<'a, Attribute<'a>>,
    /// Attached bindings, in authored order.
    pub bindings: Vec<'a, BindingOp<'a>>,
    /// The children region this op owns. Grouping children into named
    /// slot regions is left to the lowering that needs it (P2-8).
    pub children: Region<'a>,
    /// The component's full source range.
    pub span: Span,
}

/// See [`super`] for the guard rationale; figures move when P2-5b lands.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<Attribute<'_>>() == 40);
    assert!(core::mem::size_of::<ElementOp<'_>>() == 104);
    assert!(core::mem::size_of::<ComponentOp<'_>>() == 96);
};

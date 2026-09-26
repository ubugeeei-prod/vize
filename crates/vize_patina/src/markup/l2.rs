//! The L2 (Disegno) backend of the markup facade (Davinci P4-7a).
//!
//! [`L2Markup`] is a borrowed view over one lowered artifact: the L2 op tree,
//! the lowering-published text-run facts that split a merged run back into
//! its parts ([`TextParts`]), and — for SFC templates — the L1 surface tree,
//! the lossless record surface-shaped queries read instead of re-scanning
//! source: authored opening-tag items L2 consumed or dropped (a `<slot>`
//! outlet's `name`, a `v-if` carrier's static `key`) and the `<template>`
//! carriers L2 unwraps into their `ui.if` / `ui.for` regions.
//!
//! Facade values borrow the op tree or a side-table entry. Merged-run lookup
//! uses one span index of the text side table's page-order ids, built when
//! the view is built, so a walk does not scan that table per interpolation.

use vize_atelier_jsx::l2::JsxL2Root;
use vize_davinci::side_table::SideTable;
use vize_l0::Allocator;
use vize_l1::{SurfaceError, SurfaceTree};
use vize_l1_to_l2::lower::TextParts;
use vize_l1_to_l2::{Lowered, lower_preserving_comments};
use vize_l2::op::{Attribute, BindingOp, ComponentOp, ElementOp, Op, Region, SlotOp};

pub(super) mod binding;
pub(super) mod bound;
pub(super) mod children;
pub(super) mod surface;
pub(super) mod texts;
pub(super) mod walk;

/// Which input dialect an L2 artifact was lowered from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum L2Origin {
    /// A Vue template, through the L1→L2 lowering.
    Template,
    /// A JSX/TSX render root, through the P2-16 projection.
    Jsx,
}

/// A borrowed L2 artifact the markup facade views.
#[derive(Clone, Copy)]
pub struct L2Markup<'a> {
    pub(super) root: &'a Region<'a>,
    pub(super) source: &'a str,
    pub(super) op_count: u32,
    origin: L2Origin,
    pub(super) texts: Option<&'a SideTable<TextParts>>,
    /// Span index of [`texts`](Self::texts), one entry per compound run.
    text_index: &'a [texts::TextSpan],
    /// The artifact's compile arena, where a merged run's entity-bearing static
    /// part is decoded on demand (the one L2 decoder, never a second reading).
    allocator: Option<&'a Allocator>,
    pub(super) authored: Option<&'a SurfaceTree<'a>>,
    pub(super) surface: Option<&'a SurfaceTree<'a>>,
}

impl<'a> L2Markup<'a> {
    /// View a lowered Vue template and the L1 tree it was lowered from.
    pub fn from_lowered(lowered: &'a Lowered<'a>, surface: &'a SurfaceTree<'a>) -> Self {
        Self {
            root: &lowered.root,
            source: lowered.source,
            op_count: lowered.op_count,
            origin: L2Origin::Template,
            texts: Some(&lowered.texts),
            text_index: texts::index(lowered.allocator, &lowered.texts),
            allocator: Some(lowered.allocator),
            surface: Some(surface),
            authored: None,
        }
    }

    /// View one JSX/TSX render root's P2-16 L2 projection.
    pub fn from_projected_root(root: &'a JsxL2Root<'a>) -> Self {
        Self {
            root: &root.root,
            source: root.source,
            op_count: root.op_count,
            origin: L2Origin::Jsx,
            texts: None,
            text_index: texts::EMPTY,
            allocator: None,
            surface: None,
            authored: None,
        }
    }

    /// Whether the artifact was lowered from a Vue template.
    pub const fn is_template(&self) -> bool {
        matches!(self.origin, L2Origin::Template)
    }

    /// Number of ops the artifact numbers in page order.
    pub const fn op_count(&self) -> u32 {
        self.op_count
    }

    /// The recorded parts of a merged text/interpolation run.
    ///
    /// The side table is keyed by the compound op's page-order id. The view
    /// resolves that id through the span index built in [`Self::from_lowered`].
    pub(super) fn text_parts(&self, span: vize_l0::Span) -> Option<&'a TextParts> {
        let texts = self.texts?;
        texts::lookup(self.text_index, texts, span)
    }

    /// A merged run's static part as rendered text: the recorded part text,
    /// entity-decoded with the L2 decoder when it carries an entity.
    pub(super) fn static_part_text(&self, text: &'a str) -> &'a str {
        match self.allocator {
            Some(allocator) if text.contains('&') => {
                allocator.alloc_str(vize_l1_to_l2::emit::decode_html_entities(text).as_str())
            }
            _ => text,
        }
    }

    /// An authored static attribute value as the template parser presents
    /// it: entity-decoded with the tokenizer's attribute-context rules.
    pub(super) fn decode_attribute(&self, value: &'a str) -> &'a str {
        match self.allocator {
            Some(allocator) if value.contains('&') => allocator
                .alloc_str(vize_l1_to_l2::emit::decode_html_attribute_entities(value).as_str()),
            _ => value,
        }
    }

    pub(super) fn frozen_attribute_name(&self, name: &'a str) -> &'a str {
        match self.allocator {
            Some(allocator) => vize_l1_to_l2::lower::frozen_attribute_name(allocator, name),
            None => name,
        }
    }

    /// The range a template reports an element at: its opening tag (the
    /// Relief parse records element locations as opening tags, and the lint
    /// lane's diagnostics address them). JSX projections keep the op span,
    /// which is the lowered root's own location.
    pub(super) fn open_tag_range(&self, span: vize_l0::Span) -> crate::ir::ByteRange {
        let Some(element) = self
            .surface
            .and_then(|tree| surface::element_at(tree, span.start))
        else {
            return super::l2_range(span);
        };
        let gt = element.open.gt.text;
        let end = surface::offset_in(self.source, gt) + gt.len() as u32;
        crate::ir::ByteRange::new(span.start, end)
    }

    /// The range of a scope: from its first carrier's start to the end of its
    /// last carrier's reported range.
    pub(super) fn scope_range(
        &self,
        span: vize_l0::Span,
        last_carrier: vize_l0::Span,
    ) -> crate::ir::ByteRange {
        if self.surface.is_none() {
            return super::l2_range(span);
        }
        crate::ir::ByteRange::new(span.start, self.open_tag_range(last_carrier).end)
    }

    /// The authored `<template>` carrier a `ui.if` branch or `ui.for` region
    /// was unwrapped from: present when the region is not exactly the one
    /// element op the scope's span covers and L1 records an element there.
    pub(super) fn unwrapped_carrier(
        &self,
        span: vize_l0::Span,
        region: &'a [Op<'a>],
    ) -> Option<&'a vize_l1::Element<'a>> {
        if let [op] = region
            && L2ElementOp::from_op(op).is_some_and(|element| element.span() == span)
        {
            return None;
        }
        if let [Op::For(for_op)] = region
            && for_op.span == span
        {
            return None;
        }
        surface::element_at(self.surface?, span.start)
    }
}

/// An owned L1 parse plus its comment-preserving L2 lowering, for callers that
/// start from template source.
pub struct L2Template<'a> {
    tree: SurfaceTree<'a>,
    authored: Option<SurfaceTree<'a>>,
    errors: vize_l0::Vec<'a, SurfaceError>,
    lowered: Lowered<'a>,
}

impl<'a> L2Template<'a> {
    /// Parse `source` into L1 and lower it into L2, keeping authored comments
    /// as `ui.comment` ops (the child list the lint parser sees).
    pub fn lower(allocator: &'a Allocator, source: &'a str) -> Self {
        let (tree, authored, errors) = vize_l1::parse_with_authored(allocator, source);
        let lowered = lower_preserving_comments(allocator, &tree, &errors);
        Self {
            tree,
            authored,
            errors,
            lowered,
        }
    }

    /// The borrowed facade view of this artifact.
    pub fn markup(&self) -> L2Markup<'_> {
        let mut markup = L2Markup::from_lowered(&self.lowered, &self.tree);
        markup.authored = self.authored.as_ref().or(Some(&self.tree));
        markup
    }

    /// The tokenizer errors the L1 parse reported.
    pub fn surface_errors(&self) -> &[SurfaceError] {
        &self.errors
    }

    /// The lowered artifact.
    pub fn lowered(&self) -> &Lowered<'a> {
        &self.lowered
    }

    /// The L1 surface tree the artifact was lowered from.
    pub fn surface(&self) -> &SurfaceTree<'a> {
        &self.tree
    }
}

/// The element-shaped L2 ops the facade presents as a [`super::MarkupElement`].
#[derive(Clone, Copy)]
pub(super) enum L2ElementOp<'a> {
    Element(&'a ElementOp<'a>),
    Component(&'a ComponentOp<'a>),
    Slot(&'a SlotOp<'a>),
}

impl<'a> L2ElementOp<'a> {
    pub(super) fn from_op(op: &'a Op<'a>) -> Option<Self> {
        match op {
            Op::Element(element) => Some(Self::Element(element)),
            Op::Component(component) => Some(Self::Component(component)),
            Op::Slot(slot) => Some(Self::Slot(slot)),
            Op::Text(_) | Op::Interpolation(_) | Op::Comment(_) | Op::If(_) | Op::For(_) => None,
        }
    }

    pub(super) fn tag(self) -> &'a str {
        match self {
            Self::Element(element) => element.tag,
            Self::Component(component) => component.name,
            Self::Slot(_) => "slot",
        }
    }

    pub(super) fn span(self) -> vize_l0::Span {
        match self {
            Self::Element(element) => element.span,
            Self::Component(component) => component.span,
            Self::Slot(slot) => slot.span,
        }
    }

    pub(super) fn attributes(self) -> &'a [Attribute<'a>] {
        match self {
            Self::Element(element) => &element.attributes,
            Self::Component(component) => &component.attributes,
            Self::Slot(slot) => &slot.attributes,
        }
    }

    pub(super) fn bindings(self) -> &'a [BindingOp<'a>] {
        match self {
            Self::Element(element) => &element.bindings,
            Self::Component(component) => &component.bindings,
            Self::Slot(slot) => &slot.bindings,
        }
    }

    pub(super) fn children(self) -> &'a [Op<'a>] {
        match self {
            Self::Element(element) => &element.children.ops,
            Self::Component(component) => &component.children.ops,
            Self::Slot(slot) => &slot.fallback.ops,
        }
    }

    /// Whether the op is an owner the lowering synthesized rather than one the
    /// author wrote: the implicit `tbody` / `tr` table owners HTML tree
    /// construction inserts (the Relief parse inserts them too). In a
    /// template an authored tag is always a slice of the lowered source; a
    /// synthesized one is a static literal. (JSX projections synthesize
    /// nothing.)
    pub(super) fn is_synthesized(self, doc: &L2Markup<'_>) -> bool {
        let Self::Element(element) = self else {
            return false;
        };
        if !doc.is_template() {
            return false;
        }
        let tag = element.tag.as_ptr() as usize;
        let source = doc.source;
        let start = source.as_ptr() as usize;
        tag < start || tag > start + source.len()
    }
}

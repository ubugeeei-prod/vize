// The lowering entry: L1 surface tree in, L2 artifact out — total, with no
// rollback. Module documentation remains on the compatibility owner.

use alloc::vec::Vec as StdVec;
use core::fmt;

use vize_l0::diag::Diagnostic;
use vize_l0::side_table::SideTable;
use vize_l0::{Allocator, SourceBlock, SourceRoot, Span, String};
use vize_l1::markup::{SurfaceError, SurfaceTree};

use vize_l2::op::Region;
use vize_l2::provenance::{ProvenanceRecord, ProvenanceSink};
use vize_l2::scope::ScopeFacts;

#[cfg(any())]
mod caps;
#[cfg(any())]
mod cx;
#[cfg(any())]
mod frozen;

pub use bindop::{BindValue, bind_value};
pub use caps::LegacyCaps;
pub use element::identity::{children_ns, enter_ns};
pub use element::{BranchKind, Structure, frozen_attribute_name, structure};
pub use features::{LoweringFeatures, OpFamily};
pub use foreign::{ForeignDialect, lower_source_block_with_foreign_expressions};
#[doc(hidden)]
pub use frozen::lower_with_frozen_element_spans;
pub use if_keys::{BranchKey, BranchKeyKind, IfFacts, SAME_KEY_MESSAGE};

// The one-scanner rule (#4365): the L2 passes re-derive binding names
// with exactly the enumeration the lowering used, never a second one.
pub use css::{lower_style_block, lower_style_block_in};
pub(crate) use expr::simple_identifier;
pub use forop::{ForName, ForParts, v_for_value};
pub use structural::{
    ForWrapper, IfChain, WrapperAttr, WrapperClass, WrapperKey, WrapperKeys, if_chain,
};
pub use vfor::split_v_for_value;
// The one-rebuild rule (the same discipline): compound text facts and
// their opaque display spelling share one construction rule.
pub use text::{
    RunShape, TextAction, TextLane, TextPart, TextParts, TextRun, TextSource, branch_gap_text,
    branch_gap_text_at, rebuild_source,
};
pub(crate) use text::{legacy_slot_filler_needs_props_placeholder, legacy_slot_filler_text};

/// Whether directive values carry `name: expression` clauses instead of a
/// separate argument token. Kept in the physically Vue-owned policy module so
/// neutral lowering modules consume a resolved fact without naming a version.
pub(super) fn directive_values_are_clauses(
    cx: &cx::Cx<'_, impl vize_l2::provenance::ProvenanceSink>,
) -> bool {
    cx.caps.directive_arg_style == vize_l1::dialect::DirectiveArgStyle::Clause
}

/// The L2 artifact one lowering produces: the op tree plus the three
/// fact channels, all live even when diagnostics are present (the
/// Lean-InfoTree survival property — kept fragments, not rollback).
/// `P` is the provenance sink the consumer chose; by default every
/// decision is kept.
pub struct Lowered<'a, P = StdVec<ProvenanceRecord>> {
    /// The compile arena this artifact was lowered into. The legalizing
    /// pass allocates rewritten expressions and inserted listeners here.
    pub allocator: &'a Allocator,
    /// The complete authored source that L0 spans are measured against.
    pub source: &'a str,
    /// The root region's ops, in document order.
    pub root: Region<'a>,
    /// How many ops were numbered (page order: every region op and
    /// attached binding, top to bottom). Equals the folio's `ops=`
    /// header.
    pub op_count: u32,
    /// Surface diagnostics (the converted tokenizer errors) followed by
    /// the lowering's own, in decision order. Owned and `'static` — they
    /// outlive the arena.
    pub diagnostics: StdVec<Diagnostic>,
    /// One record per lowering decision, in decision order, dropped
    /// content included (`vize_l2::provenance`), when `P` keeps records.
    pub provenance: P,
    /// Hygiene scope facts per binding-introducing op
    /// (`vize_l2::scope`).
    pub scopes: SideTable<ScopeFacts>,
    /// The recorded parts of every merged text/interpolation run, keyed
    /// by its compound `ui.interpolation` op (P2-9 installment 4,
    /// [`lower::text`](text)); validated and published by lowering.
    pub texts: SideTable<TextParts>,
    /// Retained expressions consumed into compound text ops. They no longer
    /// have a direct op slot, but lint/editor consumers still need their
    /// authentic AST and authored coordinates without reparsing L1 text.
    /// Compile-only lowerings leave this empty and avoid parsing expressions
    /// that the emitter will parse from the compound text facts.
    pub consumed_expressions: StdVec<vize_l2::expr::ExprRef<'a>>,
    /// Consumed `ui.for` scope views, keyed by the `ui.for` op's
    /// page-order id; validated and published by lowering.
    pub for_facts: SideTable<ForParts>,
    /// Per-`ui.if` branch-key facts, keyed by the `ui.if` op's
    /// page-order id; validated and published by lowering.
    pub if_facts: SideTable<IfFacts>,
    /// Captured `<template v-if>` wrapper keys, keyed by the `ui.if`
    /// op's page-order id (P2-9 series 5, [`WrapperKeys`]).
    /// `from_template` is the P2-11 emit signal for template-fragment
    /// vs branch-root blocks.
    pub wrappers: SideTable<WrapperKeys>,
    /// Captured `<template v-for>` unwrap facts, keyed by the `ui.for`
    /// op's page-order id. Presence means the carrier was a template.
    pub for_wrappers: SideTable<ForWrapper>,
    /// Lowering-observed feature bits used by the L2 pass planner.
    pub features: LoweringFeatures,
    /// Vue dialect sugar the lowering (and the legalizing pass) consult.
    /// [`LegacyCaps::VUE3`] unless the caller used [`lower_with_caps`].
    pub caps: LegacyCaps,
}

/// Lower a parsed L1 tree (and the tokenizer errors its parse reported)
/// into L2.
///
/// Total over arbitrary input: malformed source arrives as typed L1
/// holes and lowers structurally; whatever the op family cannot carry
/// becomes a diagnostic plus a kept fragment. A source too large for
/// `u32` offsets lowers to an empty root with one diagnostic.
#[must_use]
pub fn lower<'a>(
    allocator: &'a Allocator,
    tree: &SurfaceTree<'a>,
    errors: &[SurfaceError],
) -> Lowered<'a> {
    lower_with_caps(allocator, tree, errors, LegacyCaps::VUE3)
}

/// Lower with an explicit Vue dialect. Vue 3 is [`LegacyCaps::VUE3`]
/// (identical to [`lower`]); Vue 2 admits `.sync` / `slot-scope` /
/// pipe-filter payloads so the legalizing pass can rewrite them.
#[must_use]
pub fn lower_with_caps<'a>(
    allocator: &'a Allocator,
    tree: &SurfaceTree<'a>,
    errors: &[SurfaceError],
    caps: LegacyCaps,
) -> Lowered<'a> {
    let Ok(root) = SourceRoot::new(tree.source) else {
        return oversized_source(allocator, tree.source, caps);
    };
    lower_source_block_with_caps(allocator, tree, errors, root.whole_block(), caps)
}

/// [`lower`] with authored comments kept as `ui.comment` ops.
///
/// The consumer-facing lowering for surfaces that must see comments: the
/// lint facade (suppression comments, comment-aware sibling rules) reads
/// the same child list the Relief parser keeps under its default
/// `comments: true`. Comments then act as real children for whitespace
/// condensing and as hard boundaries for text-run merging (see
/// [`lower::text`](text)).
#[must_use]
pub fn lower_preserving_comments<'a>(
    allocator: &'a Allocator,
    tree: &SurfaceTree<'a>,
    errors: &[SurfaceError],
) -> Lowered<'a> {
    lower_with_caps_and_comment_policy(
        allocator,
        tree,
        errors,
        LegacyCaps::VUE3,
        true,
        true,
        &[],
        None,
    )
}

pub(crate) fn lower_with_caps_and_comment_policy<'a, P: ProvenanceSink + Default>(
    allocator: &'a Allocator,
    tree: &SurfaceTree<'a>,
    errors: &[SurfaceError],
    caps: LegacyCaps,
    preserve_comments: bool,
    retain_consumed_expressions: bool,
    custom_element_patterns: &[String],
    custom_element_predicate: Option<fn(&str) -> bool>,
) -> Lowered<'a, P> {
    let Ok(root) = SourceRoot::new(tree.source) else {
        return oversized_source(allocator, tree.source, caps);
    };
    let custom_elements = LowerCustomElements {
        patterns: custom_element_patterns,
        predicate: custom_element_predicate,
    };
    lower_source_block_with_caps_and_comment_policy(
        allocator,
        tree,
        errors,
        root.whole_block(),
        caps,
        preserve_comments,
        retain_consumed_expressions,
        custom_elements,
    )
}

/// Lower a parsed L1 source block while preserving file-absolute L0 spans.
#[must_use]
pub fn lower_source_block<'a>(
    allocator: &'a Allocator,
    tree: &SurfaceTree<'a>,
    errors: &[SurfaceError],
    block: SourceBlock<'a>,
) -> Lowered<'a> {
    lower_source_block_with_caps(allocator, tree, errors, block, LegacyCaps::VUE3)
}

/// [`lower_source_block`] under an explicit Vue dialect capability set.
#[must_use]
pub fn lower_source_block_with_caps<'a>(
    allocator: &'a Allocator,
    tree: &SurfaceTree<'a>,
    errors: &[SurfaceError],
    block: SourceBlock<'a>,
    caps: LegacyCaps,
) -> Lowered<'a> {
    lower_source_block_with_caps_and_comment_policy(
        allocator,
        tree,
        errors,
        block,
        caps,
        false,
        false,
        LowerCustomElements::default(),
    )
}

#[derive(Clone, Copy, Default)]
struct LowerCustomElements<'a> {
    patterns: &'a [String],
    predicate: Option<fn(&str) -> bool>,
}

fn lower_source_block_with_caps_and_comment_policy<'a, P: ProvenanceSink + Default>(
    allocator: &'a Allocator,
    tree: &SurfaceTree<'a>,
    errors: &[SurfaceError],
    block: SourceBlock<'a>,
    caps: LegacyCaps,
    preserve_comments: bool,
    retain_consumed_expressions: bool,
    custom_elements: LowerCustomElements<'_>,
) -> Lowered<'a, P> {
    debug_assert!(
        tree.source.as_ptr() == block.source().as_ptr()
            && tree.source.len() == block.source().len(),
        "the source block must be the exact string parsed into the L1 tree"
    );
    let cx = cx::Cx::with_source_block_and_comment_policy(
        allocator,
        block,
        caps,
        preserve_comments,
        retain_consumed_expressions,
        custom_elements.patterns,
        custom_elements.predicate,
    );
    finish::lower_in_context(cx, tree, errors, block)
}

/// The artifact for a source that cannot be addressed by `u32` offsets.
///
/// [`vize_l1::markup::parse`] refuses such sources, so a tree reaching here was
/// built by hand. Nothing can be spanned inside it: lowering yields an
/// empty root and one diagnostic instead of mis-measured ops.
fn oversized_source<'a, P: Default>(
    allocator: &'a Allocator,
    source: &'a str,
    caps: LegacyCaps,
) -> Lowered<'a, P> {
    Lowered {
        allocator,
        source,
        root: Region {
            ops: vize_l0::Vec::new_in(&allocator),
        },
        op_count: 0,
        diagnostics: alloc::vec![crate::exemptions::lowering(
            Span::new(0, 0),
            "the template source is too large to address with u32 offsets; nothing was lowered",
        )],
        provenance: P::default(),
        scopes: SideTable::new(),
        texts: SideTable::new(),
        consumed_expressions: StdVec::new(),
        for_facts: SideTable::new(),
        if_facts: SideTable::new(),
        wrappers: SideTable::new(),
        for_wrappers: SideTable::new(),
        features: LoweringFeatures::EMPTY,
        caps,
    }
}

fn surface_error_span(block: SourceBlock<'_>, offset: u32) -> Span {
    let absolute = block.start().saturating_add(offset);
    let hole = block.zero_width_at(absolute);
    block.span_of(hole).unwrap_or_else(|| {
        debug_assert!(
            false,
            "SourceBlock::zero_width_at returned a slice outside its block"
        );
        Span::new(block.start(), block.start())
    })
}

impl<P: fmt::Debug> fmt::Debug for Lowered<'_, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Lowered")
            .field("root", &self.root)
            .field("op_count", &self.op_count)
            .field("diagnostics", &self.diagnostics)
            .field("provenance", &self.provenance)
            .field("scopes", &self.scopes)
            .field("texts", &self.texts)
            .field("consumed_expressions", &self.consumed_expressions)
            .field("for_facts", &self.for_facts)
            .field("if_facts", &self.if_facts)
            .field("wrappers", &self.wrappers)
            .field("for_wrappers", &self.for_wrappers)
            .field("features", &self.features)
            .field("caps", &self.caps)
            .finish_non_exhaustive()
    }
}

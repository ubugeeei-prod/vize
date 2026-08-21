//! The lowering entry: S1 surface tree in, S2 artifact out — **total,
//! no rollback** (the MLIR import).
//!
//! [`lower`] never panics and never abandons partial work: every S1
//! construct either becomes an op of the existing S2 family or leaves a
//! [`Diagnostic`] beside the kept fragments, and the three fact channels
//! (diagnostics, provenance, hygiene scopes) are all populated by the
//! time the function returns — including on inputs that are wrong from
//! the first byte. The tokenizer's [`SurfaceError`]s enter the unified
//! channel here (`Stage::Surface`, the exact `ErrorCode` message),
//! ahead of the lowering's own `Stage::Semantic` findings.
//!
//! # Id accounting (the numbering law)
//!
//! Ops are numbered densely in page order as they are decided
//! ([`lower::cx`]); [`Lowered::op_count`] equals
//! `DisegnoFolio::of(&lowered.root.ops).op_count()` on every input —
//! the law the side tables and the S2 verifier's `verify_table` key on.
//!
//! [`Diagnostic`]: vize_davinci::diagnostic::Diagnostic
//! [`SurfaceError`]: vize_sinopia::SurfaceError

use alloc::vec::Vec as StdVec;

use vize_carton::{Allocator, Span, String};
use vize_davinci::diagnostic::{Diagnostic, Severity, Stage};
use vize_davinci::side_table::SideTable;
use vize_sinopia::{SurfaceError, SurfaceTree};

use vize_disegno::op::{Namespace, Region};
use vize_disegno::provenance::ProvenanceRecord;
use vize_disegno::scope::ScopeFacts;

mod binding;
mod bindop;
mod cx;
mod directive;
mod element;
mod expr;
mod forop;
mod leaf;
#[cfg(feature = "_legacy")]
pub(crate) mod legacy;
mod slot;
mod structural;
mod text;
mod vfor;

// The one-scanner rule (#4365): the S2 passes re-derive binding names
// with exactly the enumeration the lowering used, never a second one.
pub(crate) use expr::simple_identifier;
// The wrapper-key channel (P2-9 series 5): captured `<template v-if>`
// keys, folded into branch-key facts by the v-if pass.
pub use structural::{WrapperKey, WrapperKeys};
// The one-rebuild rule (the same discipline): the text pass re-derives a
// compound's source with exactly the spelling the lowering minted.
#[cfg(feature = "_legacy")]
pub use legacy::mode_probe as legacy_mode_probe;
#[cfg(feature = "_legacy")]
pub use legacy::{FilterParts, FilterSegment, LegacyVueLine, filter_split};
pub use text::{TextPart, TextParts, rebuild_source};

/// The S2 artifact one lowering produces: the op tree plus the three
/// fact channels, all live even when diagnostics are present (the
/// Lean-InfoTree survival property — kept fragments, not rollback).
#[derive(Debug)]
pub struct Lowered<'a> {
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
    /// content included (`vize_disegno::provenance`).
    pub provenance: StdVec<ProvenanceRecord>,
    /// Hygiene scope facts per binding-introducing op
    /// (`vize_disegno::scope`).
    pub scopes: SideTable<ScopeFacts>,
    /// The recorded parts of every merged text/interpolation run, keyed
    /// by its compound `ui.interpolation` op (P2-9 installment 4,
    /// [`lower::text`](text)); validated and consumed by `pass::text`.
    pub texts: SideTable<TextParts>,
    /// Captured `<template v-if>` wrapper keys, keyed by the `ui.if`
    /// op's page-order id (P2-9 series 5, [`WrapperKeys`]); folded into
    /// branch-key facts by `pass::vif`.
    pub wrappers: SideTable<WrapperKeys>,
    /// The recorded split of every legacy filter site, keyed by its
    /// `vue.filter` or `ui.bind` op's page-order id (P2-9 series 7,
    /// [`FilterParts`]); validated and consumed by `pass::legacy`.
    /// Always empty under the default entry point.
    #[cfg(feature = "_legacy")]
    pub filters: SideTable<FilterParts>,
}

/// Lower a parsed S1 tree (and the tokenizer errors its parse reported)
/// into S2.
///
/// Total over arbitrary input: malformed source arrives as typed S1
/// holes and lowers structurally; whatever the op family cannot carry
/// becomes a diagnostic plus a kept fragment. The only inherited
/// precondition is [`vize_sinopia::parse`]'s own u32-addressability
/// assertion, which every tree passed here has already satisfied.
#[must_use]
pub fn lower<'a>(
    allocator: &'a Allocator,
    tree: &SurfaceTree<'a>,
    errors: &[SurfaceError],
) -> Lowered<'a> {
    lower_impl(cx::Cx::new(allocator, tree.source), tree, errors)
}

/// Lower a parsed S1 tree under a **legacy Vue dialect** (P2-9 series
/// 7): the same total conversion as [`lower`], with the dialect's
/// template sugar mirrored from the shipped legacy lane — see
/// [`legacy`]'s module docs for the measured split. The default entry
/// point never resolves a legacy mode, so this function (and everything
/// it arms) compiles out entirely without the `_legacy` feature.
#[cfg(feature = "_legacy")]
#[must_use]
pub fn lower_legacy<'a>(
    allocator: &'a Allocator,
    tree: &SurfaceTree<'a>,
    errors: &[SurfaceError],
    line: legacy::LegacyVueLine,
) -> Lowered<'a> {
    let mut cx = cx::Cx::new(allocator, tree.source);
    cx.legacy = legacy::LegacyMode::for_line(line);
    lower_impl(cx, tree, errors)
}

fn lower_impl<'a>(
    mut cx: cx::Cx<'a>,
    tree: &SurfaceTree<'a>,
    errors: &[SurfaceError],
) -> Lowered<'a> {
    for error in errors {
        cx.diagnostics.push(Diagnostic::new(
            Severity::Error,
            Stage::Surface,
            Span::new(error.offset, error.offset),
            String::from(error.code.message()),
        ));
    }
    let ops = structural::lower_children(&mut cx, &tree.children, Namespace::Html);
    Lowered {
        root: Region { ops },
        op_count: cx.op_count(),
        diagnostics: cx.diagnostics,
        provenance: cx.provenance,
        scopes: cx.scopes,
        texts: cx.texts,
        wrappers: cx.wrappers,
        #[cfg(feature = "_legacy")]
        filters: cx.filters,
    }
}

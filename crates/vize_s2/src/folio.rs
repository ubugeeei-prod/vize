//! The S2 folio: the stage dump.
//!
//! [`S2Folio`] is an **owned document model** of an S2 op tree,
//! printable and parseable under the `vize_davinci::folio` contract; the
//! grammar is documented in `davinci-road/plan/folio-format.md` ("Disegno
//! page"). [`S2Folio::of`] mirrors a live arena tree into the owned
//! model, because arena references cannot persist across a compile
//! (P1-11's contract) and `parse` must construct values without an arena.
//!
//! # Why this page is hand-written (the P2-4 boundary, applied)
//!
//! `#[derive(Folio)]` generates the mechanical trio for **flat** documents:
//! header scalars plus one-level list/map sections, computable from the
//! type shape alone. The S2 artifact is region-nested by its central
//! design decision - ops own their regions - and flattening the tree into
//! derivable lines would move structure validation (indentation, which op
//! may own what) outside `parse`, stripping its 1-based line numbers. That
//! is a semantic grammar, so it is hand-written and reviewed, exactly the
//! `CroquisFolio` precedent; the derive stays the right tool for any flat
//! S2 side artifact a later task adds.
//!
//! A folio models the dump, not the analysis: the `ops=` header count is
//! the printer's computed statement about the tree (parse validates its
//! syntax and discards the value - normalization by the first print), and
//! no semantic invariant is enforced beyond tree shape; branch and region
//! well-formedness beyond the grammar belongs to the S2 verifier
//! ([`crate::verify`]).

use alloc::vec::Vec;

use vize_davinci::folio::{Folio, FolioError, FolioMode};

mod owned;
mod parse;
mod print;

pub use owned::{
    FolioAttribute, FolioBind, FolioBinding, FolioBranch, FolioComment, FolioComponent,
    FolioContract, FolioElement, FolioExpr, FolioFor, FolioForBinding, FolioIf, FolioInterpolation,
    FolioModel, FolioName, FolioOn, FolioOp, FolioSlot, FolioSlotContent, FolioText, FolioVueCloak,
    FolioVueCssBind, FolioVueDirective, FolioVueHtml, FolioVueMemo, FolioVueOnce, FolioVueShow,
    FolioVueSlotScope, FolioVueSync, FolioVueText,
};

/// Document model of an S2 op-tree dump.
///
/// The root region's ops, in document order. Everything else - the `ops=`
/// count, section headers, indentation - is the printer's derived
/// statement about this tree.
#[doc(alias = "DisegnoFolio")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct S2Folio {
    /// The root region's ops.
    pub ops: Vec<FolioOp>,
}

/// Compatibility alias for the original S2 codename type.
pub type DisegnoFolio = S2Folio;

impl Folio for S2Folio {
    fn print<W: core::fmt::Write>(&self, w: &mut W, mode: FolioMode) -> core::fmt::Result {
        print::print(self, w, mode)
    }

    fn parse(input: &str) -> Result<Self, FolioError> {
        parse::parse(input)
    }
}

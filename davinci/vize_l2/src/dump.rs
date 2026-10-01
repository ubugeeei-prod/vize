//! The L2 folio: the stage dump.
//!
//! [`Page`] is an **owned document model** of an L2 op tree,
//! printable and parseable under the `vize_l0::dump` contract; the
//! grammar is documented in `docs/davinci/plan/folio-format.md` ("Disegno
//! page"). [`Page::of`] mirrors a live arena tree into the owned
//! model, because arena references cannot persist across a compile
//! (P1-11's contract) and `parse` must construct values without an arena.
//!
//! # Why this page is hand-written (the P2-4 boundary, applied)
//!
//! `#[derive(Dump)]` generates the mechanical trio for **flat** documents:
//! header scalars plus one-level list/map sections, computable from the
//! type shape alone. The L2 artifact is region-nested by its central
//! design decision - ops own their regions - and flattening the tree into
//! derivable lines would move structure validation (indentation, which op
//! may own what) outside `parse`, stripping its 1-based line numbers. That
//! is a semantic grammar, so it is hand-written and reviewed, exactly the
//! `CroquisPage` precedent; the derive stays the right tool for any flat
//! L2 side artifact a later task adds.
//!
//! A folio models the dump, not the analysis: the `ops=` header count is
//! the printer's computed statement about the tree (parse validates its
//! syntax and discards the value - normalization by the first print), and
//! no semantic invariant is enforced beyond tree shape; branch and region
//! well-formedness beyond the grammar belongs to the L2 verifier
//! ([`crate::verify`]).

use alloc::vec::Vec;

use vize_l0::dump::{Dump, Error as DumpError, Mode as DumpMode};
use vize_l0::key::{KeySink, KeyedArtifact, schema};
use vize_l0::stage::Stage;

mod codec;
pub mod historical;
mod owned;
mod parse;
mod print;
pub mod provenance;

pub use crate::dump::owned::{
    Attribute, Bind, Binding, Branch, Comment, Component, Contract, Element, Expr, For, ForBinding,
    If, Interpolation, Model, Name, On, Op, Slot, SlotContent, Text, VueCloak, VueCssBind,
    VueDirective, VueHtml, VueMemo, VueOnce, VueShow, VueSlotScope, VueSync, VueText,
};
pub use crate::dump::provenance::{Page as ProvenancePage, Record as DumpProvenance};

/// Render any existing stage page in its canonical full form for opt-in capture.
#[must_use]
pub fn render_full_page<P: Dump>(page: &P) -> vize_l0::String {
    page.print_to_string(DumpMode::Full)
}

/// Document model of an L2 op-tree dump.
///
/// The root region's ops, in document order. Everything else - the `ops=`
/// count, section headers, indentation - is the printer's derived
/// statement about this tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Page {
    /// The root region's ops.
    pub ops: Vec<Op>,
}

/// A native payload has no contract in the existing owned dump grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeDumpError {
    /// A retained formal parameter cannot be serialized as an expression.
    JsBindingUnsupported { span: vize_l0::Span },
}

impl core::fmt::Display for NativeDumpError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::JsBindingUnsupported { span } => write!(
                formatter,
                "native binding dump unsupported at bytes {}..{}",
                span.start, span.end
            ),
        }
    }
}

impl core::error::Error for NativeDumpError {}

impl Dump for Page {
    fn print<W: core::fmt::Write>(&self, w: &mut W, mode: DumpMode) -> core::fmt::Result {
        print::print(self, w, print::Style::current(mode))
    }

    fn parse(input: &str) -> Result<Self, DumpError> {
        parse::parse(input)
    }
}

/// The P5-1a L2 page key: the `Full` form with every span rebased to the
/// block start, so a page keys identically wherever its block sits.
impl KeyedArtifact for Page {
    const STAGE: Stage = Stage::Semantic;
    const SCHEMA_VERSION: u32 = schema::L2_PAGE;

    fn feed_key(&self, sink: &mut KeySink) {
        let style = print::Style::keyed(sink.block_start());
        // A key sink never fails a write, so there is no error to surface.
        let _ = print::print(self, sink, style);
    }
}

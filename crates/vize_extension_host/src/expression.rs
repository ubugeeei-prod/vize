//! The expression-dialect world's host half (P6-1b).
//!
//! A guest receives one block's expressions and their environment and
//! answers with an `expression-facts` α page (P4-2), a projection page (the
//! P4-5a rows) and diagnostics. The host accepts the answer as untrusted
//! input, exactly like the input world's ([`crate::accept`]):
//!
//! 1. both pages at a schema version this host reads, parsed and canonical;
//! 2. one fact per batch expression — the α page's three sections name
//!    exactly the batch's ids — and every referenced binding in the
//!    environment when the enumeration claims to be exact;
//! 3. every projection row inside the generated text (on character
//!    boundaries) and inside one expression's authored span, every sub-span
//!    inside its row and, on the generated side, on a character boundary;
//! 4. every diagnostic span inside an expression.

use core::fmt;

use serde::{Deserialize, Serialize};
use vize_davinci::fact::{AlphaDocument, ExpressionFacts};
use vize_s0::String;

use crate::accept::{AcceptError, read_page};
use crate::contract::{Diagnostic, GuestError, Page, Span};

mod accept;
pub mod projection;
mod session;

pub use accept::accept_analysis;
pub use projection::{ProjectionPage, ProjectionRow, Range};
pub use session::{ExpressionError, ExpressionSession};

/// The facts page schema version this host reads (the group's α schema).
pub const FACTS_PAGE_SCHEMA: u32 = 1;
/// The projection page schema version this host reads.
pub const PROJECTION_PAGE_SCHEMA: u32 = 1;
/// Features the expression-dialect world requires, sorted.
pub const REQUIRED_FEATURES: &[&str] = &["facts-page@1", "projection-page@1"];

/// `expression-analysis.binding`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub name: String,
    pub kind: String,
}

/// `expression-analysis.expression`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expression {
    pub id: u32,
    pub source: String,
    pub span: Span,
}

/// `expression-analysis.expression-batch`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpressionBatch {
    pub environment: Vec<Binding>,
    pub expressions: Vec<Expression>,
}

/// `expression-analysis.analysis`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Analysis {
    pub facts: Page,
    pub projection: Page,
    pub diagnostics: Vec<Diagnostic>,
}

/// One expression-dialect guest, whatever hosts it.
pub trait ExpressionDialectGuest {
    /// `handshake.get-capability`.
    fn get_capability(&mut self) -> Result<crate::Capability, GuestError>;
    /// `expression-analysis.analyze`.
    fn analyze(&mut self, batch: &ExpressionBatch) -> Result<Analysis, GuestError>;
}

impl<G: ExpressionDialectGuest + ?Sized> ExpressionDialectGuest for Box<G> {
    fn get_capability(&mut self) -> Result<crate::Capability, GuestError> {
        (**self).get_capability()
    }

    fn analyze(&mut self, batch: &ExpressionBatch) -> Result<Analysis, GuestError> {
        (**self).analyze(batch)
    }
}

/// An analysis the host accepted, with its pages parsed.
#[derive(Debug, PartialEq)]
pub struct AcceptedAnalysis {
    /// The answer exactly as the guest serialized it.
    pub analysis: Analysis,
    /// The parsed facts document.
    pub facts: AlphaDocument<ExpressionFacts>,
    /// The parsed projection.
    pub projection: ProjectionPage,
}

/// Why the host refused an analysis beyond the shared page checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnalysisError {
    /// A page refusal shared with the input world.
    Page(AcceptError),
    /// The facts page's entries are not exactly the batch's expression ids.
    FactIds { expected: Vec<u32>, found: Vec<u32> },
    /// An exact enumeration names a binding outside the environment.
    UnknownBinding { id: u32, name: String },
    /// Projection row `index` (or one of its sub-spans) is out of bounds.
    Row { index: usize, reason: &'static str },
    /// Diagnostic `index` points outside every expression.
    DiagnosticSpan { index: usize, span: Span },
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Page(error) => error.fmt(f),
            Self::FactIds { expected, found } => write!(
                f,
                "facts-page names expressions {found:?}, the batch has {expected:?}"
            ),
            Self::UnknownBinding { id, name } => write!(
                f,
                "expression {id} exactly references {name:?}, which is not in the environment"
            ),
            Self::Row { index, reason } => write!(f, "projection row {index} {reason}"),
            Self::DiagnosticSpan { index, span } => write!(
                f,
                "diagnostic {index} span {}:{} lies outside every expression",
                span.start, span.end
            ),
        }
    }
}

pub(crate) fn read_facts(page: &Page) -> Result<AlphaDocument<ExpressionFacts>, AnalysisError> {
    read_page("facts-page", FACTS_PAGE_SCHEMA, page).map_err(AnalysisError::Page)
}

pub(crate) fn read_projection(page: &Page) -> Result<ProjectionPage, AnalysisError> {
    read_page("projection-page", PROJECTION_PAGE_SCHEMA, page).map_err(AnalysisError::Page)
}

//! The output-target world's host half (P6-1c).
//!
//! A guest receives one template's S2 and S3 pages and answers with an
//! `emit-document-page@1` folio (P3-9's span-carrying document) and
//! diagnostics. The host accepts the answer as untrusted input:
//!
//! 1. both request pages at the schema this host reads;
//! 2. the document page at schema 1, parsed and canonical;
//! 3. every generated range inside the document text, on a character
//!    boundary;
//! 4. every diagnostic span inside an authored link.

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::contract::{Diagnostic, GuestError, Page, Span};

mod accept;
mod document;
mod session;

pub use accept::accept_emitted;
pub use document::{EmitDocument, EmitLink};
pub use session::{OutputError, OutputSession};

/// The S3 page schema version this host reads.
pub const S3_PAGE_SCHEMA: u32 = 1;
/// The emit-document page schema version this host reads.
pub const EMIT_DOCUMENT_PAGE_SCHEMA: u32 = 1;
/// Features the output-target world requires, sorted.
pub const REQUIRED_FEATURES: &[&str] = &["emit-document-page@1", "s2-page@1", "s3-page@1"];

/// `emission.emit-request`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmitRequest {
    pub s2: Page,
    pub s3: Page,
}

/// `emission.emitted`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Emitted {
    pub document: Page,
    pub diagnostics: Vec<Diagnostic>,
}

/// One output-target guest, whatever hosts it.
pub trait OutputTargetGuest {
    /// `handshake.get-capability`.
    fn get_capability(&mut self) -> Result<crate::Capability, GuestError>;
    /// `emission.emit`.
    fn emit(&mut self, request: &EmitRequest) -> Result<Emitted, GuestError>;
}

impl<G: OutputTargetGuest + ?Sized> OutputTargetGuest for Box<G> {
    fn get_capability(&mut self) -> Result<crate::Capability, GuestError> {
        (**self).get_capability()
    }

    fn emit(&mut self, request: &EmitRequest) -> Result<Emitted, GuestError> {
        (**self).emit(request)
    }
}

/// An emission the host accepted, with its document parsed.
#[derive(Debug, PartialEq)]
pub struct AcceptedEmit {
    /// The answer exactly as the guest serialized it.
    pub emitted: Emitted,
    /// The parsed document.
    pub document: EmitDocument,
}

/// Why the host refused an emission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmitError {
    /// A page refusal shared with the other worlds.
    Page(crate::accept::AcceptError),
    /// Request page `page` is not the schema this host reads.
    RequestSchema {
        page: &'static str,
        found: u32,
        reads: u32,
    },
    /// Generated range `index` does not index the document text.
    GeneratedRange { index: usize, span: Span },
    /// Diagnostic `index` points outside every authored link.
    DiagnosticSpan { index: usize, span: Span },
}

impl fmt::Display for EmitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Page(error) => error.fmt(f),
            Self::RequestSchema { page, found, reads } => write!(
                f,
                "{page} schema version {found} is unreadable: this host reads version {reads}"
            ),
            Self::GeneratedRange { index, span } => write!(
                f,
                "generated range {index} {}:{} is outside the document or splits a character",
                span.start, span.end
            ),
            Self::DiagnosticSpan { index, span } => write!(
                f,
                "diagnostic {index} span {}:{} lies outside every authored link",
                span.start, span.end
            ),
        }
    }
}

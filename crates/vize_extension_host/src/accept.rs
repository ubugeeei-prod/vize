//! Host-side acceptance: a guest's answer is untrusted input.
//!
//! [`accept`] checks, in order, and refuses with the first failure:
//!
//! 1. each page's schema version is one this host reads (P2-17's
//!    versioned-refusal pattern) — before the text is parsed at all;
//! 2. each page parses and is canonical (`print(parse(text)) == text`), so
//!    one tree has one spelling on the wire;
//! 3. the S1 page's tokens tile the block source exactly;
//! 4. every diagnostic span, and every part's span, lies inside the block.

use core::fmt;

use vize_davinci::diagnostic as davinci;
use vize_davinci::folio::{Folio, FolioError, FolioMode};
use vize_s0::String;
use vize_s2::folio::S2Folio;

use crate::contract::{LoweredBlock, Page, S1_PAGE_SCHEMA, S2_PAGE_SCHEMA, SourceBlock, Span};
use crate::surface_page::{SurfacePage, TileError};

/// A guest answer the host accepted, with the pages parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accepted {
    /// The answer exactly as the guest serialized it.
    pub lowered: LoweredBlock,
    /// The parsed S1 page; it tiles the block source.
    pub surface: SurfacePage,
    /// The parsed S2 page.
    pub semantic: S2Folio,
    /// The diagnostics in the in-tree channel's type.
    pub diagnostics: Vec<davinci::Diagnostic>,
}

/// Why the host refused a guest's answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptError {
    /// A page's schema version is not one this host reads.
    UnreadableSchema {
        page: &'static str,
        found: u32,
        reads: u32,
    },
    /// A page does not parse.
    Parse {
        page: &'static str,
        error: FolioError,
    },
    /// A page parses but is not in canonical form; `at` is the first byte
    /// where its text and the canonical reprint differ.
    NotCanonical { page: &'static str, at: usize },
    /// The S1 page does not tile the block source.
    Tiles(TileError),
    /// Diagnostic `index` (or one of its parts) points outside the block.
    DiagnosticSpan {
        index: usize,
        span: Span,
        block: Span,
    },
}

impl fmt::Display for AcceptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnreadableSchema { page, found, reads } => write!(
                f,
                "{page} schema version {found} is unreadable: this host reads version {reads}"
            ),
            Self::Parse { page, error } => write!(f, "{page}: {error}"),
            Self::NotCanonical { page, at } => {
                write!(
                    f,
                    "{page} is not canonical: it differs from its reprint at byte {at}"
                )
            }
            Self::Tiles(error) => write!(f, "s1-page does not tile the block: {error}"),
            Self::DiagnosticSpan { index, span, block } => write!(
                f,
                "diagnostic {index} span {}:{} lies outside the block {}:{}",
                span.start, span.end, block.start, block.end
            ),
        }
    }
}

/// Accept a guest's answer to `block`.
///
/// # Errors
///
/// The first check the answer fails, as an [`AcceptError`].
pub fn accept(block: &SourceBlock, lowered: LoweredBlock) -> Result<Accepted, AcceptError> {
    let surface: SurfacePage = read_page("s1-page", S1_PAGE_SCHEMA, &lowered.surface)?;
    let semantic: S2Folio = read_page("s2-page", S2_PAGE_SCHEMA, &lowered.semantic)?;
    surface
        .check_tiles(&block.source)
        .map_err(AcceptError::Tiles)?;
    let bounds = Span {
        start: block.base,
        end: block.base + block.source.len() as u32,
    };
    for (index, diagnostic) in lowered.diagnostics.iter().enumerate() {
        let spans =
            core::iter::once(diagnostic.span).chain(diagnostic.parts.iter().map(|p| p.span));
        for span in spans {
            if span.start > span.end || span.start < bounds.start || span.end > bounds.end {
                return Err(AcceptError::DiagnosticSpan {
                    index,
                    span,
                    block: bounds,
                });
            }
        }
    }
    let diagnostics = lowered
        .diagnostics
        .iter()
        .map(davinci::Diagnostic::from)
        .collect();
    Ok(Accepted {
        lowered,
        surface,
        semantic,
        diagnostics,
    })
}

pub(crate) fn read_page<T: Folio>(
    name: &'static str,
    reads: u32,
    page: &Page,
) -> Result<T, AcceptError> {
    if page.schema_version != reads {
        return Err(AcceptError::UnreadableSchema {
            page: name,
            found: page.schema_version,
            reads,
        });
    }
    let parsed = T::parse(&page.text).map_err(|error| AcceptError::Parse { page: name, error })?;
    let reprint = full_text(&parsed);
    if reprint != page.text {
        let at = reprint
            .bytes()
            .zip(page.text.bytes())
            .position(|(a, b)| a != b)
            .unwrap_or(reprint.len().min(page.text.len()));
        return Err(AcceptError::NotCanonical { page: name, at });
    }
    Ok(parsed)
}

/// A page's canonical `Full` text.
#[must_use]
pub fn full_text<T: Folio>(page: &T) -> String {
    let mut out = String::default();
    page.print(&mut out, FolioMode::Full)
        .expect("printing into a String cannot fail");
    out
}

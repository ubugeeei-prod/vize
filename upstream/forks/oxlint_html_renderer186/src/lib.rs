// This immutable third-party kernel retains its upstream implementation policy.
#![allow(clippy::all, clippy::wildcard_imports)]

//! Private renderer assets from OXC 2ae2939; see LICENSE and UPSTREAM.json.
//! No parser, linter, reporter service or second OXC span family is included.

mod handlers;
mod protocol;
pub mod source_impls;

pub use handlers::{GraphicalReportHandler, GraphicalTheme, JSONReportHandler};
use oxc_span::Span;
pub use protocol::{Diagnostic, Severity, SourceCode};

/// Adapter for the existing OXC Span; copied renderer methods borrow this data.
#[derive(Clone, Debug)]
pub struct LabeledSpan {
    label: Option<String>,
    span: Span,
}

impl LabeledSpan {
    pub fn new(label: Option<String>, offset: usize, length: usize) -> Self {
        Self {
            label,
            span: Span::new(offset as u32, offset.saturating_add(length) as u32),
        }
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn offset(&self) -> usize {
        self.span.start as usize
    }
    pub fn len(&self) -> usize {
        self.span.size() as usize
    }
    pub fn is_empty(&self) -> bool {
        self.span.is_empty()
    }
    pub fn primary(&self) -> bool {
        false
    }
}

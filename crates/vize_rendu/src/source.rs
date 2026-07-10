//! Owned source documents and frontend-independent provenance.

use crate::RenduSourceId;
use vize_carton::source_anchor::SourceAnchor;

/// A byte position with optional human-readable line and column information.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct RenduPosition {
    pub offset: u32,
    pub line: u32,
    pub column: u32,
}

impl RenduPosition {
    pub const fn new(offset: u32, line: u32, column: u32) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }

    /// A position when the producer only has byte offsets.
    pub const fn offset(offset: u32) -> Self {
        Self::new(offset, 0, 0)
    }
}

/// A source span tied to one document in the root's source arena.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct RenduSpan {
    pub source: RenduSourceId,
    pub start: RenduPosition,
    pub end: RenduPosition,
}

impl RenduSpan {
    pub const fn new(source: RenduSourceId, start: RenduPosition, end: RenduPosition) -> Self {
        Self { source, start, end }
    }

    pub const fn offsets(source: RenduSourceId, start: u32, end: u32) -> Self {
        Self::new(
            source,
            RenduPosition::offset(start),
            RenduPosition::offset(end),
        )
    }

    pub const fn is_empty(self) -> bool {
        self.start.offset == self.end.offset
    }
}

/// Primary and related source regions for one HIR item.
///
/// `related` preserves provenance for synthesized constructs such as a slot
/// assembled from an opening tag and a separately-authored body.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct RenduProvenance {
    pub primary: Option<RenduSpan>,
    pub related: Vec<RenduSpan>,
}

impl RenduProvenance {
    pub const fn generated() -> Self {
        Self {
            primary: None,
            related: Vec::new(),
        }
    }

    pub const fn from_span(span: RenduSpan) -> Self {
        Self {
            primary: Some(span),
            related: Vec::new(),
        }
    }

    pub fn with_related(mut self, span: RenduSpan) -> Self {
        self.related.push(span);
        self
    }

    pub fn spans(&self) -> impl Iterator<Item = RenduSpan> + '_ {
        self.primary
            .iter()
            .copied()
            .chain(self.related.iter().copied())
    }
}

/// One source document retained by an owned Rendu root.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenduSource {
    pub name: Option<Box<str>>,
    pub contents: Box<str>,
    /// Opaque language label supplied by the producer (for example `vue` or
    /// `tsx`). Rendu does not branch on this value.
    pub language: Option<Box<str>>,
    /// Stable compilation source identity, distinct from the root-local ID.
    pub anchor: Option<SourceAnchor>,
}

impl RenduSource {
    pub fn anonymous(contents: impl Into<Box<str>>) -> Self {
        Self {
            name: None,
            contents: contents.into(),
            language: None,
            anchor: None,
        }
    }

    pub fn named(name: impl Into<Box<str>>, contents: impl Into<Box<str>>) -> Self {
        Self {
            name: Some(name.into()),
            contents: contents.into(),
            language: None,
            anchor: None,
        }
    }

    pub fn with_language(mut self, language: impl Into<Box<str>>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Tie this root-local source record to its compilation source revision.
    pub const fn with_anchor(mut self, anchor: SourceAnchor) -> Self {
        self.anchor = Some(anchor);
        self
    }

    pub const fn anchor(&self) -> Option<SourceAnchor> {
        self.anchor
    }
}

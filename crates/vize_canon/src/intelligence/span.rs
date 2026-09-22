//! Authored ranges and projection-row classification for type intelligence.

use super::CursorContext;
use crate::virtual_ts::ProjectionSpanKind;

/// Compact authored byte range (`u32` offsets) reported by type intelligence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    #[inline]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    #[inline]
    pub const fn len(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    #[inline]
    pub const fn contains(&self, offset: u32) -> bool {
        offset >= self.start && offset < self.end
    }
}

impl From<ProjectionSpanKind> for CursorContext {
    fn from(kind: ProjectionSpanKind) -> Self {
        match kind {
            ProjectionSpanKind::Script => CursorContext::Script,
            // An unclassified template expression completes like an
            // interpolation: bindings plus template globals.
            ProjectionSpanKind::Interpolation | ProjectionSpanKind::TemplateExpression => {
                CursorContext::Interpolation
            }
            ProjectionSpanKind::DirectiveExpr => CursorContext::DirectiveExpr,
            ProjectionSpanKind::DirectiveArg => CursorContext::DirectiveArg,
            ProjectionSpanKind::EventHandler => CursorContext::EventHandler,
            ProjectionSpanKind::VForVar => CursorContext::VForVar,
            ProjectionSpanKind::SlotBinding
            | ProjectionSpanKind::ComponentRef
            | ProjectionSpanKind::Unknown => CursorContext::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CursorContext, Span};
    use crate::virtual_ts::{ProjectionMapping, ProjectionMeta, ProjectionSpanKind, VizeMapping};

    #[test]
    fn span_is_half_open() {
        let span = Span::new(10, 20);
        assert!(!span.contains(9));
        assert!(span.contains(10));
        assert!(span.contains(19));
        assert!(!span.contains(20));
        assert_eq!(span.len(), 10);
        assert!(!span.is_empty());
        assert!(Span::new(4, 4).is_empty());
    }

    #[test]
    fn cursor_context_reads_the_projection_row_kind() {
        let croquis = vize_croquis::Croquis::default();
        let mut mapping = ProjectionMapping::new();
        mapping.push_with(
            VizeMapping::new(0..5, 10..15),
            ProjectionMeta::of_kind(ProjectionSpanKind::EventHandler),
        );
        mapping.push_with(
            VizeMapping::new(5..9, 20..24),
            ProjectionMeta::of_kind(ProjectionSpanKind::TemplateExpression),
        );
        mapping.push(VizeMapping::new(9..12, 30..33));
        let intelligence =
            super::super::TypeIntelligence::new("", &croquis).with_source_map(&mapping);

        assert_eq!(intelligence.cursor_context(12), CursorContext::EventHandler);
        assert_eq!(
            intelligence.cursor_context(20),
            CursorContext::Interpolation
        );
        assert_eq!(intelligence.cursor_context(31), CursorContext::Unknown);
        assert_eq!(intelligence.cursor_context(15), CursorContext::Unknown);
    }
}

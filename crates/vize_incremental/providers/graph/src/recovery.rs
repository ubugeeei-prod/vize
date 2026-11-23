//! Read typed graph rows or the original recovery records from the same revision.
use super::{
    BlockView, Diagnostic, Lang, NativeHole, Span, TemplateGraphError, TemplateQueries,
    TemplateReachBatch, source_sites::project,
};

/// The native template lowering either supplies a complete graph or describes
/// the source constructs that prevented its construction.
pub enum TemplateGraphObservation<'db> {
    Complete(TemplateReachBatch<'db>),
    Incomplete(TemplateRecovery<'db>),
}

impl<'db> TemplateGraphObservation<'db> {
    /// Observe one resident lowering. A block outside the template language
    /// or a construction or graph integrity failure retains its original error.
    #[inline]
    pub fn read(block: BlockView<'db>, lang: Lang) -> Result<Self, &'db TemplateGraphError> {
        match block.reach_batch(lang) {
            Ok(graph) => Ok(Self::Complete(graph)),
            Err(TemplateGraphError::Incomplete { holes, diagnostics }) => {
                Ok(Self::Incomplete(TemplateRecovery {
                    holes,
                    diagnostics,
                    text: block.source().text.as_str(),
                    start: block.start(),
                }))
            }
            Err(error) => Err(error),
        }
    }
}

/// Original block-relative holes and diagnostic records with their current source.
pub struct TemplateRecovery<'db> {
    holes: &'db [NativeHole],
    diagnostics: &'db [Diagnostic],
    text: &'db str,
    start: u32,
}

impl<'db> TemplateRecovery<'db> {
    #[must_use]
    pub fn holes(&self) -> &'db [NativeHole] {
        self.holes
    }

    #[must_use]
    pub fn diagnostics(&self) -> &'db [Diagnostic] {
        self.diagnostics
    }

    /// Project a primary, related or witness span in this block to its
    /// file-absolute range and borrow the exact original bytes it covers.
    #[inline]
    pub fn source_site(&self, relative: Span) -> Result<(Span, &'db str), TemplateGraphError> {
        project(self.text, self.start, relative).ok_or(TemplateGraphError::OutsideBlock(relative))
    }
}

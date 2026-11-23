//! The lossless L1 tree and both L2 analysis results from one selected source.
use super::{TemplateAnalysis, analyze};
use crate::{Lowered, lower_source_block};
use vize_l0::{
    Allocator, SourceBlock, Span,
    pass::{PassFailure, PassObserver},
};
use vize_l1::{
    container::vue::CheckedTemplateBlock,
    markup::{SurfaceError, SurfaceTree, parse_with_options},
};

/// One original template for source-aware compiler and tooling requests.
/// The retained syntax includes trivia and recovery. L2 spans address the
/// original file. Sibling script bindings remain outside this structural query.
pub struct OriginalTemplate<'a> {
    block: SourceBlock<'a>,
    open_tag: Span,
    l1ir: SurfaceTree<'a>,
    syntax_errors: &'a [SurfaceError],
    l2ir: Lowered<'a>,
    analysis: TemplateAnalysis,
}

impl<'a> OriginalTemplate<'a> {
    /// Parse the selected block once and run both analyses over its lowering.
    pub fn analyze(
        arena: &'a Allocator,
        selected: CheckedTemplateBlock<'_, 'a>,
        observer: &mut impl PassObserver,
    ) -> Result<Self, PassFailure> {
        let block = selected.block();
        let (l1ir, syntax_errors) =
            parse_with_options(arena, block.source(), selected.surface_options());
        let syntax_errors = syntax_errors.into_arena_slice();
        let mut l2ir = lower_source_block(arena, &l1ir, syntax_errors, block);
        let analysis = analyze(&mut l2ir, observer)?;
        Ok(Self {
            block,
            open_tag: selected.open_tag(),
            l1ir,
            syntax_errors,
            l2ir,
            analysis,
        })
    }

    #[must_use]
    pub fn block(&self) -> SourceBlock<'a> {
        self.block
    }
    /// Whole-file byte range of the original opening template tag.
    #[must_use]
    pub fn open_tag(&self) -> Span {
        self.open_tag
    }
    #[must_use]
    pub fn l1ir(&self) -> &SurfaceTree<'a> {
        &self.l1ir
    }
    #[must_use]
    pub fn syntax_errors(&self) -> &[SurfaceError] {
        self.syntax_errors
    }
    #[must_use]
    pub fn l2ir(&self) -> &Lowered<'a> {
        &self.l2ir
    }
    #[must_use]
    pub fn analysis(&self) -> &TemplateAnalysis {
        &self.analysis
    }
}

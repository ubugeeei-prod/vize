//! Compiler static analysis and lint complexity from the same original Vue 3 template.
//! The descriptor owns selection and current file positions. Only the selected
//! content invalidates the grouped L2 pass, including when it contains recoverable errors.
use vize_l0::{SourceBlock, SourceFrameError, Span, diag::Diagnostic, pass::PassFailure};
use vize_l1::container::{ContainerError, vue::DescriptorIssue};
use vize_l1_to_l2::pass::{
    cfg::{ComplexityFacts, Contribution},
    template::TemplateAnalysis,
};

/// Full descriptor evidence from an unselectable template.
#[derive(Debug, PartialEq, Eq)]
pub struct TemplateRefusal {
    pub issues: Box<[DescriptorIssue]>,
    pub errors: Box<[ContainerError]>,
}

#[expect(
    clippy::disallowed_types,
    reason = "repeated editor reads share the complete native refusal"
)]
type SharedRefusal = std::sync::Arc<TemplateRefusal>;

/// Original descriptor evidence, source frame error or registered pass failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateReadError {
    Descriptor(SharedRefusal),
    OutsideBlock(Span),
    Source(SourceFrameError),
    Pass(PassFailure),
}

mod cursor;
pub use cursor::TemplateDocument;

struct TemplateProducts {
    analysis: TemplateAnalysis,
    diagnostics: Vec<Diagnostic>,
}

/// A read of the current source frame and its block-relative analysis.
pub struct TemplateRead<'source, 'read> {
    block: SourceBlock<'source>,
    opening: Span,
    products: &'read TemplateProducts,
}

/// The editor-facing complexity and diagnostics, in current file coordinates.
#[derive(Debug, PartialEq, Eq)]
pub struct TemplateReport {
    pub opening: Span,
    pub content: Span,
    pub complexity: ComplexityFacts,
    pub diagnostics: Vec<Diagnostic>,
}

impl<'source, 'read> TemplateRead<'source, 'read> {
    /// Exact authored content in this file revision.
    #[must_use]
    pub fn block(&self) -> SourceBlock<'source> {
        self.block
    }

    /// Complete original opening tag in file coordinates.
    #[must_use]
    pub fn opening(&self) -> Span {
        self.opening
    }

    /// The compiler's static owners and the linter's ordered complexity rows.
    #[must_use]
    pub fn relative(&self) -> &'read TemplateAnalysis {
        &self.products.analysis
    }

    /// All lowering and syntax diagnostics. Their ranges are block-relative.
    #[must_use]
    pub fn diagnostics(&self) -> &'read [Diagnostic] {
        &self.products.diagnostics
    }

    /// Project a complete UTF-8 span into the matching file revision.
    #[must_use]
    pub fn authored_span(&self, relative: Span) -> Option<Span> {
        self.block
            .source()
            .get(relative.start as usize..relative.end as usize)?;
        // Both endpoints are inside a block already validated against a u32-sized root.
        let start = self.block.start();
        Some(Span::new(start + relative.start, start + relative.end))
    }

    /// Materialize the source ranges used by an editor response.
    pub fn report(&self) -> Result<TemplateReport, TemplateReadError> {
        let project = |span| {
            self.authored_span(span)
                .ok_or(TemplateReadError::OutsideBlock(span))
        };
        let mut complexity = self.relative().complexity.clone();
        for row in &mut complexity.contributions {
            row.span = project(row.span)?;
        }
        let mut diagnostics = Vec::with_capacity(self.diagnostics().len());
        for diagnostic in self.diagnostics() {
            diagnostics.push(diagnostic.clone().try_map_spans(project)?);
        }
        Ok(TemplateReport {
            opening: self.opening,
            content: self.block.span(),
            complexity,
            diagnostics,
        })
    }

    /// Stream each unchanged decision with its current file position.
    pub fn authored_contributions(
        &self,
    ) -> impl ExactSizeIterator<Item = Result<Contribution, Span>> + '_ {
        self.relative().complexity.contributions.iter().map(|row| {
            let span = self.authored_span(row.span).ok_or(row.span)?;
            Ok(Contribution { span, ..*row })
        })
    }
}

#[cfg(test)]
mod tests;

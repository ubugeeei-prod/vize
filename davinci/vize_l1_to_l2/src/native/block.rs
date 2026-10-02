//! Parser-owned block admission before any generic factory is allowed to mint.

use super::{Context, NativeEmbed, NativeHole, NativeHoleKind};
use alloc::{boxed::Box, vec::Vec};
use vize_l0::{Allocator, SourceBlock, Span, diag::Diagnostic};
use vize_l1::embed::{Lang, SourceError, syntax::NativeSyntax};
use vize_l1::markup::{ComponentParse, ComponentSourceError};
use vize_l2::artifact::ComponentFactory;
use vize_l2::op::Namespace;

/// A native parser's complete carrier paired with its exact checked source.
/// Private fields prevent pairing a foreign or modified carrier with a block.
pub struct NativeComponent<'a> {
    allocator: &'a Allocator,
    block: SourceBlock<'a>,
    component: ComponentParse<'a>,
}

impl core::fmt::Debug for NativeComponent<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("NativeComponent")
            .field("block", &self.block)
            .field("component", &self.component)
            .finish_non_exhaustive()
    }
}

/// Source rejection retains the original parser owner before any factory call.
#[derive(Debug)]
pub struct RejectedNativeComponent<'a> {
    pub component: NativeComponent<'a>,
}

/// Complete observations from this one construction, beside its factory nodes.
#[derive(Debug)]
pub struct NativeProduced<'a> {
    pub component: NativeComponent<'a>,
    pub holes: Vec<NativeHole>,
    pub diagnostics: Vec<Diagnostic>,
    pub embeds: Vec<NativeEmbed<'a>>,
    pub rejected_syntax: Vec<NativeSyntax<'a>>,
}

impl NativeProduced<'_> {
    /// The bounded contract's admission, independent of file-owned resolution.
    #[must_use]
    pub fn is_supported(&self) -> bool {
        self.holes.is_empty()
    }
}

impl<'a> NativeComponent<'a> {
    /// Parse exactly this checked block once with the genuine native L1 parser.
    pub fn parse_in(
        allocator: &'a Allocator,
        block: SourceBlock<'a>,
    ) -> Result<Self, ComponentSourceError> {
        let component = vize_l1::markup::parse_component(allocator, block.source())?;
        Ok(Self {
            allocator,
            block,
            component,
        })
    }

    #[must_use]
    pub fn block(&self) -> SourceBlock<'a> {
        self.block
    }

    /// Read-only original relative-coordinate errors and complete native tree.
    #[must_use]
    pub fn carrier(&self) -> &ComponentParse<'a> {
        &self.component
    }

    /// Consume source admission when transferring the original carrier alone.
    #[must_use]
    pub fn into_carrier(self) -> ComponentParse<'a> {
        self.component
    }

    /// Construct inside a source-matching factory without reparsing the surface.
    ///
    /// Every expression is handed off once from L1 at its real factory call.
    /// Surface/admission coordinates are rebased during their existing loops;
    /// the original carrier's relative facts and owned diagnostics stay intact.
    pub fn construct_in<R: ComponentFactory<'a>>(
        self,
        region: &mut R,
        lang: Lang,
    ) -> Result<NativeProduced<'a>, Box<RejectedNativeComponent<'a>>> {
        let expected = self.block.root_source();
        let actual = region.source();
        if expected.as_ptr() != actual.as_ptr() || expected.len() != actual.len() {
            return Err(Box::new(RejectedNativeComponent { component: self }));
        }
        let mut cx = Context {
            allocator: self.allocator,
            block: self.block,
            lang,
            holes: Vec::new(),
            diagnostics: Vec::new(),
            embeds: Vec::new(),
            rejected_syntax: Vec::new(),
        };
        for error in &self.component.errors {
            cx.relative_hole(
                region,
                NativeHoleKind::Surface(error.code),
                Span::new(error.offset, error.offset),
            );
        }
        for admission in &self.component.unsupported {
            cx.relative_hole(
                region,
                NativeHoleKind::DirectiveAdmission(admission.error),
                admission.span,
            );
        }
        cx.children(
            region,
            &self.component.tree.children,
            (Namespace::Html, None),
        );
        Ok(NativeProduced {
            component: self,
            holes: cx.holes,
            diagnostics: cx.diagnostics,
            embeds: cx.embeds,
            rejected_syntax: cx.rejected_syntax,
        })
    }
}

impl<'a> Context<'a> {
    fn relative_hole<R: ComponentFactory<'a>>(
        &mut self,
        region: &mut R,
        kind: NativeHoleKind,
        relative: Span,
    ) {
        let span = self
            .block
            .start()
            .checked_add(relative.start)
            .zip(self.block.start().checked_add(relative.end))
            .map(|(start, end)| Span::new(start, end))
            .filter(|span| self.block.contains_block_span(*span));
        match span {
            Some(span) => self.hole(region, kind, span),
            None => self.hole(
                region,
                NativeHoleKind::Source(SourceError::InvalidAuthoredSpan),
                self.block.span(),
            ),
        }
    }
}

#[cfg(test)]
mod observed;
#[cfg(test)]
mod tests;

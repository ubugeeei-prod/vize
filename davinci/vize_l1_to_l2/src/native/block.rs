//! Parser-owned block admission before any generic factory is allowed to mint.

use super::{
    ConstructionFactory, Context, NativeEmbed, NativeHole, NativeHoleKind, NativeObservations,
    PendingNativeComponent,
};
use alloc::{boxed::Box, vec::Vec};
use vize_l0::{Allocator, SourceBlock, Span, diag::Diagnostic};
use vize_l1::embed::{Lang, SourceError, syntax::EmbedSyntax};
use vize_l1::markup::{ComponentParse, ComponentSourceError};
use vize_l2::artifact::ComponentFactory;
use vize_l2::op::Namespace;

/// A native parser's complete carrier paired with its exact checked source.
/// Private fields prevent pairing a foreign or modified carrier with a block.
pub struct NativeComponent<'a> {
    #[cfg(feature = "component-text-condense")]
    pub(super) condense: bool,
    allocator: &'a Allocator,
    block: SourceBlock<'a>,
    component: ComponentParse<'a>,
    #[cfg(feature = "authored-bind-shorthand")]
    pub(super) authored_bind_shorthand: bool,
    #[cfg(feature = "component-listener-queries")]
    pub(super) event_references: bool,
    #[cfg(feature = "lexical-arrow-initializers")]
    pub(super) retained_expressions: bool,
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
    pub rejected_syntax: Vec<EmbedSyntax<'a>>,
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
            #[cfg(feature = "component-text-condense")]
            condense: false,
            allocator,
            block,
            component,
            #[cfg(feature = "authored-bind-shorthand")]
            authored_bind_shorthand: false,
            #[cfg(feature = "component-listener-queries")]
            event_references: false,
            #[cfg(feature = "lexical-arrow-initializers")]
            retained_expressions: false,
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

    #[cfg(test)]
    pub(in crate::native) fn test_allocator(&self) -> &'a Allocator {
        self.allocator
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
        let mut pending = PendingNativeComponent::from_component(self);
        if pending.construct_diagnostic_in(region, lang).is_err() {
            return Err(pending.source_rejection());
        }
        pending
            .into_produced()
            .map_err(|pending| (*pending).source_rejection())
    }
}

/// Both routes borrow the same original owners into the existing single walk.
pub(super) fn construct_component<'a, R: ConstructionFactory<'a>>(
    component: &NativeComponent<'a>,
    observations: &mut NativeObservations<'a>,
    region: &mut R,
    lang: Lang,
) {
    let mut cx = Context {
        #[cfg(feature = "component-text-condense")]
        text_lane: component
            .condense
            .then(|| crate::lower::TextLane::root(true)),
        allocator: component.allocator,
        component: component_input(component),
        lang,
        holes: &mut observations.holes,
        diagnostics: &mut observations.diagnostics,
        embeds: &mut observations.embeds,
        rejected_syntax: &mut observations.rejected_syntax,
    };
    for error in &component.component.errors {
        cx.relative_hole(
            region,
            NativeHoleKind::Surface(error.code),
            Span::new(error.offset, error.offset),
        );
    }
    for admission in &component.component.unsupported {
        cx.relative_hole(
            region,
            NativeHoleKind::DirectiveAdmission(admission.error),
            admission.span,
        );
    }
    cx.children(
        region,
        &component.component.tree.children,
        (Namespace::Html, None),
    );
}

impl<'a> Context<'_, 'a> {
    fn relative_hole<R: ConstructionFactory<'a>>(
        &mut self,
        region: &mut R,
        kind: NativeHoleKind,
        relative: Span,
    ) {
        let span = self
            .block()
            .start()
            .checked_add(relative.start)
            .zip(self.block().start().checked_add(relative.end))
            .map(|(start, end)| Span::new(start, end))
            .filter(|span| self.block().contains_block_span(*span));
        match span {
            Some(span) => self.hole(region, kind, span),
            None => self.hole(
                region,
                NativeHoleKind::Source(SourceError::InvalidAuthoredSpan),
                self.block().span(),
            ),
        }
    }
}

#[cfg(test)]
mod observed;
#[cfg(test)]
mod tests;

#[cfg(not(feature = "lexical-arrow-initializers"))]
pub(super) type ComponentInput<'c, 'a> = &'c NativeComponent<'a>;
#[cfg(not(feature = "lexical-arrow-initializers"))]
pub(super) use core::convert::identity as component_input;

#[cfg(feature = "lexical-arrow-initializers")]
pub(super) struct ComponentInput<'c, 'a> {
    component: &'c NativeComponent<'a>,
    pub(super) expressions: Option<vize_l1::embed::syntax::RetainedExpressionParser<'a>>,
}

#[cfg(feature = "lexical-arrow-initializers")]
pub(super) fn component_input<'c, 'a>(
    component: &'c NativeComponent<'a>,
) -> ComponentInput<'c, 'a> {
    ComponentInput {
        component,
        expressions: component
            .retained_expressions
            .then(|| vize_l1::embed::syntax::RetainedExpressionParser::new(component.allocator)),
    }
}

#[cfg(feature = "lexical-arrow-initializers")]
impl<'a> core::ops::Deref for ComponentInput<'_, 'a> {
    type Target = NativeComponent<'a>;

    fn deref(&self) -> &Self::Target {
        self.component
    }
}

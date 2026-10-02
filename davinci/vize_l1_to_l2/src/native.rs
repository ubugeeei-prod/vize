//! Native Component surface → canonical L2, with one retained L1 expression parse.
//!
//! This bounded producer preserves whitespace and supports ordinary markup,
//! static attributes, mustaches and explicit static named binds.
//! Unsupported Vue constructs stay explicit
//! holes beside real fragments. No product route selects this entry point yet.

use alloc::boxed::Box;
use alloc::vec::Vec;
use vize_l0::diag::{Advisory, Diagnostic, Stage};
use vize_l0::{Allocator, SourceBlock, SourceRoot, Span, String, id::NodeId};
use vize_l1::embed::syntax::{EmbedHole, NativeSyntax, RetainedExpression};
use vize_l1::embed::{Lang, SourceError};
use vize_l1::markup::{ComponentParse, DirectiveNameError};
use vize_l1::{SurfaceChild, Token};
use vize_l2::artifact::{Artifact, ArtifactError, ArtifactParts, Builder, RejectedArtifact};
use vize_l2::op::{Namespace, Region};
use vize_l2::provenance::ProvenanceRecord;

mod construction;
pub(crate) use construction::{ConstructionBody, ConstructionFactory};
mod pending;
pub(crate) use pending::{NativeObservations, PendingNativeComponent};
mod block;
pub use block::{NativeComponent, NativeProduced, RejectedNativeComponent};
mod element;
mod expression;
mod pattern;
mod text;
pub use expression::{NativeExpressionError, retain_expression_in};

/// An unsupported or recovered source construct, never a legacy fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeHoleKind {
    Surface(vize_l0::ErrorCode),
    MissingMarkup,
    UnexpectedMarkup,
    UnsupportedTag,
    Directive,
    DirectiveSyntax,
    DirectiveAdmission(DirectiveNameError),
    PreCarrier,
    Embed(EmbedHole),
    Source(SourceError),
    ExpressionCoordinates(vize_l2::expr::js::JsCoordinateError),
    /// Vue's simple-identifier spelling for outer NBSP/BOM is not admitted yet.
    InterpolationIdentifierTrivia,
    Construction(ArtifactError),
}

/// Source evidence for a construct this bounded native producer cannot admit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeHole {
    pub span: Span,
    pub kind: NativeHoleKind,
}

/// Complete original syntax observations, keyed by their produced L2 node.
#[derive(Debug)]
pub struct NativeEmbed<'a> {
    pub node: Option<NodeId>,
    pub syntax: RetainedExpression<'a>,
}

/// A real canonical fragment artifact with explicit native incompleteness.
#[derive(Debug)]
pub struct NativeLowered<'a> {
    pub artifact: Artifact<'a>,
    pub holes: Vec<NativeHole>,
    pub diagnostics: Vec<Diagnostic>,
    /// The original native carrier, including all errors and admission facts.
    pub component: ComponentParse<'a>,
    pub embeds: Vec<NativeEmbed<'a>>,
    /// Retain the original owning observations even on an unexpected shape.
    pub rejected_syntax: Vec<NativeSyntax<'a>>,
}

impl NativeLowered<'_> {
    /// Completeness for this bounded preserve-whitespace contract only.
    #[must_use]
    pub fn is_supported(&self) -> bool {
        self.holes.is_empty()
    }
}

/// Use the actual native L1 Component parser, then construct canonical nodes.
///
/// The caller supplies JS/TS; file language selection remains unfinished.
/// Remaining directive/scope/control-flow semantics, Vue-special carriers, recovered
/// missing owners, condense whitespace and product integration remain holes
/// or later contracts. L1 comments and full parser diagnostics stay retained.
pub fn lower_component_native<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    lang: Lang,
) -> Result<NativeLowered<'a>, RejectedArtifact<'a>> {
    let mut builder = Builder::new(allocator, source)
        .map_err(|error| empty_rejection(allocator, source, error))?;
    let block = SourceRoot::new(source)
        .map_err(|_| empty_rejection(allocator, source, ArtifactError::SourceLimit))?
        .whole_block();
    let component = NativeComponent::parse_in(allocator, block)
        .map_err(|_| empty_rejection(allocator, source, ArtifactError::SourceLimit))?;
    let produced = component
        .construct_in(&mut builder.region(), lang)
        .map_err(|_| empty_rejection(allocator, source, ArtifactError::SourceLimit))?;
    Ok(NativeLowered {
        artifact: builder.finish()?,
        holes: produced.holes,
        diagnostics: produced.diagnostics,
        component: produced.component.into_carrier(),
        embeds: produced.embeds,
        rejected_syntax: produced.rejected_syntax,
    })
}

fn empty_rejection<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    error: ArtifactError,
) -> RejectedArtifact<'a> {
    RejectedArtifact {
        error,
        parts: Box::new(ArtifactParts {
            source,
            root: Region {
                ops: vize_l0::Vec::new_in(&allocator),
            },
            provenance: Vec::new(),
            scopes: vize_l0::side_table::SideTable::new(),
        }),
    }
}

struct Context<'component, 'a> {
    allocator: &'a Allocator,
    component: &'component NativeComponent<'a>,
    lang: Lang,
    holes: &'component mut Vec<NativeHole>,
    diagnostics: &'component mut Vec<Diagnostic>,
    embeds: &'component mut Vec<NativeEmbed<'a>>,
    rejected_syntax: &'component mut Vec<NativeSyntax<'a>>,
}

impl<'a> Context<'_, 'a> {
    fn block(&self) -> SourceBlock<'a> {
        self.component.block()
    }

    fn children<R: ConstructionFactory<'a>>(
        &mut self,
        region: &mut R,
        children: &[SurfaceChild<'a>],
        parent: (Namespace, Option<&'a str>),
    ) {
        vize_l0::ensure_sufficient_stack(|| {
            for child in children {
                match child {
                    SurfaceChild::Element(element) => self.element(region, element, parent),
                    SurfaceChild::Interpolation(interpolation) => {
                        self.interpolation(region, interpolation)
                    }
                    SurfaceChild::Text(token) => self.text(region, token),
                    SurfaceChild::Comment(token) => self.comment(region, token),
                    SurfaceChild::Cdata(token)
                    | SurfaceChild::ProcessingInstruction(token)
                    | SurfaceChild::Unexpected(token) => self.hole(
                        region,
                        NativeHoleKind::UnexpectedMarkup,
                        self.token_span(token),
                    ),
                }
            }
        });
    }

    fn token_span(&self, token: &Token<'_>) -> Span {
        self.block().span_of(token.text).unwrap_or(Span::new(0, 0))
    }

    fn record<R: ConstructionFactory<'a>>(
        &mut self,
        region: &mut R,
        rule: &'static str,
        node: Option<NodeId>,
        span: Span,
        after: &'static str,
    ) {
        let before = self
            .block()
            .root_source()
            .get(span.start as usize..span.end as usize)
            .unwrap_or("");
        if let Err(error) = region.record(ProvenanceRecord {
            rule: String::from(rule),
            node,
            before: String::from(before),
            after: String::from(after),
            span,
        }) {
            self.holes.push(NativeHole {
                span,
                kind: NativeHoleKind::Construction(error),
            });
        }
    }

    fn hole<R: ConstructionFactory<'a>>(
        &mut self,
        region: &mut R,
        kind: NativeHoleKind,
        span: Span,
    ) {
        self.holes.push(NativeHole { span, kind });
        self.diagnostics.push(Diagnostic::new(
            Advisory::Warning,
            Stage::Semantic,
            span,
            String::from(
                "This construct is not supported by the current native Component lowering.",
            ),
        ));
        self.record(region, "native.unsupported", None, span, "");
    }

    fn produced<R: ConstructionFactory<'a>>(
        &mut self,
        region: &mut R,
        result: Result<NodeId, ArtifactError>,
        rule: &'static str,
        span: Span,
        after: &'static str,
    ) -> Option<NodeId> {
        match result {
            Ok(node) => {
                self.record(region, rule, Some(node), span, after);
                Some(node)
            }
            Err(error) => {
                self.hole(region, NativeHoleKind::Construction(error), span);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests;

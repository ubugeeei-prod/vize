//! The actual L1 consuming handoff, never a parse from L2 or transformed text.

use super::{Context, NativeEmbed, NativeHoleKind};
use vize_l0::{Allocator, Span, Vec};
use vize_l1::Interpolation;
use vize_l1::embed::syntax::{EmbedHole, RetainedExpression, parse_once};
use vize_l1::embed::{DecodeSegmentKind, Embed, EmbedSource, Grammar, Shape};
use vize_l2::artifact::{ArtifactError, RegionBuilder};
use vize_l2::expr::js::{JsCoordinateError, JsCoordinates, JsSegment};
use vize_l2::expr::{ExprRef, JsExpr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeExpressionError {
    Hole(EmbedHole),
    Coordinates(JsCoordinateError),
}

/// Convert only a real retained L1 Expr handoff into the neutral L2 view.
/// The checked source, parser prefix and existing decode correspondences are
/// transferred without another HTML decoding or expression/AST parse.
pub fn retain_expression_in<'a>(
    allocator: &'a Allocator,
    file: &'a str,
    syntax: &RetainedExpression<'a>,
) -> Result<&'a JsExpr<'a>, NativeExpressionError> {
    let ast = syntax.expression().ok_or(NativeExpressionError::Hole(
        syntax.hole().unwrap_or(EmbedHole::InvalidExpressionShape),
    ))?;
    let source = syntax.source();
    let mut segments = Vec::new_in(&allocator);
    if let Some(map) = source.decode_map() {
        for segment in map.segments() {
            segments.push(JsSegment {
                decoded: segment.decoded(),
                authored: segment.authored(),
                entity: segment.kind() == DecodeSegmentKind::Entity,
            });
        }
    }
    let segments = segments.into_boxed_slice().into_arena_slice();
    let coordinates = JsCoordinates::checked(
        file,
        source.text(),
        source.span(),
        syntax.parser_prefix(),
        segments,
    )
    .map_err(NativeExpressionError::Coordinates)?;
    JsExpr::from_retained_in(allocator, ast, source.text(), source.span(), coordinates)
        .map_err(NativeExpressionError::Coordinates)
}

impl<'a> Context<'a> {
    pub(super) fn interpolation(
        &mut self,
        region: &mut RegionBuilder<'_, 'a>,
        interpolation: &Interpolation<'a>,
    ) {
        let span = Span::new(
            self.token_span(&interpolation.open).start,
            self.token_span(&interpolation.close).end,
        );
        let content = interpolation.content.text;
        let source_span = self.block.span_of(content).unwrap_or(Span::new(0, 0));
        let source = match EmbedSource::authored(self.block.root_source(), source_span) {
            Ok(source) => source,
            Err(error) => {
                self.hole(region, NativeHoleKind::Source(error), source_span);
                return;
            }
        };
        self.expression(
            region,
            source,
            span,
            "native.interpolation",
            "ui.interpolation",
            |region, expression| region.interpolation(expression, span),
        );
    }

    pub(super) fn expression(
        &mut self,
        region: &mut RegionBuilder<'_, 'a>,
        source: EmbedSource<'a>,
        span: Span,
        rule: &'static str,
        after: &'static str,
        construct: impl FnOnce(
            &mut RegionBuilder<'_, 'a>,
            ExprRef<'a>,
        ) -> Result<vize_l0::id::NodeId, ArtifactError>,
    ) {
        let source_span = source.span();
        let syntax = parse_once(
            self.allocator,
            Embed {
                grammar: Grammar {
                    shape: Shape::Expr,
                    lang: self.lang,
                },
                source,
            },
        );
        let retained = match syntax.into_expression() {
            Ok(retained) => retained,
            Err(syntax) => {
                self.rejected_syntax.push(*syntax);
                self.hole(
                    region,
                    NativeHoleKind::Embed(EmbedHole::UnsupportedShape),
                    source_span,
                );
                return;
            }
        };
        let node = match retain_expression_in(self.allocator, self.block.root_source(), &retained) {
            Ok(js) => {
                let result = construct(region, ExprRef::Js(js));
                self.produced(region, result, rule, span, after)
            }
            Err(error) => {
                let kind = match error {
                    NativeExpressionError::Hole(hole) => NativeHoleKind::Embed(hole),
                    NativeExpressionError::Coordinates(error) => {
                        NativeHoleKind::ExpressionCoordinates(error)
                    }
                };
                self.hole(region, kind, source_span);
                None
            }
        };
        self.embeds.push(NativeEmbed {
            node,
            syntax: retained,
        });
    }
}

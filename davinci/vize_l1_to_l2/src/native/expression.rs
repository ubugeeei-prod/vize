//! The actual L1 consuming handoff, never a parse from L2 or transformed text.

use super::{Context, NativeEmbed, NativeHoleKind};
use vize_l0::{Allocator, Span, Vec};
use vize_l1::Interpolation;
use vize_l1::embed::prepare_vue_interpolation_in;
use vize_l1::embed::syntax::{EmbedHole, RetainedExpression, parse_once};
use vize_l1::embed::{DecodeSegmentKind, Embed, EmbedSource, Grammar, Shape};
use vize_l2::artifact::{ArtifactError, ComponentFactory};
use vize_l2::expr::JsExpr;
use vize_l2::expr::js::{JsCoordinateError, JsCoordinates, JsSegment};

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
    pub(super) fn interpolation<R: ComponentFactory<'a>>(
        &mut self,
        region: &mut R,
        interpolation: &Interpolation<'a>,
    ) {
        let span = Span::new(
            self.token_span(&interpolation.open).start,
            self.token_span(&interpolation.close).end,
        );
        let content = interpolation.content.text;
        let source_span = self.block.span_of(content).unwrap_or(Span::new(0, 0));
        let source = match prepare_vue_interpolation_in(
            self.allocator,
            self.block.root_source(),
            source_span,
        ) {
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
            ("native.interpolation", "ui.interpolation"),
            admit_interpolation,
            |region, expression| region.interpolation(expression, span),
        );
    }

    pub(super) fn expression<R: ComponentFactory<'a>>(
        &mut self,
        region: &mut R,
        source: EmbedSource<'a>,
        span: Span,
        (rule, after): (&'static str, &'static str),
        admit: impl FnOnce(&RetainedExpression<'a>) -> Result<(), NativeHoleKind>,
        construct: impl FnOnce(&mut R, &'a JsExpr<'a>) -> Result<vize_l0::id::NodeId, ArtifactError>,
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
        let retained = match syntax.into_expression(self.allocator) {
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
        let expression = match admit(&retained) {
            Ok(()) => retain_expression_in(self.allocator, self.block.root_source(), &retained),
            Err(kind) => {
                self.hole(region, kind, source_span);
                self.embeds.push(NativeEmbed {
                    node: None,
                    syntax: retained,
                });
                return;
            }
        };
        let node = match expression {
            Ok(js) => {
                let result = construct(region, js);
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

fn admit_interpolation(syntax: &RetainedExpression<'_>) -> Result<(), NativeHoleKind> {
    // This is dialect spelling policy on the real parsed root and actual source
    // edges, not a grammar heuristic. Characters inside comments and literals
    // do not trigger the refusal, and generic retained expressions stay neutral.
    if matches!(
        syntax.expression(),
        Some(oxc_ast::ast::Expression::Identifier(_))
    ) {
        let text = syntax.source().text();
        let edge = |ch| matches!(ch, '\u{00a0}' | '\u{feff}');
        if text.chars().next().is_some_and(edge) || text.chars().next_back().is_some_and(edge) {
            return Err(NativeHoleKind::InterpolationIdentifierTrivia);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;

//! The actual L1 consuming handoff, never a parse from L2 or transformed text.

use super::{Context, NativeEmbed, NativeHoleKind};
use crate::native::ConstructionFactory;
use vize_l0::{Allocator, Span, Vec};
use vize_l1::embed::prepare_vue_interpolation_in;
use vize_l1::embed::syntax::{EmbedHole, RetainedExpression, parse_once};
use vize_l1::embed::{DecodeSegmentKind, Embed, EmbedSource, Grammar, Shape};
use vize_l1::markup::Interpolation;
use vize_l2::artifact::ArtifactError;
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

impl<'a> Context<'_, 'a> {
    pub(super) fn interpolation<R: ConstructionFactory<'a>>(
        &mut self,
        region: &mut R,
        interpolation: &Interpolation<'a>,
    ) {
        let span = Span::new(
            self.token_span(&interpolation.open).start,
            self.token_span(&interpolation.close).end,
        );
        let content = interpolation.content.text;
        let source_span = self.block().span_of(content).unwrap_or(Span::new(0, 0));
        let source = match prepare_vue_interpolation_in(
            self.allocator,
            self.block().root_source(),
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

    pub(super) fn expression<R: ConstructionFactory<'a>>(
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
        let expression = admit(&retained).and_then(|()| {
            retain_expression_in(self.allocator, self.block().root_source(), &retained).map_err(
                |error| match error {
                    NativeExpressionError::Hole(hole) => NativeHoleKind::Embed(hole),
                    NativeExpressionError::Coordinates(error) => {
                        NativeHoleKind::ExpressionCoordinates(error)
                    }
                },
            )
        });
        // Keep the owning observation before lower policy/provenance callbacks.
        self.embeds.push(NativeEmbed {
            node: None,
            syntax: retained,
        });
        match expression {
            Ok(js) => {
                let result = construct(region, js);
                if let Ok(node) = result
                    && let Some(embed) = self.embeds.last_mut()
                {
                    embed.node = Some(node);
                }
                self.produced(region, result, rule, span, after);
            }
            Err(kind) => {
                self.hole(region, kind, source_span);
            }
        }
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
mod tests {
    use crate::native::{NativeHoleKind, lower_component_native};
    use vize_l0::{Allocator, Span, cstr};
    use vize_l1::embed::Lang;
    use vize_l2::expr::ExprRef;
    use vize_l2::op::{BindingOp, Op};

    #[test]
    fn actual_interpolation_window_keeps_once_ast_entities_and_full_operation_source() {
        let a = Allocator::default();
        let file = "<p>{{ \t&fjlig; + 作者 \r\n}}</p>";
        let lowered = lower_component_native(&a, file, Lang::Js).unwrap();
        assert!(lowered.is_supported(), "{:?}", lowered.holes);
        let retained = &lowered.embeds.first().unwrap().syntax;
        assert_eq!(retained.source().text(), "fj + 作者");
        let start = file.find("&fjlig;").unwrap() as u32;
        let end = file.find(" \r\n}}").unwrap() as u32;
        assert_eq!(retained.source().span(), Span::new(start, end));
        assert!(retained.source().decode_map().is_some());
        let [Op::Element(owner)] = lowered.artifact.root().ops.as_slice() else {
            panic!("original owner");
        };
        let [Op::Interpolation(interpolation)] = owner.children.ops.as_slice() else {
            panic!("actual interpolation");
        };
        let ExprRef::Js(js) = interpolation.expression else {
            panic!("retained JS");
        };
        assert!(core::ptr::eq(js.ast, retained.expression().unwrap()));
        assert_eq!(js.span, retained.source().span());
        assert_eq!(js.source, "fj + 作者");
        assert_eq!(
            js.authored_span(Span::new(0, 2)),
            Some(Span::new(start, start + 7))
        );
        assert_eq!(js.authored_span(Span::new(0, 1)), None);
        assert!(js.matches_authored_source(file));
        let open = file.find("{{").unwrap() as u32;
        let close = file.find("}}").unwrap() as u32 + 2;
        assert_eq!(interpolation.span, Span::new(open, close));
        let provenance = lowered
            .artifact
            .provenance()
            .iter()
            .find(|record| record.rule.as_str() == "native.interpolation")
            .unwrap();
        assert_eq!(provenance.span, interpolation.span);
        assert_eq!(provenance.before.as_str(), "{{ \t&fjlig; + 作者 \r\n}}");
        assert_eq!(lowered.artifact.node_count(), 2);
    }

    #[test]
    fn trimmed_line_comment_keeps_original_observations_and_refuses_before_mint() {
        let a = Allocator::default();
        let file = "<p>{{ value //tail \n}}{{good}}after</p>";
        let lowered = lower_component_native(&a, file, Lang::Js).unwrap();
        assert!(!lowered.is_supported());
        assert!(lowered.holes.iter().any(|hole| hole.kind
            == NativeHoleKind::ExpressionCoordinates(
                vize_l2::expr::js::JsCoordinateError::OutsideExpression
            )));
        let first = lowered.embeds.first().unwrap();
        assert_eq!(first.node, None);
        assert_eq!(first.syntax.source().text(), "value //tail");
        assert_eq!(first.syntax.hole(), None);
        assert!(first.syntax.expression().is_some());
        assert_eq!(
            first.syntax.comments().next().unwrap().text().unwrap(),
            "//tail"
        );
        let [_, second, ..] = lowered.embeds.as_slice() else {
            panic!("retained later expression");
        };
        assert_eq!(second.node.unwrap().index(), 1);
        assert_eq!(lowered.artifact.node_count(), 3);
    }

    #[test]
    fn actual_identifier_edge_nbsp_and_bom_keep_syntax_but_refuse_before_coordinates_and_mint() {
        for (prefix, suffix) in [
            ("\u{00a0}", ""),
            ("", "\u{00a0}"),
            ("\u{feff}", ""),
            ("", "\u{feff}"),
            ("&#160;", ""),
            ("", "&#65279;"),
        ] {
            let a = Allocator::default();
            let file = cstr!("<p>{{{{ {prefix}msg{suffix} }}}}{{{{kept}}}}</p>");
            let lowered = lower_component_native(&a, &file, Lang::Js).unwrap();
            assert!(!lowered.is_supported(), "{prefix:?}/{suffix:?}");
            assert_eq!(lowered.holes.len(), 1);
            assert_eq!(
                lowered.holes.first().unwrap().kind,
                NativeHoleKind::InterpolationIdentifierTrivia
            );
            let first = lowered.embeds.first().unwrap();
            assert_eq!(first.node, None);
            assert_eq!(first.syntax.hole(), None);
            assert_eq!(first.syntax.diagnostics().count(), 0);
            assert!(matches!(
                first.syntax.expression(),
                Some(oxc_ast::ast::Expression::Identifier(_))
            ));
            assert!(core::ptr::eq(
                lowered.artifact.source().as_ptr(),
                file.as_ptr()
            ));
            let [_, second, ..] = lowered.embeds.as_slice() else {
                panic!("retained later expression");
            };
            assert_eq!(second.node.unwrap().index(), 1);
            assert_eq!(lowered.artifact.node_count(), 2);
        }
    }

    #[test]
    fn characters_inside_original_comments_and_literals_do_not_trigger_edge_refusal() {
        for (expression, comments) in [
            ("/*\u{00a0}*/msg", 1),
            ("msg/*\u{feff}*/", 1),
            ("'\u{00a0}msg'", 0),
            ("'msg\u{feff}'", 0),
            ("/*&#160;*/msg", 1),
            ("msg/*&#65279;*/", 1),
            ("'&#160;msg'", 0),
        ] {
            let a = Allocator::default();
            let file = cstr!("{{{{ {expression} }}}}");
            let lowered = lower_component_native(&a, &file, Lang::Js).unwrap();
            assert!(
                lowered.is_supported(),
                "{expression:?}: {:?}",
                lowered.holes
            );
            assert_eq!(lowered.artifact.node_count(), 1);
            let retained = &lowered.embeds.first().unwrap().syntax;
            assert_eq!(retained.hole(), None);
            let [Op::Interpolation(interpolation)] = lowered.artifact.root().ops.as_slice() else {
                panic!("original source expression");
            };
            let ExprRef::Js(js) = interpolation.expression else {
                panic!("original AST");
            };
            assert!(core::ptr::eq(js.ast, retained.expression().unwrap()));
            assert!(js.matches_authored_source(&file));
            assert_eq!(retained.comments().count(), comments);
        }
    }

    #[test]
    fn interpolation_spelling_policy_does_not_change_neutral_named_binding_admission() {
        let a = Allocator::default();
        let file = "<p :value=\"&#160;msg\">{{msg&#160;}}</p>";
        let lowered = lower_component_native(&a, file, Lang::Js).unwrap();
        assert!(!lowered.is_supported());
        assert_eq!(
            lowered.holes.first().unwrap().kind,
            NativeHoleKind::InterpolationIdentifierTrivia
        );
        let [Op::Element(owner)] = lowered.artifact.root().ops.as_slice() else {
            panic!("binding owner");
        };
        let [BindingOp::Bind(binding)] = owner.bindings.as_slice() else {
            panic!("actual named binding");
        };
        let Some(ExprRef::Js(js)) = binding.value else {
            panic!("retained bound expression");
        };
        assert_eq!(js.source, "\u{00a0}msg");
        assert_eq!(lowered.embeds.first().unwrap().node.unwrap().index(), 1);
        let [_, second, ..] = lowered.embeds.as_slice() else {
            panic!("retained refused interpolation");
        };
        assert_eq!(second.node, None);
        assert!(owner.children.ops.is_empty());
        assert_eq!(lowered.artifact.node_count(), 2);
    }
}

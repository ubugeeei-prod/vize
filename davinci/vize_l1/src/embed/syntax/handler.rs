//! Consuming the original parser-owned handler bodies and normal observations.

use alloc::boxed::Box;
use oxc_ast::ast::FunctionBody;
use oxc_diagnostics::Diagnostics;
use oxc_parser::{AdmittedHandlerBody, HandlerBodyObservation};
use oxc_span::SourceType;
use vize_l0::Span;

use super::{
    CommentView, DiagnosticView, EmbedHole, EmbedSource, Grammar, NativeSyntax, Shape, SourceError,
    coordinates::Coordinates,
};

/// Original arena body authenticated by its complete ordinary parser owner.
/// Raw arena roots can outlive this owner; admission remains a short owner borrow.
///
/// ```compile_fail
/// use vize_l1::embed::syntax::RetainedHandlerBody;
/// fn substitute<'a>(owner: &mut RetainedHandlerBody<'a>) {
///     owner.observation = None;
/// }
/// ```
pub struct RetainedHandlerBody<'a> {
    grammar: Grammar,
    source_type: SourceType,
    coordinates: Coordinates<'a>,
    observation: Option<HandlerBodyObservation<'a>>,
    diagnostics: Diagnostics,
    hole: Option<EmbedHole>,
}

impl core::fmt::Debug for RetainedHandlerBody<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RetainedHandlerBody")
            .field("grammar", &self.grammar)
            .field("source_type", &self.source_type)
            .field("source", &self.source())
            .field("hole", &self.hole)
            .field("diagnostic_count", &self.diagnostics().count())
            .finish_non_exhaustive()
    }
}

impl<'a> RetainedHandlerBody<'a> {
    #[must_use]
    pub const fn grammar(&self) -> Grammar {
        self.grammar
    }

    #[must_use]
    pub const fn source_type(&self) -> SourceType {
        self.source_type
    }

    #[must_use]
    pub const fn source(&self) -> EmbedSource<'a> {
        self.coordinates.source
    }

    /// Actual generated prefix bytes in the original parser coordinates.
    #[must_use]
    pub const fn parser_prefix(&self) -> u32 {
        self.coordinates.prefix
    }

    #[must_use]
    /// Original stock admission borrowed from this owner. It does not establish
    /// event directive provenance, file association or identifier resolution.
    pub fn admitted_body(&self) -> Option<AdmittedHandlerBody<'_, 'a>> {
        if self.hole.is_some() {
            return None;
        }
        self.observation.as_ref()?.admitted()
    }

    /// The generated FunctionBody container is not an authored source range;
    /// only its directives and statements may be projected to authored spans.
    #[must_use]
    pub fn body(&self) -> Option<&'a FunctionBody<'a>> {
        self.admitted_body().map(|admitted| admitted.body())
    }

    #[must_use]
    pub const fn hole(&self) -> Option<EmbedHole> {
        self.hole
    }

    pub fn comments(&self) -> impl Iterator<Item = CommentView<'_, 'a>> {
        self.observation
            .iter()
            .flat_map(|owner| owner.comments().iter())
            .map(|comment| CommentView {
                comment,
                coordinates: self.coordinates,
            })
    }

    pub fn diagnostics(&self) -> impl Iterator<Item = DiagnosticView<'_, 'a>> {
        self.diagnostics
            .iter()
            .chain(
                self.observation
                    .iter()
                    .flat_map(|owner| owner.diagnostics().iter()),
            )
            .map(|diagnostic| DiagnosticView {
                diagnostic,
                coordinates: self.coordinates,
            })
    }

    pub fn decoded_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.coordinates.decoded_span(span)
    }

    pub fn authored_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.source().authored_span(self.decoded_span(span)?)
    }
}

pub(super) fn into_handler_body<'a>(
    mut syntax: NativeSyntax<'a>,
) -> Result<RetainedHandlerBody<'a>, Box<NativeSyntax<'a>>> {
    if syntax.grammar.shape != Shape::HandlerBody {
        return Err(Box::new(syntax));
    }
    let observation = if let Some(owner) = syntax.embedding.take() {
        match owner.into_handler_body() {
            Ok(owner) => Some(owner),
            Err(owner) => {
                syntax.embedding = Some(*owner);
                return Err(Box::new(syntax));
            }
        }
    } else {
        None
    };
    let NativeSyntax {
        grammar,
        source_type,
        coordinates,
        observation: _,
        embedding: _,
        diagnostics,
        hole,
    } = syntax;
    let hole = hole.or_else(|| {
        observation
            .as_ref()
            .and_then(|owner| owner.hole())
            .map(super::wrapped::hole)
    });
    Ok(RetainedHandlerBody {
        grammar,
        source_type,
        coordinates,
        observation,
        diagnostics,
        hole,
    })
}

#[cfg(test)]
mod tests {
    use alloc::borrow::ToOwned;
    use oxc_ast::ast::{Expression, Statement};
    use oxc_span::{GetSpan, SourceType};
    use vize_l0::{Allocator, Span};

    use super::super::{Embed, EmbedHole, EmbedSource, Grammar, Lang, Shape, parse_once};
    use crate::embed::prepare_attribute_value;

    fn embed(text: &str, shape: Shape, lang: Lang) -> Embed<'_> {
        Embed {
            grammar: Grammar { shape, lang },
            source: EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap(),
        }
    }

    #[test]
    fn original_statement_and_directive_children_survive_consumption_and_owner_drop() {
        let arena = Allocator::default();
        let text = "'use strict'; const α = 1; return α; /*kept*/";
        let syntax = parse_once(&arena, embed(text, Shape::HandlerBody, Lang::Js));
        let original = syntax.handler_body().unwrap();
        let statements = original.statements().as_ptr();
        let directives = original.directives().as_ptr();
        let Statement::ReturnStatement(returned) = &original.statements()[1] else {
            panic!("actual return statement")
        };
        let descendant = core::ptr::from_ref(returned.argument.as_ref().unwrap());
        let comments = syntax.embedding.as_ref().unwrap().comments().as_ptr();
        let retained = syntax.into_handler_body().unwrap();
        assert!(core::mem::needs_drop::<super::RetainedHandlerBody<'_>>());
        assert_eq!(retained.hole(), None);
        let body = retained.body().unwrap();
        assert_eq!(body.statements.as_ptr(), statements);
        assert_eq!(body.directives.as_ptr(), directives);
        assert_eq!(
            retained.observation.as_ref().unwrap().comments().as_ptr(),
            comments
        );
        let Statement::ReturnStatement(returned) = &body.statements[1] else {
            panic!("original return statement")
        };
        assert_eq!(
            core::ptr::from_ref(returned.argument.as_ref().unwrap()),
            descendant
        );
        assert_eq!(
            retained.comments().next().unwrap().text().unwrap(),
            "/*kept*/"
        );
        assert!(retained.authored_span(body.span).is_err());
        assert_eq!(
            retained.authored_span(body.directives[0].span),
            Ok(Span::new(0, 13))
        );
        drop(retained);
        assert_eq!(body.statements.len(), 2);
        let Statement::ReturnStatement(returned) = &body.statements[1] else {
            panic!("arena body outlives owner")
        };
        assert!(matches!(returned.argument, Some(Expression::Identifier(_))));
    }

    #[test]
    fn admission_keeps_the_original_body_content_profile_and_parser_window() {
        let arena = Allocator::default();
        for lang in [Lang::Js, Lang::Ts] {
            let text = "return count // tail";
            let syntax = parse_once(&arena, embed(text, Shape::HandlerBody, lang));
            let statements = syntax.handler_body().unwrap().statements().as_ptr();
            let retained = syntax.into_handler_body().unwrap();
            let admitted = retained.admitted_body().unwrap();
            assert_eq!(admitted.body().statements.as_ptr(), statements);
            assert!(core::ptr::eq(admitted.content(), text));
            assert!(core::ptr::eq(retained.source().text(), admitted.content()));
            assert_eq!(admitted.parser_content_span(), oxc_span::Span::new(6, 26));
            assert_eq!(retained.parser_prefix(), 6);
            assert_eq!(admitted.source_type(), retained.source_type());
            assert!(admitted.source_type().is_module());
            assert_eq!(admitted.source_type().is_typescript(), lang == Lang::Ts);
            assert_eq!(
                retained.grammar(),
                Grammar {
                    shape: Shape::HandlerBody,
                    lang
                }
            );
            assert_eq!(retained.source().span(), Span::new(0, text.len() as u32));
        }
    }

    #[test]
    fn once_decoded_unicode_entity_maps_keep_original_body_and_comments() {
        let arena = Allocator::default();
        let file = "前/*kept*/ return α &amp;&amp; β;後";
        let source =
            prepare_attribute_value(&arena, file, Span::new(3, file.len() as u32 - 3)).unwrap();
        let map = source.decode_map().unwrap();
        let syntax = parse_once(
            &arena,
            Embed {
                grammar: Grammar {
                    shape: Shape::HandlerBody,
                    lang: Lang::Js,
                },
                source,
            },
        );
        let comments = syntax.comments().next().unwrap().text().unwrap().as_ptr();
        let retained = syntax.into_handler_body().unwrap();
        assert!(core::ptr::eq(retained.source().text(), source.text()));
        assert!(core::ptr::eq(
            retained.source().decode_map().unwrap().segments().as_ptr(),
            map.segments().as_ptr()
        ));
        assert_eq!(
            retained.comments().next().unwrap().text().unwrap().as_ptr(),
            comments
        );
        assert_eq!(
            retained.comments().next().unwrap().authored_span(),
            Ok(Span::new(3, 11))
        );
        let Statement::ReturnStatement(returned) = &retained.body().unwrap().statements[0] else {
            panic!("return")
        };
        let Expression::LogicalExpression(logical) = returned.argument.as_ref().unwrap() else {
            panic!("logical")
        };
        assert_eq!(
            retained.authored_span(logical.right.span()),
            Ok(Span::new(
                file.find('β').unwrap() as u32,
                file.find('β').unwrap() as u32 + 2
            ))
        );
    }

    #[test]
    fn ts_annotations_remain_original_and_js_diagnostics_are_preserved() {
        let arena = Allocator::default();
        let text = "const count: number = 1; return count;";
        let syntax = parse_once(&arena, embed(text, Shape::HandlerBody, Lang::Ts));
        let original = syntax.handler_body().unwrap().statements().as_ptr();
        let retained = syntax.into_handler_body().unwrap();
        assert_eq!(retained.body().unwrap().statements.as_ptr(), original);
        assert_eq!(retained.source_type(), SourceType::ts().with_module(true));
        let syntax = parse_once(&arena, embed(text, Shape::HandlerBody, Lang::Js));
        let count = syntax.diagnostics().count();
        assert!(count > 0);
        let retained = syntax.into_handler_body().unwrap();
        assert_eq!(retained.hole(), Some(EmbedHole::Syntax));
        assert_eq!(retained.diagnostics().count(), count);
        assert!(retained.body().is_none() && retained.admitted_body().is_none());
    }

    #[test]
    fn complete_original_syntax_diagnostics_and_comments_survive_local_failure() {
        let arena = Allocator::default();
        let syntax = parse_once(
            &arena,
            embed("return /x/uv; // kept", Shape::HandlerBody, Lang::Js),
        );
        let diagnostics: alloc::vec::Vec<_> = syntax
            .diagnostics()
            .map(|d| {
                (
                    d.message().as_ptr(),
                    d.message().to_owned(),
                    alloc::format!("{:?}", d.code()),
                    d.url().map(str::to_owned),
                    d.severity(),
                    d.help().map(str::to_owned),
                    d.note().map(str::to_owned),
                    d.labels()
                        .map(|l| {
                            (
                                l.message().map(str::to_owned),
                                l.primary(),
                                l.decoded_span(),
                                l.authored_span(),
                            )
                        })
                        .collect::<alloc::vec::Vec<_>>(),
                )
            })
            .collect();
        assert!(!diagnostics.is_empty());
        let comments = syntax.comments().next().unwrap().text().unwrap().as_ptr();
        let retained = syntax.into_handler_body().unwrap();
        let after: alloc::vec::Vec<_> = retained
            .diagnostics()
            .map(|d| {
                (
                    d.message().as_ptr(),
                    d.message().to_owned(),
                    alloc::format!("{:?}", d.code()),
                    d.url().map(str::to_owned),
                    d.severity(),
                    d.help().map(str::to_owned),
                    d.note().map(str::to_owned),
                    d.labels()
                        .map(|l| {
                            (
                                l.message().map(str::to_owned),
                                l.primary(),
                                l.decoded_span(),
                                l.authored_span(),
                            )
                        })
                        .collect::<alloc::vec::Vec<_>>(),
                )
            })
            .collect();
        assert_eq!(after, diagnostics);
        assert_eq!(
            retained.comments().next().unwrap().text().unwrap().as_ptr(),
            comments
        );
        assert!(retained.body().is_none() && retained.admitted_body().is_none());
    }

    #[test]
    fn invalid_context_wrapper_and_local_budget_holes_never_grant_admission() {
        let arena = Allocator::default();
        for text in [
            "import count from 'pkg';",
            "return await count;",
            "}; count; ()=>{",
        ] {
            let syntax = parse_once(&arena, embed(text, Shape::HandlerBody, Lang::Js));
            let expected = syntax.hole();
            assert!(expected.is_some());
            let retained = syntax.into_handler_body().unwrap();
            assert_eq!(retained.hole(), expected);
            assert!(retained.body().is_none() && retained.admitted_body().is_none());
            assert_eq!(retained.source().text(), text);
        }
        let text = ";".repeat(32);
        let before = arena.allocated_bytes();
        let retained = parse_once(&arena, embed(&text, Shape::HandlerBody, Lang::Js))
            .into_handler_body()
            .unwrap();
        assert_eq!(retained.hole(), Some(EmbedHole::TokenBudget));
        assert!(retained.body().is_none() && retained.admitted_body().is_none());
        assert_eq!(retained.diagnostics().count(), 0);
        assert_eq!(arena.allocated_bytes(), before);
    }

    #[test]
    fn empty_and_comment_only_owners_preserve_their_actual_result() {
        let arena = Allocator::default();
        for text in ["", "/*nothing*/", "//tail"] {
            let retained = parse_once(&arena, embed(text, Shape::HandlerBody, Lang::Js))
                .into_handler_body()
                .unwrap();
            assert_eq!(retained.hole(), None);
            assert!(retained.body().unwrap().directives.is_empty());
            assert!(retained.body().unwrap().statements.is_empty());
            assert_eq!(retained.comments().count(), usize::from(!text.is_empty()));
        }
    }

    #[test]
    fn wrong_shapes_return_the_same_original_program_expression_and_parameters() {
        let arena = Allocator::default();
        for (text, shape) in [
            ("const count = 1; /*keep*/", Shape::Program),
            ("count + 1 /*keep*/", Shape::Expr),
            ("count = 1 /*keep*/", Shape::SlotParams),
        ] {
            let syntax = parse_once(&arena, embed(text, shape, Lang::Ts));
            let program = syntax.program().map(|p| p.body.as_ptr());
            let expression = syntax.expression().map(core::ptr::from_ref);
            let parameters = syntax.slot_params().map(|p| p.parameters().as_ptr());
            let comments = syntax.comments().next().unwrap().text().unwrap().as_ptr();
            let syntax = syntax.into_handler_body().unwrap_err();
            assert_eq!(syntax.program().map(|p| p.body.as_ptr()), program);
            assert_eq!(syntax.expression().map(core::ptr::from_ref), expression);
            assert_eq!(
                syntax.slot_params().map(|p| p.parameters().as_ptr()),
                parameters
            );
            assert_eq!(
                syntax.comments().next().unwrap().text().unwrap().as_ptr(),
                comments
            );
            assert_eq!(syntax.grammar().shape, shape);
            assert_eq!(syntax.hole(), None);
            assert_eq!(syntax.source().text(), text);
        }
    }
}

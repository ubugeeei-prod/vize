use oxc_span::SourceType;
use vize_l0::{Allocator, Span};

use super::super::{Embed, EmbedHole, EmbedSource, Grammar, Lang, Shape, parse_once};

fn embed(text: &str, shape: Shape, lang: Lang) -> Embed<'_> {
    Embed {
        grammar: Grammar { shape, lang },
        source: EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap(),
    }
}

#[test]
fn admission_borrows_the_same_formals_and_original_explicit_profile() {
    let allocator = Allocator::default();
    for lang in [Lang::Js, Lang::Ts] {
        let syntax = parse_once(
            &allocator,
            embed("item, index = 0 /*tail*/", Shape::SlotParams, lang),
        );
        let original = syntax.slot_params().unwrap().parameters().as_ptr();
        let comments = syntax.embedding.as_ref().unwrap().comments().as_ptr();
        let retained = syntax.into_slot_params().unwrap();
        let admitted = retained.admitted_parameters().unwrap();
        assert_eq!(admitted.parameters().items.as_ptr(), original);
        assert_eq!(admitted.content(), retained.source().text());
        assert_eq!(
            admitted.parser_content_span().start,
            retained.parser_prefix()
        );
        assert_eq!(admitted.source_type(), retained.source_type());
        assert!(admitted.source_type().is_module());
        assert_eq!(admitted.source_type().is_typescript(), lang == Lang::Ts);
        assert_eq!(
            retained.observation.as_ref().unwrap().comments().as_ptr(),
            comments
        );
        assert_eq!(
            retained.comments().next().unwrap().text().unwrap(),
            "/*tail*/"
        );
    }
}

#[test]
fn strict_semantic_formals_retain_full_context_refusal_and_authored_observations() {
    let allocator = Allocator::default();
    for lang in [Lang::Js, Lang::Ts] {
        for spelling in ["arguments", "eval", "yield", "\\u0065val", "\\u0079ield"] {
            let authored = vize_l0::cstr!("zz{} /*kept*/zz", spelling);
            let source =
                EmbedSource::authored(&authored, Span::new(2, authored.len() as u32 - 2)).unwrap();
            let syntax = parse_once(
                &allocator,
                Embed {
                    grammar: Grammar {
                        shape: Shape::SlotParams,
                        lang,
                    },
                    source,
                },
            );
            let comment = syntax.comments().next().unwrap().text().unwrap().as_ptr();
            let retained = syntax.into_slot_params().unwrap();
            assert_eq!(retained.hole(), Some(EmbedHole::InvalidParameterContext));
            assert!(retained.admitted_parameters().is_none());
            assert!(retained.parameters().is_none() && retained.rest().is_none());
            assert_eq!(retained.diagnostics().count(), 0);
            assert_eq!(retained.source().span(), source.span());
            assert_eq!(
                retained.comments().next().unwrap().text().unwrap().as_ptr(),
                comment
            );
            let start = 3 + spelling.len() as u32;
            assert_eq!(
                retained.comments().next().unwrap().authored_span(),
                Ok(Span::new(start, start + 8))
            );
        }
    }
}

#[test]
fn rejected_program_preserves_its_original_admission_ast_comments_and_profile() {
    let allocator = Allocator::default();
    let syntax = parse_once(
        &allocator,
        embed("/*original*/let value=1", Shape::Program, Lang::Ts),
    );
    // The normally owned Program header moves with the intact boxed artifact;
    // its original arena statement buffer and descendant nodes stay identical.
    let original = syntax.program().unwrap().body.as_ptr();
    let comment = syntax.comments().next().unwrap().text().unwrap().as_ptr();
    let syntax = syntax.into_slot_params().unwrap_err();
    assert_eq!(syntax.program().unwrap().body.as_ptr(), original);
    assert!(syntax.admitted_program().is_some());
    assert_eq!(syntax.source_type(), SourceType::ts().with_module(true));
    assert_eq!(
        syntax.comments().next().unwrap().text().unwrap().as_ptr(),
        comment
    );
    assert_eq!(syntax.diagnostics().count(), 0);
}

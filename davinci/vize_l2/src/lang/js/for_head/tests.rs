use super::{ForHeadInput, NativeForInput};
use oxc_ast::ast::BindingPattern;
use oxc_span::GetSpan;
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::syntax::{NativeForHead, NativeForInput as SourceForInput, NativeForRefusal};
use vize_l1::embed::{Embed, EmbedSource, Grammar, Lang, Shape, SourceError};

fn observe<'a>(arena: &'a Allocator, source: &'a str, lang: Lang) -> NativeForHead<'a> {
    SourceForInput::attribute_in(
        arena,
        SourceRoot::new(source).unwrap().whole_block(),
        source,
        lang,
    )
    .unwrap()
    .observe()
}

#[test]
fn complete_two_observations_profiles_and_roots_survive_movement() {
    let arena = Allocator::default();
    for lang in [Lang::Js, Lang::Ts] {
        let text = "(α, key) in β";
        let syntax = observe(&arena, text, lang);
        let aliases = syntax
            .aliases()
            .unwrap()
            .unwrap()
            .parameters()
            .unwrap()
            .as_ptr();
        let collection = syntax.collection().unwrap().unwrap().expression().unwrap() as *const _;
        let input = ForHeadInput::new(syntax).unwrap();
        assert!(core::mem::needs_drop::<ForHeadInput<'_>>());
        let mut pending = alloc::vec::Vec::new();
        pending.push(input);
        pending.reserve(32);
        let input = pending.pop().unwrap();
        assert_eq!(input.aliases().as_ptr(), aliases);
        assert_eq!(input.collection() as *const _, collection);
        assert!(core::ptr::eq(input.source().text(), text));
        assert_eq!(
            input.syntax().grammar(),
            Grammar {
                lang,
                shape: Shape::ForHead
            }
        );
        let admitted = input.syntax().admitted_dense().unwrap();
        for profile in [
            admitted.aliases().source_type(),
            admitted.collection().source_type(),
        ] {
            assert!(profile.is_module());
            assert!(!profile.is_jsx());
            assert_eq!(profile.is_typescript(), lang == Lang::Ts);
        }
        let aliases = input.aliases();
        let collection = input.collection();
        drop(input);
        assert_eq!(aliases.len(), 2);
        assert!(matches!(
            collection,
            oxc_ast::ast::Expression::Identifier(_)
        ));
    }
}

#[test]
fn unicode_alias_and_collection_namespaces_have_independent_exact_goldens() {
    let arena = Allocator::default();
    let file = "頭(α, key) in β尾";
    let value = file.get(3..file.len() - 3).unwrap();
    let syntax = SourceForInput::attribute_in(
        &arena,
        SourceRoot::new(file).unwrap().whole_block(),
        value,
        Lang::Ts,
    )
    .unwrap()
    .observe();
    let input = ForHeadInput::new(syntax).unwrap();
    let BindingPattern::BindingIdentifier(first) = &input.aliases()[0].pattern else {
        panic!("original alias")
    };
    let BindingPattern::BindingIdentifier(second) = &input.aliases()[1].pattern else {
        panic!("original alias")
    };
    assert_eq!(input.alias_decoded_span(first.span), Ok(Span::new(0, 2)));
    assert_eq!(input.alias_decoded_span(second.span), Ok(Span::new(4, 7)));
    assert_eq!(input.alias_authored_span(first.span), Ok(Span::new(4, 6)));
    assert_eq!(input.alias_authored_span(second.span), Ok(Span::new(8, 11)));
    assert_eq!(
        input.collection_decoded_span(input.collection().span()),
        Ok(Span::new(0, 2))
    );
    assert_eq!(
        input.collection_authored_span(input.collection().span()),
        Ok(Span::new(16, 18))
    );
    assert_eq!(
        input.alias_decoded_span(oxc_span::Span::new(0, 1)),
        Err(SourceError::InvalidDecodedSpan)
    );
    assert_eq!(
        input.collection_decoded_span(oxc_span::Span::new(0, 1)),
        Err(SourceError::InvalidDecodedSpan)
    );
    assert_eq!(
        input.collection_decoded_span(oxc_span::Span::new(3, 2)),
        Err(SourceError::InvalidDecodedSpan)
    );
    assert_eq!(
        input.alias_authored_span(oxc_span::Span::new(1, 2)),
        Err(SourceError::InvalidDecodedSpan)
    );
}

#[test]
fn neutral_dense_roots_do_not_establish_the_original_source_capability() {
    let arena = Allocator::default();
    let text = "item in items";
    let syntax = vize_l1::embed::syntax::parse_vue_for_head_once(
        &arena,
        Embed {
            grammar: Grammar {
                lang: Lang::Js,
                shape: Shape::ForHead,
            },
            source: EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap(),
        },
    );
    assert!(syntax.dense().is_some());
    let aliases = syntax
        .aliases()
        .unwrap()
        .unwrap()
        .parameters()
        .unwrap()
        .as_ptr();
    let rejected = ForHeadInput::new(syntax).unwrap_err();
    assert_eq!(rejected.kind, NativeForRefusal::UnpreparedSource);
    assert_eq!(
        rejected
            .syntax()
            .aliases()
            .unwrap()
            .unwrap()
            .parameters()
            .unwrap()
            .as_ptr(),
        aliases
    );
    assert!(core::ptr::eq(rejected.into_syntax().source().text(), text));
}

mod refusal;
mod selected;

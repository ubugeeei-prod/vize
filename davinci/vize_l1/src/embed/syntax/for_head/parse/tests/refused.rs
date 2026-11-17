use super::{Allocator, ForRefusal, Lang, observe};
use crate::embed::syntax::{EmbedHole, ForHeadHole};

#[test]
fn bounded_native_family_refuses_source_spelling_without_discarding_roots() {
    let arena = Allocator::default();
    for (text, expected) in [
        ("(a,b,c) in items", ForRefusal::AliasCount(3)),
        ("{item} in items", ForRefusal::AliasShape),
        ("item=1 in items", ForRefusal::AliasShape),
        ("item:Type in items", ForRefusal::AliasShape),
        ("item in items.list", ForRefusal::CollectionShape),
        ("item in (items)", ForRefusal::CollectionShape),
        ("item /*a*/ in items", ForRefusal::Comment),
        ("item in items /*c*/", ForRefusal::Comment),
        ("\\u0069tem in items", ForRefusal::EscapedSpelling),
        ("item in \\u0069tems", ForRefusal::EscapedSpelling),
        (" item in items", ForRefusal::OuterWhitespace),
        ("item in items\n", ForRefusal::OuterWhitespace),
        ("\u{feff}item in items", ForRefusal::OuterWhitespace),
        ("_ctx in items", ForRefusal::RenderContextName),
        ("item in _ctx", ForRefusal::RenderContextName),
        ("item in &#105;tems", ForRefusal::EntityOutput),
    ] {
        let head = observe(&arena, text, Lang::Ts);
        assert_eq!(head.for_refusal(), Some(expected), "{text}");
        assert!(head.admitted_dense().is_none(), "{text}");
        assert_eq!(head.grammar().lang, Lang::Ts);
        assert_eq!(head.source().span().end as usize, text.len());
        assert!(
            head.aliases()
                .unwrap()
                .unwrap()
                .admitted_parameters()
                .is_some()
        );
        assert!(
            head.collection()
                .unwrap()
                .unwrap()
                .admitted_expression()
                .is_some()
        );
    }
}

#[test]
fn strict_context_and_real_errors_keep_typed_holes_and_original_observations() {
    let arena = Allocator::default();
    for lang in [Lang::Js, Lang::Ts] {
        for text in [
            "arguments in items",
            "eval in items",
            "yield in items",
            "\\u0079ield in items",
        ] {
            let head = observe(&arena, text, lang);
            assert_eq!(
                head.for_refusal(),
                Some(ForRefusal::Head(ForHeadHole::AliasesUnavailable(
                    EmbedHole::InvalidParameterContext
                )))
            );
            assert!(head.admitted_dense().is_none());
            assert_eq!(head.source().text(), text);
            assert_eq!(head.diagnostics().count(), 0);
            assert!(
                head.collection()
                    .unwrap()
                    .unwrap()
                    .admitted_expression()
                    .is_some()
            );
        }
        let malformed = observe(&arena, "item in value + /*kept*/", lang);
        assert_eq!(
            malformed.for_refusal(),
            Some(ForRefusal::Head(ForHeadHole::CollectionUnavailable(
                EmbedHole::Syntax
            )))
        );
        assert!(malformed.diagnostics().count() > 0);
        assert_eq!(
            malformed.comments().next().unwrap().1.text().unwrap(),
            "/*kept*/"
        );
        assert_eq!(malformed.source().text(), "item in value + /*kept*/");
    }
}

#[test]
fn whole_head_and_sparse_refusals_do_not_manufacture_joint_parts() {
    let arena = Allocator::default();
    let long = vize_l0::cstr!("{} in items", ";".repeat(32));
    let head = observe(&arena, &long, Lang::Js);
    assert_eq!(
        head.for_refusal(),
        Some(ForRefusal::Head(ForHeadHole::WholeHeadAdmission(
            EmbedHole::TokenBudget
        )))
    );
    assert!(head.aliases().is_none() && head.collection().is_none());
    assert!(head.admitted_dense().is_none());
    assert_eq!(head.source().text(), long);
    for text in [
        "(...item) in items",
        "(,item) in items",
        "(item,) in items",
        "item in",
    ] {
        let head = observe(&arena, text, Lang::Js);
        assert!(matches!(head.for_refusal(), Some(ForRefusal::Head(_))));
        assert!(head.admitted_dense().is_none());
        assert_eq!(head.source().text(), text);
    }
}

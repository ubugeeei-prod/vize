use oxc_ast::ast::{BindingPattern, Expression};
use oxc_span::GetSpan;
use vize_l0::{Allocator, Span};

use super::super::{Embed, EmbedHole, EmbedSource, Grammar, Lang, Shape, parse_once};
use super::{ForHeadHole, ForHeadPart, ForKeyword, NativeForHead, parse_vue_for_head_once};
use crate::embed::prepare_attribute_value;

fn embed(text: &str, lang: Lang) -> Embed<'_> {
    Embed {
        grammar: Grammar {
            shape: Shape::ForHead,
            lang,
        },
        source: EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap(),
    }
}

fn parse<'a>(allocator: &'a Allocator, text: &'a str, lang: Lang) -> NativeForHead<'a> {
    parse_vue_for_head_once(allocator, embed(text, lang))
}

#[test]
fn retains_real_dense_bindings_and_collection_forms_in_js_and_ts() {
    let allocator = Allocator::default();
    for (input, lang, count) in [
        ("value in 10", Lang::Js, 1),
        ("(value,key) of 'abc'", Lang::Js, 2),
        ("(value,key,index) in [1,2]", Lang::Js, 3),
        ("value in new Map()", Lang::Js, 1),
        ("({value},key) in {a:1}", Lang::Js, 2),
        ("value in record", Lang::Js, 1),
        ("(value: T,key) in ({} as T)", Lang::Ts, 2),
    ] {
        let head = parse(&allocator, input, lang);
        assert_eq!(head.hole(), None, "{input}: {head:?}");
        let view = head.dense().unwrap();
        assert_eq!(view.parameters().len(), count);
        assert_eq!(
            head.aliases().unwrap().unwrap().grammar().shape,
            Shape::SlotParams
        );
        assert_eq!(
            head.collection().unwrap().unwrap().grammar().shape,
            Shape::Expr
        );
        assert_eq!(
            view.alias_prefix(),
            head.aliases().unwrap().unwrap().parser_prefix()
        );
        assert_eq!(
            view.collection_prefix(),
            head.collection().unwrap().unwrap().parser_prefix()
        );
        for parameter in view.parameters() {
            let authored = view.alias_authored_span(parameter.span).unwrap();
            assert!(
                !input
                    .get(authored.start as usize..authored.end as usize)
                    .unwrap()
                    .is_empty()
            );
        }
        let authored = view
            .collection_authored_span(view.collection().span())
            .unwrap();
        assert_eq!(
            input
                .get(authored.start as usize..authored.end as usize)
                .unwrap()
                .trim(),
            view.collection_source().text().trim()
        );
        assert!(view.alias_decoded_span(oxc_span::Span::new(0, 2)).is_err());
    }
    let head = parse(&allocator, "(value: T,key) in xs", Lang::Js);
    assert_eq!(
        head.hole(),
        Some(ForHeadHole::AliasesUnavailable(EmbedHole::Syntax))
    );
    assert!(head.dense().is_none());
    assert!(head.collection().unwrap().unwrap().expression().is_some());
}

#[test]
fn strict_first_separator_and_unicode_offsets_preserve_authored_roots() {
    let allocator = Allocator::default();
    let authored = "zz  ( α, β )\u{2003}of\u{2003}xs  zz";
    let source = EmbedSource::authored(authored, Span::new(2, authored.len() as u32 - 2)).unwrap();
    let head = parse_vue_for_head_once(
        &allocator,
        Embed {
            grammar: Grammar {
                shape: Shape::ForHead,
                lang: Lang::Js,
            },
            source,
        },
    );
    assert_eq!(head.hole(), None);
    let view = head.dense().unwrap();
    assert_eq!(view.alias_source().text(), "α, β");
    assert_eq!(view.collection_source().text(), "xs  ");
    let separator = head.separator().unwrap();
    assert_eq!(separator.0, ForKeyword::Of);
    let keyword = source.authored_span(separator.1).unwrap();
    assert_eq!(
        authored
            .get(keyword.start as usize..keyword.end as usize)
            .unwrap(),
        "of"
    );
    for (parameter, expected) in view.parameters().iter().zip(["α", "β"]) {
        let span = view.alias_authored_span(parameter.span).unwrap();
        assert_eq!(
            authored
                .get(span.start as usize..span.end as usize)
                .unwrap(),
            expected
        );
    }
    let head = parse(&allocator, "a in b in c", Lang::Js);
    assert_eq!(head.hole(), None);
    assert_eq!(head.dense().unwrap().collection_source().text(), "b in c");
    assert!(matches!(
        head.dense().unwrap().collection(),
        Expression::BinaryExpression(_)
    ));
    assert_eq!(head.separator(), Some((ForKeyword::In, Span::new(2, 4))));
    // The ordinary one-Program API still refuses composite shapes honestly.
    assert_eq!(
        parse_once(&allocator, embed("a in xs", Lang::Js)).hole(),
        Some(EmbedHole::UnsupportedShape)
    );
}

#[test]
fn decoded_parts_keep_exact_entity_atoms_and_live_after_owner_drop() {
    let allocator = Allocator::default();
    let authored = "zz(&fjlig;,k) in &fjlig;zz";
    let source = prepare_attribute_value(
        &allocator,
        authored,
        Span::new(2, authored.len() as u32 - 2),
    )
    .unwrap();
    let view = {
        let head = parse_vue_for_head_once(
            &allocator,
            Embed {
                grammar: Grammar {
                    shape: Shape::ForHead,
                    lang: Lang::Js,
                },
                source,
            },
        );
        assert_eq!(head.hole(), None);
        assert_eq!(
            head.source().decode_map().unwrap().segments().as_ptr(),
            source.decode_map().unwrap().segments().as_ptr()
        );
        let aliases = head.aliases().unwrap().unwrap();
        let collection = head.collection().unwrap().unwrap();
        let view = head.dense().unwrap();
        assert_eq!(
            view.parameters().as_ptr(),
            aliases.parameters().unwrap().as_ptr()
        );
        assert!(core::ptr::eq(
            view.collection(),
            collection.expression().unwrap()
        ));
        assert_eq!(view.alias_source().text(), "fj,k");
        assert_eq!(view.collection_source().text(), "fj");
        assert_eq!(
            view.alias_authored_span(view.parameters()[0].span),
            Ok(Span::new(3, 10))
        );
        assert_eq!(
            view.collection_authored_span(view.collection().span()),
            Ok(Span::new(17, 24))
        );
        assert!(view.alias_source().authored_span(Span::new(0, 1)).is_err());
        drop(head);
        view
    };
    let BindingPattern::BindingIdentifier(identifier) = &view.parameters()[0].pattern else {
        panic!("expected original binding")
    };
    assert_eq!(identifier.name.as_str(), "fj");
    let Expression::Identifier(identifier) = view.collection() else {
        panic!("expected original collection root")
    };
    assert_eq!(identifier.name.as_str(), "fj");
    assert_eq!(
        view.collection_authored_span(identifier.span),
        Ok(Span::new(17, 24))
    );
}

#[test]
fn actual_ast_positions_keep_commas_inside_all_lexical_binding_forms() {
    let allocator = Allocator::default();
    for (input, lang, roots) in [
        ("({a,b},k) in xs", Lang::Js, 2),
        ("([a,b],k) in xs", Lang::Js, 2),
        ("(x='a,b',k) in xs", Lang::Js, 2),
        ("(x=\"a,b\",k) in xs", Lang::Js, 2),
        ("(x=/a,b/,k) in xs", Lang::Js, 2),
        ("(x=`a,b`,k) in xs", Lang::Js, 2),
        ("(x: Map<A,B>,k) in xs", Lang::Ts, 2),
        ("(x/*,*/,k) in xs", Lang::Js, 2),
        ("(x,k/*,*/) in xs", Lang::Js, 2),
    ] {
        let head = parse(&allocator, input, lang);
        assert_eq!(head.hole(), None, "{input}: {head:?}");
        let view = head.dense().unwrap();
        assert_eq!(view.parameters().len(), roots);
        let last = view
            .alias_authored_span(view.parameters().last().unwrap().span)
            .unwrap();
        assert_eq!(
            input.get(last.start as usize..last.end as usize).unwrap(),
            "k"
        );
    }
}

#[test]
fn sparse_extra_rest_and_trailing_comma_forms_keep_source_and_both_owners() {
    let allocator = Allocator::default();
    for (input, hole) in [
        (" in xs", ForHeadHole::EmptyAliases),
        ("() in xs", ForHeadHole::EmptyAliases),
        ("(a,b,c,d) in xs", ForHeadHole::ExtraAliases(4)),
        ("(a,...rest) in xs", ForHeadHole::RestAlias),
        ("(a,) in xs", ForHeadHole::TrailingAliasSyntax),
        ("(a,b,) in xs", ForHeadHole::TrailingAliasSyntax),
        ("(a,/*,*/) in xs", ForHeadHole::TrailingAliasSyntax),
        ("(a/*,*/,) in xs", ForHeadHole::TrailingAliasSyntax),
        (
            "(a,,c) in xs",
            ForHeadHole::AliasesUnavailable(EmbedHole::Syntax),
        ),
        (
            "(,b,c) in xs",
            ForHeadHole::AliasesUnavailable(EmbedHole::Syntax),
        ),
    ] {
        let head = parse(&allocator, input, Lang::Js);
        assert_eq!(head.hole(), Some(hole), "{input}: {head:?}");
        assert_eq!(head.source().text(), input);
        assert!(head.dense().is_none());
        assert!(head.aliases().is_some() && head.collection().is_some());
        assert_eq!(head.collection().unwrap().unwrap().source().text(), "xs");
        assert!(head.collection().unwrap().unwrap().expression().is_some());
    }
}

#[test]
fn failed_parts_keep_complete_role_labelled_diagnostics_and_comments() {
    let allocator = Allocator::default();
    let input = "x=/*a*/ in +/*b*/";
    let head = parse(&allocator, input, Lang::Js);
    assert!(head.dense().is_none());
    let aliases = head.aliases().unwrap().unwrap();
    let collection = head.collection().unwrap().unwrap();
    assert_eq!(aliases.hole(), Some(EmbedHole::Syntax));
    assert_eq!(collection.hole(), Some(EmbedHole::Syntax));
    assert_eq!(
        head.diagnostics().count(),
        aliases.diagnostics().count() + collection.diagnostics().count()
    );
    for (part, diagnostic) in head.diagnostics() {
        let source = match part {
            ForHeadPart::Aliases => aliases.source(),
            ForHeadPart::Collection => collection.source(),
        };
        let same = |original: super::super::DiagnosticView<'_, '_>| {
            core::ptr::eq(original.message().as_ptr(), diagnostic.message().as_ptr())
        };
        assert!(match part {
            ForHeadPart::Aliases => aliases.diagnostics().any(same),
            ForHeadPart::Collection => collection.diagnostics().any(same),
        });
        for label in diagnostic.labels() {
            let decoded = label.decoded_span().unwrap();
            assert!(
                source
                    .text()
                    .get(decoded.start as usize..decoded.end as usize)
                    .is_some()
            );
            let authored = label.authored_span().unwrap();
            assert!(source.span().start <= authored.start && authored.end <= source.span().end);
        }
    }
    let mut comments = head.comments();
    let (part, comment) = comments.next().unwrap();
    assert_eq!(
        (part, comment.text().unwrap()),
        (ForHeadPart::Aliases, "/*a*/")
    );
    assert_eq!(comment.authored_span(), Ok(Span::new(2, 7)));
    let (part, comment) = comments.next().unwrap();
    assert_eq!(
        (part, comment.text().unwrap()),
        (ForHeadPart::Collection, "/*b*/")
    );
    assert_eq!(comment.authored_span(), Ok(Span::new(12, 17)));
    assert!(comments.next().is_none());
}

mod boundaries;

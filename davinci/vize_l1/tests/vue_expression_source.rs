//! Real checked preparation and the sole retained parser, without a backend.

use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use vize_l0::{Allocator, Span};
use vize_l1::embed::{
    DecodeSegmentKind, Embed, Grammar, Lang, Shape, SourceError, prepare_attribute_value,
    prepare_vue_interpolation_in,
    syntax::{NativeSyntax, parse_once},
};

#[cfg(test)]
fn content(file: &str) -> Span {
    // These fixtures have known Unicode prefix/suffix and real authored bytes.
    Span::new("前{{".len() as u32, (file.len() - "}}後".len()) as u32)
}

#[cfg(test)]
fn syntax<'a>(allocator: &'a Allocator, file: &'a str) -> NativeSyntax<'a> {
    let source = prepare_vue_interpolation_in(allocator, file, content(file)).unwrap();
    parse_once(
        allocator,
        Embed {
            grammar: Grammar {
                shape: Shape::Expr,
                lang: Lang::Js,
            },
            source,
        },
    )
}

#[test]
fn only_five_authored_html_whitespace_bytes_are_removed_without_allocation() {
    let allocator = Allocator::default();
    let file = "前{{ \t\n\x0c\r 作者 \r\x0c\n\t }}後";
    let source = prepare_vue_interpolation_in(&allocator, file, content(file)).unwrap();
    assert_eq!(source.text(), "作者");
    assert_eq!(source.span(), Span::new(11, 17));
    assert_eq!(file.get(11..17), Some(source.text()));
    assert_eq!(file.get(11..).unwrap().as_ptr(), source.text().as_ptr());
    assert!(source.decode_map().is_none());
    assert_eq!(allocator.allocated_bytes(), 0);
    assert_eq!(file, "前{{ \t\n\x0c\r 作者 \r\x0c\n\t }}後");
}

#[test]
fn non_html_whitespace_is_preserved_as_source_without_claiming_backend_admission() {
    let allocator = Allocator::default();
    for raw in ["\u{a0}msg\u{a0}", "\u{feff}msg\u{feff}", "\u{b}msg\u{b}"] {
        let file = vize_l0::cstr!("前{{{{ {raw} }}}}後");
        let source = prepare_vue_interpolation_in(&allocator, &file, content(&file)).unwrap();
        assert_eq!(source.text(), raw);
        assert_eq!(source.span().start, content(&file).start + 1);
        assert_eq!(source.span().end, content(&file).end - 1);
        assert_eq!(
            file.get(source.span().start as usize..source.span().end as usize),
            Some(raw)
        );
        assert!(source.decode_map().is_none());
    }
    assert_eq!(allocator.allocated_bytes(), 0);
    // Vue's NBSP/BOM simple-name spelling needs a distinct typed backend refusal.
}

#[test]
fn text_context_and_attribute_context_have_distinct_decoding_and_edge_policies() {
    let allocator = Allocator::default();
    let file = "前{{ \"&copycat\" + \"&amp=1\" }}後";
    let text = prepare_vue_interpolation_in(&allocator, file, content(file)).unwrap();
    let attribute = prepare_attribute_value(&allocator, file, content(file)).unwrap();
    assert_eq!(text.text(), "\"©cat\" + \"&=1\"");
    assert_eq!(attribute.text(), " \"&copycat\" + \"&amp=1\" ");
    assert_eq!(attribute.span(), content(file));
    assert_eq!(text.span().start, content(file).start + 1);
    assert_eq!(text.span().end, content(file).end - 1);
    assert!(text.decode_map().is_some());
    assert!(attribute.decode_map().is_none());
}

#[test]
fn entities_are_decoded_once_after_edge_selection_and_unknown_references_stay_borrowed() {
    let allocator = Allocator::default();
    for raw in ["msg", "'&unknown;'", "'&#x;'", "'a&b'"] {
        let file = vize_l0::cstr!("前{{{{ {raw} }}}}後");
        let source = prepare_vue_interpolation_in(&allocator, &file, content(&file)).unwrap();
        assert_eq!(source.text(), raw);
        assert!(source.decode_map().is_none());
    }
    assert_eq!(allocator.allocated_bytes(), 0);
    let file = "前{{ '&amp;amp;' }}後";
    let source = prepare_vue_interpolation_in(&allocator, file, content(file)).unwrap();
    assert_eq!(source.text(), "'&amp;'");
    assert!(source.decode_map().is_some());
}

#[test]
fn entity_whitespace_survives_and_real_ast_endpoints_project_to_complete_authored_bytes() {
    let allocator = Allocator::default();
    let file = "前{{ &#32;作者 &amp;&amp; value&#32; }}後";
    let syntax = syntax(&allocator, file);
    assert_eq!(syntax.hole(), None);
    let source = syntax.source();
    assert_eq!(source.text(), " 作者 && value ");
    assert_eq!(source.span().start, content(file).start + 1);
    assert_eq!(source.span().end, content(file).end - 1);
    assert_eq!(
        source.authored_span(Span::new(0, source.text().len() as u32)),
        Ok(source.span())
    );
    let ast = syntax.expression().unwrap();
    let root = syntax.decoded_span(ast.span()).unwrap();
    assert_eq!(
        source.text().get(root.start as usize..root.end as usize),
        Some("作者 && value")
    );
    let authored = syntax.authored_span(ast.span()).unwrap();
    assert_eq!(
        file.get(authored.start as usize..authored.end as usize),
        Some("作者 &amp;&amp; value")
    );
    let Expression::LogicalExpression(logical) = ast else {
        panic!("expected the actual logical expression")
    };
    let ast = core::ptr::from_ref(&**logical);
    let retained = syntax.into_expression(&allocator).unwrap();
    let Expression::LogicalExpression(logical) = retained.expression().unwrap() else {
        panic!("expected the same retained logical payload")
    };
    assert_eq!(ast, core::ptr::from_ref(&**logical));
    assert_eq!(retained.source().text(), source.text());
    assert_eq!(retained.source().span(), source.span());
}

#[test]
fn a_trimmed_window_keeps_complete_multi_scalar_entities_and_rejects_interior_edits() {
    let allocator = Allocator::default();
    let file = "前{{ '&acE;&fjlig;' /*k*/ }}後";
    let syntax = syntax(&allocator, file);
    assert_eq!(syntax.hole(), None);
    let source = syntax.source();
    assert_eq!(source.text(), "'\u{223e}\u{333}fj' /*k*/");
    let map = source.decode_map().unwrap();
    let entities: Vec<_> = map
        .segments()
        .iter()
        .filter(|segment| segment.kind() == DecodeSegmentKind::Entity)
        .collect();
    assert_eq!(entities.len(), 2);
    for (entity, authored_text) in entities.iter().zip(["&acE;", "&fjlig;"]) {
        assert_eq!(
            source.authored_span(entity.decoded()),
            Ok(entity.authored())
        );
        assert_eq!(
            file.get(entity.authored().start as usize..entity.authored().end as usize),
            Some(authored_text)
        );
    }
    let first = entities.first().unwrap();
    assert_eq!(first.decoded(), Span::new(1, 6));
    assert_eq!(
        first.decoded().end - first.decoded().start,
        first.authored().end - first.authored().start
    );
    assert_eq!(
        source.authored_span(Span::new(1, 4)),
        Err(SourceError::PartialEntityBoundary)
    );
    assert_eq!(
        source.authored_span(Span::new(2, 4)),
        Err(SourceError::InvalidDecodedSpan)
    );
    let comment = syntax.comments().next().unwrap();
    assert_eq!(comment.text().unwrap(), "/*k*/");
    let authored = comment.authored_span().unwrap();
    assert_eq!(
        file.get(authored.start as usize..authored.end as usize),
        Some("/*k*/")
    );
}

#[test]
fn retained_root_and_all_leading_trailing_comments_survive_without_minimum_envelopes() {
    let allocator = Allocator::default();
    for (file, expected, comments) in [
        ("前{{ msg /*kept*/ }}後", "msg /*kept*/", vec!["/*kept*/"]),
        (
            "前{{ /*before*/ msg /*after*/ }}後",
            "/*before*/ msg /*after*/",
            vec!["/*before*/", "/*after*/"],
        ),
        (
            "前{{ /*🌸*/ 作者 /*後*/ }}後",
            "/*🌸*/ 作者 /*後*/",
            vec!["/*🌸*/", "/*後*/"],
        ),
    ] {
        let syntax = syntax(&allocator, file);
        assert_eq!(syntax.hole(), None);
        assert_eq!(syntax.source().text(), expected);
        let Expression::Identifier(identifier) = syntax.expression().unwrap() else {
            panic!("expected the actual identifier root")
        };
        let root = core::ptr::from_ref(&**identifier);
        let before: Vec<_> = syntax
            .comments()
            .map(|comment| {
                let decoded = comment.decoded_span().unwrap();
                let authored = comment.authored_span().unwrap();
                let text = comment.text().unwrap();
                assert_eq!(
                    syntax
                        .source()
                        .text()
                        .get(decoded.start as usize..decoded.end as usize),
                    Some(text)
                );
                assert_eq!(
                    file.get(authored.start as usize..authored.end as usize),
                    Some(text)
                );
                (text, text.as_ptr(), decoded, authored)
            })
            .collect();
        assert_eq!(
            before
                .iter()
                .map(|&(text, _, _, _)| text)
                .collect::<Vec<_>>(),
            comments
        );
        let retained = syntax.into_expression(&allocator).unwrap();
        let Expression::Identifier(identifier) = retained.expression().unwrap() else {
            panic!("expected the same retained identifier payload")
        };
        assert_eq!(root, core::ptr::from_ref(&**identifier));
        let after: Vec<_> = retained
            .comments()
            .map(|comment| {
                let text = comment.text().unwrap();
                (
                    text,
                    text.as_ptr(),
                    comment.decoded_span().unwrap(),
                    comment.authored_span().unwrap(),
                )
            })
            .collect();
        assert_eq!(before, after);
        assert_eq!(retained.source().text(), expected);
    }
}

#[test]
fn line_comment_newline_is_source_data_and_never_inferred_from_the_private_wrapper() {
    let allocator = Allocator::default();
    for (file, expected, expected_tail) in [
        ("前{{ x + //inside\n y }}後", "x + //inside\n y", "\n y"),
        ("前{{ //before\n x }}後", "//before\n x", "\n x"),
        ("前{{ x //after\n }}後", "x //after", ""),
        ("前{{ x //after&#10; }}後", "x //after\n", "\n"),
    ] {
        let syntax = syntax(&allocator, file);
        assert_eq!(syntax.hole(), None);
        assert_eq!(syntax.source().text(), expected);
        let comment = syntax.comments().next().unwrap();
        let decoded = comment.decoded_span().unwrap();
        let tail = syntax.source().text().get(decoded.end as usize..).unwrap();
        assert_eq!(tail, expected_tail);
        let authored = comment.authored_span().unwrap();
        assert_eq!(
            file.get(authored.start as usize..authored.end as usize),
            Some(comment.text().unwrap())
        );
    }
    // L1's private wrapper can terminate a comment, but is never emitted.
    // Outer // stays refused by the unchanged real L2 admission guard.
}

#[test]
fn invalid_utf8_ranges_are_refused_and_an_empty_window_is_only_source_not_an_expression() {
    let allocator = Allocator::default();
    for span in [Span::new(1, 2), Span::new(0, 4), Span::new(3, 2)] {
        assert_eq!(
            prepare_vue_interpolation_in(&allocator, "éa", span).unwrap_err(),
            SourceError::InvalidAuthoredSpan
        );
    }
    assert_eq!(allocator.allocated_bytes(), 0);
    let file = "前{{ \t\n\x0c\r }}後";
    let source = prepare_vue_interpolation_in(&allocator, file, content(file)).unwrap();
    assert_eq!(source.text(), "");
    assert_eq!(
        source.span(),
        Span::new(content(file).end, content(file).end)
    );
    let syntax = syntax(&allocator, file);
    assert!(syntax.hole().is_some());
    assert!(syntax.expression().is_none());
}

use super::{assert_original, observe, selected};
use crate::embed::DecodeSegmentKind;
use alloc::vec::Vec;
use oxc_ast::ast::{CommentKind, Expression};
use vize_l0::{Allocator, Span};

#[test]
fn complete_entity_maps_physical_lf_crlf_and_original_comments_keep_exact_source_geometry() {
    let arena = Allocator::default();
    for (source, raw, decoded, value, shift) in [
        (
            "<template><p :id='/*雪*/\n(α&amp;&amp;β)//尾\n'/></template>",
            "/*雪*/\n(α&amp;&amp;β)//尾\n",
            "/*雪*/\n(α&&β)//尾\n",
            Span::new(18, 48),
            0,
        ),
        (
            "<template><p :id='/*雪*/\r\n(α&amp;&amp;β)//尾\r\n'/></template>",
            "/*雪*/\r\n(α&amp;&amp;β)//尾\r\n",
            "/*雪*/\r\n(α&&β)//尾\r\n",
            Span::new(18, 50),
            1,
        ),
    ] {
        let owner = selected(&arena, source);
        let operand = observe(&owner, 0);
        assert_eq!(operand.name_span(), Span::new(13, 16));
        assert_eq!(operand.argument_span(), Span::new(14, 16));
        assert_eq!(operand.value_span(), value);
        assert_eq!(operand.raw_value(), raw);
        assert_eq!(value.slice(source), raw);
        let syntax = operand.syntax();
        assert_eq!(syntax.source().span(), value);
        assert_eq!(syntax.source().text(), decoded);
        assert!(core::ptr::eq(syntax.source().authored_root(), source));
        assert_eq!(syntax.parser_prefix(), 2);
        assert_eq!(syntax.hole(), None);
        assert_eq!(syntax.diagnostics().count(), 0);
        let map: Vec<_> = syntax
            .source()
            .decode_map()
            .unwrap()
            .segments()
            .iter()
            .map(|segment| (segment.decoded(), segment.authored(), segment.kind()))
            .collect();
        assert_eq!(
            map,
            [
                (
                    Span::new(0, 11 + shift),
                    Span::new(18, 29 + shift),
                    DecodeSegmentKind::Identity
                ),
                (
                    Span::new(11 + shift, 12 + shift),
                    Span::new(29 + shift, 34 + shift),
                    DecodeSegmentKind::Entity
                ),
                (
                    Span::new(12 + shift, 13 + shift),
                    Span::new(34 + shift, 39 + shift),
                    DecodeSegmentKind::Entity
                ),
                (
                    Span::new(13 + shift, 22 + 2 * shift),
                    Span::new(39 + shift, 48 + 2 * shift),
                    DecodeSegmentKind::Identity
                ),
            ]
        );
        let comments: Vec<_> = syntax
            .comments()
            .map(|comment| {
                (
                    comment.kind(),
                    comment.decoded_span().unwrap(),
                    comment.authored_span().unwrap(),
                    comment.text().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            comments,
            [
                (
                    CommentKind::SingleLineBlock,
                    Span::new(0, 7),
                    Span::new(18, 25),
                    "/*雪*/"
                ),
                (
                    CommentKind::Line,
                    Span::new(16 + shift, 21 + shift),
                    Span::new(42 + shift, 47 + shift),
                    "//尾"
                ),
            ]
        );
        let Expression::ParenthesizedExpression(parentheses) = syntax.expression().unwrap() else {
            panic!("original authored parentheses")
        };
        assert_eq!(
            parentheses.span,
            oxc_span::Span::new(10 + shift, 18 + shift)
        );
        assert_eq!(
            syntax.decoded_span(parentheses.span),
            Ok(Span::new(8 + shift, 16 + shift))
        );
        assert_eq!(
            syntax.authored_span(parentheses.span),
            Ok(Span::new(26 + shift, 42 + shift))
        );
        let Expression::LogicalExpression(logical) = &parentheses.expression else {
            panic!("once-decoded original logical expression")
        };
        assert_eq!(logical.operator.as_str(), "&&");
        assert_eq!(logical.span, oxc_span::Span::new(11 + shift, 17 + shift));
        for (child, name, stock, authored) in [
            (
                &logical.left,
                "α",
                oxc_span::Span::new(11 + shift, 13 + shift),
                Span::new(27 + shift, 29 + shift),
            ),
            (
                &logical.right,
                "β",
                oxc_span::Span::new(15 + shift, 17 + shift),
                Span::new(39 + shift, 41 + shift),
            ),
        ] {
            let Expression::Identifier(identifier) = child else {
                panic!("original Unicode child")
            };
            assert_eq!(identifier.name.as_str(), name);
            assert_eq!(identifier.span, stock);
            assert_eq!(syntax.authored_span(identifier.span), Ok(authored));
        }
        assert_original(&operand, &owner, 0);
    }
}

#[test]
fn attribute_context_ambiguous_entities_stay_literal_and_decoded_output_is_not_redecoded() {
    let arena = Allocator::default();
    let source = "<template><p :id='\"&amp=1 &notit; &amp;lt;\"'/></template>";
    let owner = selected(&arena, source);
    let operand = observe(&owner, 0);
    assert_eq!(operand.name_span(), Span::new(13, 16));
    assert_eq!(operand.argument_span(), Span::new(14, 16));
    assert_eq!(operand.value_span(), Span::new(18, 43));
    assert_eq!(operand.raw_value(), "\"&amp=1 &notit; &amp;lt;\"");
    let syntax = operand.syntax();
    assert_eq!(syntax.source().text(), "\"&amp=1 &notit; &lt;\"");
    assert_eq!(syntax.source().span(), Span::new(18, 43));
    assert_eq!(syntax.hole(), None);
    assert_eq!(syntax.comments().count(), 0);
    assert_eq!(syntax.diagnostics().count(), 0);
    let map: Vec<_> = syntax
        .source()
        .decode_map()
        .unwrap()
        .segments()
        .iter()
        .map(|segment| (segment.decoded(), segment.authored(), segment.kind()))
        .collect();
    assert_eq!(
        map,
        [
            (
                Span::new(0, 16),
                Span::new(18, 34),
                DecodeSegmentKind::Identity
            ),
            (
                Span::new(16, 17),
                Span::new(34, 39),
                DecodeSegmentKind::Entity
            ),
            (
                Span::new(17, 21),
                Span::new(39, 43),
                DecodeSegmentKind::Identity
            ),
        ]
    );
    let Expression::StringLiteral(literal) = syntax.expression().unwrap() else {
        panic!("original complete string literal")
    };
    assert_eq!(literal.value.as_str(), "&amp=1 &notit; &lt;");
    assert_eq!(
        literal.raw.as_ref().unwrap().as_str(),
        "\"&amp=1 &notit; &lt;\""
    );
    assert_eq!(literal.span, oxc_span::Span::new(2, 23));
    assert_eq!(syntax.decoded_span(literal.span), Ok(Span::new(0, 21)));
    assert_eq!(syntax.authored_span(literal.span), Ok(Span::new(18, 43)));
    assert_original(&operand, &owner, 0);
}

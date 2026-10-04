use super::{assert_output, options};
use oxc_ast::ast::{CommentKind, Expression};
use vize_glyph::native_doc::LineEnding;
use vize_l0::{Allocator, Span};
use vize_l1::embed::DecodeSegmentKind;

#[test]
fn complete_physical_lf_crlf_comments_and_entity_maps_survive_both_generated_endings() {
    for (source, raw, decoded, physical, value, shift) in [
        (
            "<template><p :id='/*雪*/\n(α&amp;&amp;β)//尾\n'>{{1n}}</p></template>",
            "/*雪*/\n(α&amp;&amp;β)//尾\n",
            "/*雪*/\n(α&&β)//尾\n",
            "\n",
            Span::new(18, 48),
            0,
        ),
        (
            "<template><p :id='/*雪*/\r\n(α&amp;&amp;β)//尾\r\n'>{{1n}}</p></template>",
            "/*雪*/\r\n(α&amp;&amp;β)//尾\r\n",
            "/*雪*/\r\n(α&&β)//尾\r\n",
            "\r\n",
            Span::new(18, 50),
            1,
        ),
    ] {
        for (ending, generated) in [(LineEnding::Lf, "\n"), (LineEnding::CrLf, "\r\n")] {
            let expected = format!(
                "<template><p{generated}  :id='/*雪*/{physical}(α &amp;&amp; β)//尾{physical}'{generated}>{{{{ 1n }}}}</p></template>"
            );
            let arena = Allocator::default();
            let owner = assert_output(&arena, source, &expected, options(200, 2, ending));
            assert_eq!(owner.binding_operands().len(), 1);
            assert_eq!(owner.operands().len(), 1);
            let operand = &owner.binding_operands()[0];
            assert_eq!(operand.name_span(), Span::new(13, 16));
            assert_eq!(operand.argument_span(), Span::new(14, 16));
            assert_eq!(operand.value_span(), value);
            assert_eq!(operand.raw_value(), raw);
            let syntax = operand.syntax();
            assert_eq!(syntax.source().text(), decoded);
            assert_eq!(syntax.source().span(), value);
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
            let Expression::ParenthesizedExpression(parentheses) = syntax.expression().unwrap()
            else {
                panic!("original physical-line parentheses")
            };
            assert_eq!(
                parentheses.span,
                oxc_span::Span::new(10 + shift, 18 + shift)
            );
            assert_eq!(
                syntax.authored_span(parentheses.span),
                Ok(Span::new(26 + shift, 42 + shift))
            );
        }
    }
}

#[test]
fn attribute_context_entity_ambiguity_and_one_decode_keep_the_complete_string_literal_spelling() {
    let arena = Allocator::default();
    let source = "<template><p :id='\"&amp=1 &notit; &amp;lt;\"'/></template>";
    let expected = "<template><p :id='\"&amp=1 &notit; &amp;lt;\"' /></template>";
    let owner = assert_output(&arena, source, expected, options(200, 2, LineEnding::Lf));
    let operand = &owner.binding_operands()[0];
    assert_eq!(operand.raw_value(), "\"&amp=1 &notit; &amp;lt;\"");
    assert_eq!(operand.value_span(), Span::new(18, 43));
    let syntax = operand.syntax();
    assert_eq!(syntax.source().text(), "\"&amp=1 &notit; &lt;\"");
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
        panic!("original string")
    };
    assert_eq!(literal.value.as_str(), "&amp=1 &notit; &lt;");
    assert_eq!(
        literal.raw.as_ref().unwrap().as_str(),
        "\"&amp=1 &notit; &lt;\""
    );
    assert_eq!(literal.span, oxc_span::Span::new(2, 23));
    assert_eq!(syntax.authored_span(literal.span), Ok(Span::new(18, 43)));
}

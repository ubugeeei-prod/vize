use super::super::{assert_output, options};
use oxc_ast::ast::{CommentKind, Expression};
use vize_glyph::native_doc::LineEnding;
use vize_l0::{Allocator, Span};
use vize_l1::embed::DecodeSegmentKind;

#[test]
fn physical_lf_crlf_comments_and_complete_entity_map_survive_generated_endings() {
    for (source, raw, decoded, physical, value_span, shift) in [
        (
            "<template><p v-if='/*雪*/\n(α&amp;&amp;β)//尾\n'>{{1n}}</p></template>",
            "/*雪*/\n(α&amp;&amp;β)//尾\n",
            "/*雪*/\n(α&&β)//尾\n",
            "\n",
            Span::new(19, 49),
            0,
        ),
        (
            "<template><p v-if='/*雪*/\r\n(α&amp;&amp;β)//尾\r\n'>{{1n}}</p></template>",
            "/*雪*/\r\n(α&amp;&amp;β)//尾\r\n",
            "/*雪*/\r\n(α&&β)//尾\r\n",
            "\r\n",
            Span::new(19, 51),
            1,
        ),
    ] {
        for (ending, generated) in [(LineEnding::Lf, "\n"), (LineEnding::CrLf, "\r\n")] {
            let expected = format!(
                "<template><p{generated}  v-if='/*雪*/{physical}(α &amp;&amp; β)//尾{physical}'{generated}>{{{{ 1n }}}}</p></template>"
            );
            let arena = Allocator::default();
            let owner = assert_output(&arena, source, &expected, options(200, 2, ending));
            assert_eq!(owner.attribute_operands().len(), 1);
            assert_eq!(owner.operands().len(), 1);
            let operand = &owner.attribute_operands()[0];
            assert_eq!(operand.name_span(), Span::new(13, 17));
            assert_eq!(operand.value_span(), value_span);
            assert_eq!(operand.raw_value(), raw);
            let syntax = operand.syntax();
            assert_eq!(syntax.source().text(), decoded);
            assert_eq!(syntax.source().span(), value_span);
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
                        Span::new(19, 30 + shift),
                        DecodeSegmentKind::Identity
                    ),
                    (
                        Span::new(11 + shift, 12 + shift),
                        Span::new(30 + shift, 35 + shift),
                        DecodeSegmentKind::Entity
                    ),
                    (
                        Span::new(12 + shift, 13 + shift),
                        Span::new(35 + shift, 40 + shift),
                        DecodeSegmentKind::Entity
                    ),
                    (
                        Span::new(13 + shift, 22 + 2 * shift),
                        Span::new(40 + shift, 49 + 2 * shift),
                        DecodeSegmentKind::Identity
                    ),
                ]
            );
            assert_eq!(
                syntax
                    .comments()
                    .map(|comment| (
                        comment.kind(),
                        comment.decoded_span().unwrap(),
                        comment.authored_span().unwrap(),
                        comment.text().unwrap(),
                    ))
                    .collect::<Vec<_>>(),
                [
                    (
                        CommentKind::SingleLineBlock,
                        Span::new(0, 7),
                        Span::new(19, 26),
                        "/*雪*/"
                    ),
                    (
                        CommentKind::Line,
                        Span::new(16 + shift, 21 + shift),
                        Span::new(43 + shift, 48 + shift),
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
                Ok(Span::new(27 + shift, 43 + shift))
            );
        }
    }
}

//! Genuine ordered Sequence documents preserve references and child layouts.

use oxc_ast::ast::Expression;
use vize_glyph::native_doc::{LineEnding, PrintOptions};
use vize_l0::{Allocator, Span};
use vize_l1::embed::Lang;

use super::{format, preservation::assert_preserved as assert_semantics, retained};

#[path = "sequence/custody.rs"]
mod custody;
#[path = "sequence/refusals.rs"]
mod refusals;

fn options(width: usize, line_ending: LineEnding) -> PrintOptions {
    PrintOptions {
        width,
        line_ending,
        ..PrintOptions::default()
    }
}

fn assert_preserved(source: &str, decode: bool) {
    assert_semantics(source, decode);
    for lang in [Lang::Js, Lang::Ts] {
        for width in [0, 1, 7, 80, 200] {
            for ending in [LineEnding::Lf, LineEnding::CrLf] {
                let options = options(width, ending);
                let output = format(source, lang, decode, options);
                assert_eq!(format(&output, lang, decode, options), output, "{source}");
            }
        }
    }
}

#[test]
fn complete_sequence_outputs_keep_order_parentheses_and_supported_reference_shapes() {
    for (source, expected) in [
        ("a,b", "a, b"),
        ("a,b,c", "a, b, c"),
        ("(a,b)", "(a, b)"),
        ("((a,b))", "((a, b))"),
        ("a,(b,c)", "a, (b, c)"),
        ("(a,b),c", "(a, b), c"),
        ("a,[b,,]", "a, [ b, , ]"),
        ("[(a,b)]", "[ (a, b) ]"),
        ("f((a,b),c)", "f ( (a, b), c )"),
        ("obj[a,b]", "obj [ a, b ]"),
        ("(a,b).key", "(a, b) . key"),
        ("!(a,b)", "! (a, b)"),
        ("(0,obj.method)()", "(0, obj . method) ( )"),
        ("(0,eval)('x')", "(0, eval) ( 'x' )"),
        ("delete(0,obj.x)", "delete (0, obj . x)"),
        (
            "first(),obj.value,last()",
            "first ( ), obj . value, last ( )",
        ),
        (
            "日本,\\u0061,0xCA_FE,'a,b'",
            "日本, \\u0061, 0xCA_FE, 'a,b'",
        ),
        ("true,false,null", "true, false, null"),
        ("1 .value,1..value", "1 . value, 1. . value"),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 1, 7, 80, 200] {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    assert_eq!(
                        format(source, lang, false, options(width, ending)),
                        expected,
                        "{source}"
                    );
                }
            }
        }
        assert_preserved(source, false);
    }
}

#[test]
fn original_parenthesized_sequences_distinguish_value_callees_and_delete_arguments() {
    for lang in [Lang::Js, Lang::Ts] {
        for (indirect, direct, member) in [
            ("(0,obj.method)()", "obj.method()", true),
            ("(0,eval)('x')", "eval('x')", false),
        ] {
            let arena = Allocator::default();
            let original = retained(
                &arena,
                indirect,
                Span::new(0, indirect.len() as u32),
                lang,
                false,
            );
            let Expression::CallExpression(call) = original.expression().unwrap() else {
                panic!("original indirect Call")
            };
            let Expression::ParenthesizedExpression(parentheses) = &call.callee else {
                panic!("original parenthesized callee")
            };
            let Expression::SequenceExpression(sequence) = &parentheses.expression else {
                panic!("original value-producing Sequence")
            };
            assert_eq!(sequence.expressions.len(), 2);
            assert!(
                matches!(&sequence.expressions[0], Expression::NumericLiteral(number) if number.value.to_bits() == 0.0_f64.to_bits())
            );
            assert_eq!(
                matches!(
                    &sequence.expressions[1],
                    Expression::StaticMemberExpression(_)
                ),
                member
            );
            assert!(!call.optional);
            assert!(!call.pure);
            assert!(call.type_arguments.is_none());
            let original = retained(
                &arena,
                direct,
                Span::new(0, direct.len() as u32),
                lang,
                false,
            );
            let Expression::CallExpression(call) = original.expression().unwrap() else {
                panic!("original direct Call")
            };
            assert_eq!(
                matches!(&call.callee, Expression::StaticMemberExpression(_)),
                member
            );
            assert!(matches!(
                &call.callee,
                Expression::Identifier(_) | Expression::StaticMemberExpression(_)
            ));
        }
        for (source, sequence_argument) in [("delete(0,obj.x)", true), ("delete obj.x", false)] {
            let arena = Allocator::default();
            let original = retained(
                &arena,
                source,
                Span::new(0, source.len() as u32),
                lang,
                false,
            );
            let Expression::UnaryExpression(unary) = original.expression().unwrap() else {
                panic!("original delete")
            };
            assert_eq!(unary.operator.as_str(), "delete");
            if sequence_argument {
                let Expression::ParenthesizedExpression(parentheses) = &unary.argument else {
                    panic!("original parenthesized delete argument")
                };
                let Expression::SequenceExpression(sequence) = &parentheses.expression else {
                    panic!("original Sequence delete argument")
                };
                assert!(matches!(
                    sequence.expressions.last().unwrap(),
                    Expression::StaticMemberExpression(_)
                ));
            } else {
                assert!(matches!(
                    &unary.argument,
                    Expression::StaticMemberExpression(_)
                ));
            }
        }
    }
    // These typed observations do not claim execution, getter or throw behavior.
}

#[test]
fn original_commas_comments_entities_and_physical_gaps_are_consumed_after_prior_children() {
    for (source, decode, expected) in [
        ("a /*,*/ , /*,*/ b", false, "a /*,*/ , /*,*/ b"),
        ("a/*,*/,/*,*/b", false, "a/*,*/,/*,*/b"),
        ("a//,\r\n,b", false, "a//,\r\n, b"),
        ("a,//,\n b", false, "a,//,\n b"),
        ("f(/*i*/)/*o*/,g(/*j*/)", false, "f (/*i*/)/*o*/, g (/*j*/)"),
        ("a,/*#__PURE__*/f(b)", false, "a,/*#__PURE__*/f ( b )"),
        ("'a,b',c", false, "'a,b', c"),
        ("a&#44;b", true, "a&#44; b"),
        ("a&#32;&#44;&#9;b", true, "a&#32;&#44;&#9;b"),
        ("a&#44;&#10;b", true, "a&#44;&#10;b"),
        ("&#40;a&#44;b&#41;", true, "&#40;a&#44; b&#41;"),
        ("a&#47;*,,*&#47;&#44;b", true, "a&#47;*,,*&#47;&#44; b"),
        ("&#39;//x&#39;\n,b", true, "&#39;//x&#39;\n, b"),
        ("a,&#39;//x&#39;\r\n,b", true, "a, &#39;//x&#39;\r\n, b"),
        ("a,\n&#39;b&#39;", true, "a,\n&#39;b&#39;"),
        ("a,\r\nb", false, "a, b"),
        ("a,\nb", true, "a, b"),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 1, 7, 80, 200] {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    assert_eq!(
                        format(source, lang, decode, options(width, ending)),
                        expected,
                        "{source}"
                    );
                }
            }
        }
        assert_preserved(source, decode);
    }
}

#[test]
fn sequences_add_no_breaks_or_indent_while_original_children_keep_their_groups() {
    for (source, flat, narrow) in [
        ("a+b,c", "a + b, c", "a +\n  b, c"),
        ("a,b+c,d", "a, b + c, d", "a, b +\n  c, d"),
        ("a,(b+c)", "a, (b + c)", "a, (b +\n  c)"),
        ("a?b:c,d", "a ? b : c, d", "a ?\n  b :\n  c, d"),
        ("a,b?c:d", "a, b ? c : d", "a, b ?\n  c :\n  d"),
        (
            "f((a+b,c)),d",
            "f ( (a + b, c) ), d",
            "f ( (a +\n  b, c) ), d",
        ),
        ("(a,b)+c", "(a, b) + c", "(a, b) +\n  c"),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for ending in [LineEnding::Lf, LineEnding::CrLf] {
                assert_eq!(format(source, lang, false, options(200, ending)), flat);
                let narrow = if ending == LineEnding::Lf {
                    narrow.to_owned()
                } else {
                    narrow.replace('\n', "\r\n")
                };
                assert_eq!(
                    format(source, lang, false, options(0, ending)),
                    narrow,
                    "{source}"
                );
            }
        }
        assert_preserved(source, false);
    }
}

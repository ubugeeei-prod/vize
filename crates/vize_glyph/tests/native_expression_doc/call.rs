//! Original Call output, punctuation, reference shape and observation custody.

use oxc_ast::ast::Expression;
use vize_glyph::native_doc::{LineEnding, PrintOptions};
use vize_l0::{Allocator, Span};
use vize_l1::embed::Lang;

use super::{format, preservation::assert_preserved as assert_semantics, retained};

#[path = "call/custody.rs"]
mod custody;
#[path = "call/refusals.rs"]
mod refusals;

fn options(width: usize) -> PrintOptions {
    PrintOptions {
        width,
        ..PrintOptions::default()
    }
}

fn assert_preserved(source: &str, decode: bool) {
    assert_semantics(source, decode);
    for lang in [Lang::Js, Lang::Ts] {
        for width in [0, 1, 7, 80, 200] {
            for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                let options = PrintOptions {
                    width,
                    line_ending,
                    ..PrintOptions::default()
                };
                let output = format(source, lang, decode, options);
                assert_eq!(format(&output, lang, decode, options), output, "{source}");
            }
        }
    }
}

#[test]
fn ordinary_calls_keep_complete_empty_ordered_nested_and_reference_callee_outputs() {
    for (source, expected) in [
        ("f()", "f ( )"),
        ("f(a,b,)", "f ( a, b, )"),
        ("f(a)(b)", "f ( a ) ( b )"),
        ("f(g(a),-1)", "f ( g ( a ), - 1 )"),
        ("obj.f(a)", "obj . f ( a )"),
        ("obj[f(a)]", "obj [ f ( a ) ]"),
        ("f().key", "f ( ) . key"),
        ("eval('x')", "eval ( 'x' )"),
        ("(eval)('x')", "(eval) ( 'x' )"),
        ("(obj.f)(a)", "(obj . f) ( a )"),
        (
            "日本(\\u0061,0xCA_FE,'x,)(y')",
            "日本 ( \\u0061, 0xCA_FE, 'x,)(y' )",
        ),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 1, 7, 80, 200] {
                assert_eq!(
                    format(source, lang, false, options(width)),
                    expected,
                    "{source}"
                );
            }
        }
        assert_preserved(source, false);
    }
}

#[test]
fn original_call_delimiters_separators_trailing_commas_and_typed_gaps_stay_checked() {
    for (source, decode, expected) in [
        ("f(a)", false, "f ( a )"),
        ("f(a,)", false, "f ( a, )"),
        ("f(/*c,)*/)", false, "f (/*c,)*/)"),
        (
            "f(/*inner*/)/*outer*/(a)",
            false,
            "f (/*inner*/)/*outer*/( a )",
        ),
        (
            "f(a/*before*/,/*after*/b/*last*/,/*end*/)",
            false,
            "f ( a/*before*/,/*after*/b/*last*/,/*end*/)",
        ),
        ("f(a//x\r\n,b)", false, "f ( a//x\r\n, b )"),
        ("f(a, //x\r\n b)", false, "f ( a, //x\r\n b )"),
        ("f(a,/*literal,)*/)", false, "f ( a,/*literal,)*/)"),
        (
            "f&#40;a&#44;b&#44;&#41;",
            true,
            "f &#40; a&#44; b&#44; &#41;",
        ),
        (
            "f&#32;(&#9;a&#32;,&#9;b&#44;&#32;)",
            true,
            "f&#32;(&#9;a&#32;,&#9;b&#44;&#32;)",
        ),
        (
            "f(a&#47;*,,*&#47;&#44;)",
            true,
            "f ( a&#47;*,,*&#47;&#44; )",
        ),
        ("f(&#39;//x&#39;\n)", true, "f ( &#39;//x&#39;\n)"),
        (
            "f(&quot;//x&quot;\r\n, b,)",
            true,
            "f ( &quot;//x&quot;\r\n, b, )",
        ),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 1, 7, 80, 200] {
                assert_eq!(
                    format(source, lang, decode, options(width)),
                    expected,
                    "{source}"
                );
            }
        }
        assert_preserved(source, decode);
    }
}

#[test]
fn call_children_keep_original_parentheses_and_infix_breaks_without_new_call_breaks() {
    for (source, flat, narrow) in [
        ("f(a+b,c)", "f ( a + b, c )", "f ( a +\n  b, c )"),
        ("(a+b)(c)", "(a + b) ( c )", "(a +\n  b) ( c )"),
        ("a+f(b)", "a + f ( b )", "a +\n  f ( b )"),
        (
            "f((a+b),g(c))",
            "f ( (a + b), g ( c ) )",
            "f ( (a +\n  b), g ( c ) )",
        ),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            assert_eq!(format(source, lang, false, options(200)), flat);
            assert_eq!(format(source, lang, false, options(0)), narrow);
            assert_eq!(
                format(
                    source,
                    lang,
                    false,
                    PrintOptions {
                        width: 0,
                        line_ending: LineEnding::CrLf,
                        ..PrintOptions::default()
                    }
                ),
                narrow.replace('\n', "\r\n")
            );
        }
        assert_preserved(source, false);
    }
}

#[test]
fn pure_call_metadata_and_complete_original_annotation_comments_survive_real_reparse() {
    for source in ["/*#__PURE__*/f()", "/* @__PURE__ */f(a,)"] {
        for lang in [Lang::Js, Lang::Ts] {
            let allocator = Allocator::default();
            let original = retained(
                &allocator,
                source,
                Span::new(0, source.len() as u32),
                lang,
                false,
            );
            let Expression::CallExpression(call) = original.expression().unwrap() else {
                panic!("original Call")
            };
            assert!(call.pure, "{source}");
            assert_eq!(original.comments().count(), 1);
            assert!(!call.optional);
            assert!(call.type_arguments.is_none());
        }
        assert_preserved(source, false);
    }
    assert_preserved("f(/*#__PURE__*/g(a),b)", false);
    assert_eq!(
        format("/*#__PURE__*/f()", Lang::Js, false, options(200)),
        "/*#__PURE__*/f ( )"
    );
}

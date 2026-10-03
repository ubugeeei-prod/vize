//! Original sparse Array documents keep holes, punctuation and child layout.

use oxc_ast::ast::{ArrayExpressionElement, Expression};
use oxc_span::GetSpan;
use vize_glyph::native_doc::{LineEnding, PrintOptions};
use vize_l0::{Allocator, Span};
use vize_l1::embed::Lang;

use super::{format, preservation::assert_preserved as assert_semantics, retained};

#[path = "array/custody.rs"]
mod custody;
#[path = "array/refusals.rs"]
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
            for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                let options = options(width, line_ending);
                let output = format(source, lang, decode, options);
                assert_eq!(format(&output, lang, decode, options), output, "{source}");
            }
        }
    }
}

fn original_holes(source: &str, lang: Lang) -> std::vec::Vec<bool> {
    let arena = Allocator::default();
    let original = retained(
        &arena,
        source,
        Span::new(0, source.len() as u32),
        lang,
        false,
    );
    assert_eq!(original.hole(), None);
    let Expression::ArrayExpression(array) = original.expression().unwrap() else {
        panic!("original Array")
    };
    array
        .elements
        .iter()
        .map(|element| {
            if let ArrayExpressionElement::Elision(hole) = element {
                let decoded = original.decoded_span(hole.span()).unwrap();
                assert_eq!(decoded.end - decoded.start, 1);
                assert_eq!(decoded.slice(original.source().text()), ",");
                true
            } else {
                assert!(matches!(
                    element.as_expression(),
                    Some(Expression::Identifier(_))
                ));
                false
            }
        })
        .collect()
}

#[test]
fn original_sparse_array_shapes_distinguish_holes_from_empty_and_trailing_separators() {
    for lang in [Lang::Js, Lang::Ts] {
        for (source, holes) in [
            ("[]", std::vec![]),
            ("[,]", std::vec![true]),
            ("[,,]", std::vec![true, true]),
            ("[a,]", std::vec![false]),
            ("[a,,]", std::vec![false, true]),
            ("[,a,]", std::vec![true, false]),
            ("[a,,b]", std::vec![false, true, false]),
            ("[,a,b]", std::vec![true, false, false]),
        ] {
            assert_eq!(original_holes(source, lang), holes, "{source}");
        }
        for (left, right) in [("[]", "[,]"), ("[a,]", "[a,,]"), ("[a,,b]", "[,a,b]")] {
            assert_ne!(original_holes(left, lang), original_holes(right, lang));
        }
    }
}

#[test]
fn complete_array_outputs_keep_order_sparse_slots_and_nested_supported_families() {
    for (source, expected) in [
        ("[]", "[ ]"),
        ("[,]", "[ , ]"),
        ("[,,]", "[ , , ]"),
        ("[a]", "[ a ]"),
        ("[a,]", "[ a, ]"),
        ("[a,,]", "[ a, , ]"),
        ("[,a]", "[ , a ]"),
        ("[,a,]", "[ , a, ]"),
        ("[,,a,,]", "[ , , a, , ]"),
        ("[a,,b,]", "[ a, , b, ]"),
        ("[[]]", "[ [ ] ]"),
        ("[,[a,,],]", "[ , [ a, , ], ]"),
        (
            "[+a,-b,obj.key,f(x),true,null]",
            "[ + a, - b, obj . key, f ( x ), true, null ]",
        ),
        (
            "[日本,\\u0061,0xCA_FE,'x,][y']",
            "[ 日本, \\u0061, 0xCA_FE, 'x,][y' ]",
        ),
        ("[1 .value,1..value]", "[ 1 . value, 1. . value ]"),
        ("[]()", "[ ] ( )"),
        ("[a][b]", "[ a ] [ b ]"),
        ("f([,a,])", "f ( [ , a, ] )"),
        ("![a,b]", "! [ a, b ]"),
        ("['\u{a0}']", "[ '\u{a0}' ]"),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 1, 7, 80, 200] {
                for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                    assert_eq!(
                        format(source, lang, false, options(width, line_ending)),
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
fn original_array_brackets_elisions_commas_comments_entities_and_physical_gaps_stay_authored() {
    for (source, decode, expected) in [
        ("[/*open*/]", false, "[/*open*/]"),
        ("[,/*gap*/]", false, "[ ,/*gap*/]"),
        ("[/*first*/,,/*last*/]", false, "[/*first*/, ,/*last*/]"),
        (
            "[a/*before*/,/*after*/,b/*last*/,/*end*/]",
            false,
            "[ a/*before*/,/*after*/, b/*last*/,/*end*/]",
        ),
        ("[a//x\r\n,,b]", false, "[ a//x\r\n, , b ]"),
        ("[, //x\n ,a]", false, "[ , //x\n , a ]"),
        ("[a,/*literal,,]*/]", false, "[ a,/*literal,,]*/]"),
        ("&#91;&#44;&#93;", true, "&#91; &#44; &#93;"),
        (
            "&#91;a&#44;&#44;b&#44;&#93;",
            true,
            "&#91; a&#44; &#44; b&#44; &#93;",
        ),
        (
            "&#91;&#32;&#44;&#9;a&#32;&#44;&#32;&#93;",
            true,
            "&#91;&#32;&#44;&#9;a&#32;&#44;&#32;&#93;",
        ),
        ("[a&#47;*,,*&#47;&#44;]", true, "[ a&#47;*,,*&#47;&#44; ]"),
        ("[&#39;//x&#39;\n,]", true, "[ &#39;//x&#39;\n, ]"),
        ("[&#39;//x&#39;\r\n,,]", true, "[ &#39;//x&#39;\r\n, , ]"),
        ("[&#39;//x&#39;\n]", true, "[ &#39;//x&#39;\n]"),
        ("[&#39;//x&#39;&#10;]", true, "[ &#39;//x&#39;&#10;]"),
        ("[,\r\n&#39;x&#39;]", true, "[ ,\r\n&#39;x&#39; ]"),
        ("[a&#44;\n]", true, "[ a&#44;\n]"),
        ("[&#44;\n]", true, "[ &#44;\n]"),
        ("[ \n,\n a,\n ]", true, "[ , a, ]"),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 1, 7, 80, 200] {
                for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                    assert_eq!(
                        format(source, lang, decode, options(width, line_ending)),
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
fn array_adds_no_breaks_or_indent_while_original_child_groups_still_lay_out() {
    for (source, flat, narrow) in [
        ("[a+b,c]", "[ a + b, c ]", "[ a +\n  b, c ]"),
        ("[[a+b],c]", "[ [ a + b ], c ]", "[ [ a +\n  b ], c ]"),
        ("[a?b:c,d]", "[ a ? b : c, d ]", "[ a ?\n  b :\n  c, d ]"),
        (
            "[f(a+b),obj[a?b:c]]",
            "[ f ( a + b ), obj [ a ? b : c ] ]",
            "[ f ( a +\n  b ), obj [ a ?\n  b :\n  c ] ]",
        ),
        (
            "a?[b,,]:[,c]",
            "a ? [ b, , ] : [ , c ]",
            "a ?\n  [ b, , ] :\n  [ , c ]",
        ),
        (
            "[(-a)**b,-(a**b)]",
            "[ (- a) ** b, - (a ** b) ]",
            "[ (- a) **\n  b, - (a **\n  b) ]",
        ),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                assert_eq!(format(source, lang, false, options(200, line_ending)), flat);
                let narrow = if line_ending == LineEnding::Lf {
                    narrow.to_owned()
                } else {
                    narrow.replace('\n', "\r\n")
                };
                assert_eq!(
                    format(source, lang, false, options(0, line_ending)),
                    narrow,
                    "{source}"
                );
            }
        }
        assert_preserved(source, false);
    }
}

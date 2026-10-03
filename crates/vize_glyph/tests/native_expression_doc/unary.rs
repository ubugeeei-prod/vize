//! Genuine prefix nodes keep operators, separation and original custody.

use oxc_ast::ast::Expression;
use vize_glyph::native_doc::{LineEnding, PrintOptions, expression_document, print};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::Lang;

use super::{format, preservation::assert_preserved, retained};

#[test]
fn all_seven_original_unary_operators_have_whole_js_ts_outputs() {
    for lang in [Lang::Js, Lang::Ts] {
        for (source, expected) in [
            ("+1", "+ 1"),
            ("-0xCA_FE", "- 0xCA_FE"),
            ("!true", "! true"),
            ("~0b10", "~ 0b10"),
            ("typeof   日本", "typeof 日本"),
            ("void\t null", "void null"),
            ("delete  0", "delete 0"),
        ] {
            for width in [0, 7, 200] {
                assert_eq!(
                    format(
                        source,
                        lang,
                        false,
                        PrintOptions {
                            width,
                            ..PrintOptions::default()
                        }
                    ),
                    expected,
                    "{source}"
                );
            }
        }
    }
}

#[test]
fn nested_signs_never_fuse_into_prefix_or_postfix_updates() {
    for (source, expected) in [
        ("+ +a", "+ + a"),
        ("- -a", "- - a"),
        ("+ -a", "+ - a"),
        ("- +a", "- + a"),
        ("!!a", "! ! a"),
        ("~ -1", "~ - 1"),
        ("!\\u0061", "! \\u0061"),
        ("void 'x/*y*/'", "void 'x/*y*/'"),
        ("-/*x*/-1", "-/*x*/- 1"),
    ] {
        assert_eq!(
            format(source, Lang::Js, false, PrintOptions::default()),
            expected
        );
    }
}

#[test]
fn unary_precedence_and_infix_signs_keep_parentheses_at_flat_and_narrow_widths() {
    for (source, flat, narrow) in [
        ("a + +b", "a + + b", "a +\n  + b"),
        ("a - -b", "a - - b", "a -\n  - b"),
        ("-(a+b)", "- (a + b)", "- (a +\n  b)"),
        ("(-a)**b", "(- a) ** b", "(- a) **\n  b"),
        ("-(a**b)", "- (a ** b)", "- (a **\n  b)"),
        ("typeof (a&&b)", "typeof (a && b)", "typeof (a &&\n  b)"),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for (width, expected) in [(200, flat), (0, narrow)] {
                assert_eq!(
                    format(
                        source,
                        lang,
                        false,
                        PrintOptions {
                            width,
                            ..PrintOptions::default()
                        }
                    ),
                    expected
                );
            }
        }
    }
}

#[test]
fn typed_comments_preserve_exact_prefix_gaps_including_authored_crlf() {
    for source in [
        "/*l*/!/*x*/true//t",
        "typeof/*x*/日本",
        "! //x\r\n value",
        "! /*x\r\ny*/ value",
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 7, 200] {
                assert_eq!(
                    format(
                        source,
                        lang,
                        false,
                        PrintOptions {
                            width,
                            line_ending: LineEnding::CrLf,
                            ..PrintOptions::default()
                        }
                    ),
                    source
                );
            }
        }
    }
    assert_eq!(
        format(
            "a+!/*x\ny*/b",
            Lang::Js,
            false,
            PrintOptions {
                width: 0,
                line_ending: LineEnding::CrLf,
                ..PrintOptions::default()
            }
        ),
        "a +\r\n  !/*x\ny*/b"
    );
}

#[test]
fn complete_encoded_operators_gaps_and_typed_comments_keep_authored_spelling() {
    for (source, expected) in [
        ("&#33;ready", "&#33; ready"),
        ("&#45;1", "&#45; 1"),
        ("t&#121;peof value", "t&#121;peof value"),
        ("&#43;&#32;&#43;a", "&#43;&#32;&#43; a"),
        ("&#45;&#32;&#45;&#9;1", "&#45;&#32;&#45;&#9;1"),
        ("&#33;&#10;value", "&#33;&#10;value"),
        ("&#33;&#47;*x*&#47;ready", "&#33;&#47;*x*&#47;ready"),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 7, 200] {
                assert_eq!(
                    format(
                        source,
                        lang,
                        true,
                        PrintOptions {
                            width,
                            ..PrintOptions::default()
                        }
                    ),
                    expected
                );
            }
        }
    }
}

#[test]
fn nonzero_unary_blocks_keep_the_same_original_node_comment_and_decode_map_storage() {
    let allocator = Allocator::default();
    let root = "é &#33;&#47;*x*&#47;日本 tail";
    let selected = "&#33;&#47;*x*&#47;日本";
    let span = Span::new(3, (3 + selected.len()) as u32);
    let original = retained(&allocator, root, span, Lang::Ts, true);
    let Expression::UnaryExpression(unary) = original.expression().unwrap() else {
        panic!("original unary node")
    };
    let argument = core::ptr::from_ref(&unary.argument);
    let comments = original
        .comments()
        .map(|comment| {
            (
                comment.kind(),
                comment.decoded_span().unwrap(),
                comment.authored_span().unwrap(),
                comment.text().unwrap().as_ptr(),
            )
        })
        .collect::<std::vec::Vec<_>>();
    assert_eq!(comments.len(), 1);
    let map = original.source().decode_map().unwrap().segments().as_ptr();
    let block = SourceRoot::new(root)
        .unwrap()
        .block(root.get(3..span.end as usize).unwrap(), 3)
        .unwrap();
    let (same, document) = expression_document(&original, block, &allocator)
        .unwrap()
        .into_parts();
    assert!(core::ptr::eq(same, &original));
    assert_eq!(print(&document, &PrintOptions::default()), selected);
    let Expression::UnaryExpression(same_unary) = same.expression().unwrap() else {
        panic!("same original unary node")
    };
    assert!(core::ptr::eq(same_unary, unary));
    assert_eq!(core::ptr::from_ref(&same_unary.argument), argument);
    assert_eq!(same.source().decode_map().unwrap().segments().as_ptr(), map);
    assert_eq!(same.source().authored_root().as_ptr(), root.as_ptr());
    assert_eq!(
        same.comments()
            .map(|comment| (
                comment.kind(),
                comment.decoded_span().unwrap(),
                comment.authored_span().unwrap(),
                comment.text().unwrap().as_ptr(),
            ))
            .collect::<std::vec::Vec<_>>(),
        comments
    );
}

#[test]
fn unary_whole_outputs_are_fixed_points_and_preserve_independent_js_ts_ast_fingerprints() {
    for (source, decode) in [
        ("+1", false),
        ("-0xCA_FE", false),
        ("!true", false),
        ("~0b10", false),
        ("typeof 日本", false),
        ("void null", false),
        ("delete 0", false),
        ("delete (1)", false),
        ("+ +a", false),
        ("- -a", false),
        ("+ -a", false),
        ("- +a", false),
        ("!!a", false),
        ("~ -1", false),
        ("!\\u0061", false),
        ("void 'x/*y*/'", false),
        ("a + +b", false),
        ("a - -b", false),
        ("-(a+b)", false),
        ("(-a)**b", false),
        ("-(a**b)", false),
        ("typeof (a&&b)", false),
        ("/*l*/!/*x*/true//t", false),
        ("! //x\r\n value", false),
        ("! /*x\r\ny*/ value", false),
        ("-/*x*/-1", false),
        ("&#33;ready", true),
        ("t&#121;peof value", true),
        ("&#45;&#32;&#45;&#9;1", true),
        ("&#33;&#47;*x*&#47;ready", true),
        ("&#33;&#10;value", true),
    ] {
        assert_preserved(source, decode);
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 7, 20, 200] {
                let options = PrintOptions {
                    width,
                    ..PrintOptions::default()
                };
                let output = format(source, lang, decode, options);
                assert_eq!(format(&output, lang, decode, options), output, "{source}");
            }
        }
    }
}

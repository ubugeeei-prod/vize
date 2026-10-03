//! Real retained compiler-profile AST documents; no formatter parse or oracle.
use vize_glyph::native_doc::{LineEnding, PrintOptions, expression_document, print};
use vize_l0::{Allocator, SourceRoot, Span, String};
use vize_l1::embed::{
    Embed, EmbedSource, Grammar, Lang, Shape, prepare_attribute_value,
    syntax::{RetainedExpression, parse_once},
};
#[path = "native_expression_doc/array.rs"]
mod array;
#[path = "native_expression_doc/authored_newlines.rs"]
mod authored_newlines;
#[path = "native_expression_doc/call.rs"]
mod call;
#[path = "native_expression_doc/conditional.rs"]
mod conditional;
#[path = "native_expression_doc/member.rs"]
mod member;
#[path = "native_expression_doc/object.rs"]
mod object;
#[path = "native_expression_doc/preservation.rs"]
mod preservation;
#[path = "native_expression_doc/refusals.rs"]
mod refusals;
#[path = "native_expression_doc/sequence.rs"]
mod sequence;
#[path = "native_expression_doc/unary.rs"]
mod unary;
fn retained<'a>(
    allocator: &'a Allocator,
    root: &'a str,
    span: Span,
    lang: Lang,
    decode: bool,
) -> RetainedExpression<'a> {
    let source = if decode {
        prepare_attribute_value(allocator, root, span).unwrap()
    } else {
        EmbedSource::authored(root, span).unwrap()
    };
    parse_once(
        allocator,
        Embed {
            grammar: Grammar {
                shape: Shape::Expr,
                lang,
            },
            source,
        },
    )
    .into_expression()
    .unwrap()
}
fn format(source: &str, lang: Lang, decode: bool, options: PrintOptions) -> String {
    let allocator = Allocator::default();
    let original = retained(
        &allocator,
        source,
        Span::new(0, source.len() as u32),
        lang,
        decode,
    );
    let root = core::ptr::from_ref(original.expression().unwrap());
    let comments = original
        .comments()
        .map(|comment| {
            (
                comment.decoded_span().unwrap(),
                comment.text().unwrap().as_ptr(),
            )
        })
        .collect::<std::vec::Vec<_>>();
    let document = expression_document(
        &original,
        SourceRoot::new(source).unwrap().whole_block(),
        &allocator,
    )
    .unwrap();
    assert!(core::ptr::eq(document.original(), &original));
    assert_eq!(
        core::ptr::from_ref(document.original().expression().unwrap()),
        root
    );
    assert_eq!(original.grammar().lang, lang);
    assert_eq!(
        original
            .comments()
            .map(|comment| (
                comment.decoded_span().unwrap(),
                comment.text().unwrap().as_ptr()
            ))
            .collect::<std::vec::Vec<_>>(),
        comments
    );
    print(document.document(), &options)
}
#[test]
fn original_atoms_keep_unicode_escapes_and_literal_spellings_in_js_and_ts() {
    for lang in [Lang::Js, Lang::Ts] {
        for atom in [
            "name",
            "日本",
            "\\u0061",
            "1_000",
            "0xCA_FE",
            "0b1010",
            "'a\\x20b'",
            "\"x//y\"",
            "true",
            "false",
            "null",
        ] {
            let source = std::format!(" \t{atom}\r\n ");
            assert_eq!(format(&source, lang, false, PrintOptions::default()), atom);
        }
    }
}
#[test]
fn whole_binary_and_logical_output_flattens_or_breaks_original_nodes() {
    assert_eq!(
        format("left  +  right", Lang::Js, false, PrintOptions::default()),
        "left + right"
    );
    assert_eq!(
        format(
            "left  +  right",
            Lang::Js,
            false,
            PrintOptions {
                width: 8,
                ..PrintOptions::default()
            }
        ),
        "left +\n  right"
    );
    assert_eq!(
        format(
            "ready&&value",
            Lang::Ts,
            false,
            PrintOptions {
                width: 8,
                ..PrintOptions::default()
            }
        ),
        "ready &&\n  value"
    );
    assert_eq!(
        format(
            "( a + b )",
            Lang::Js,
            false,
            PrintOptions {
                width: 4,
                ..PrintOptions::default()
            }
        ),
        "(a +\n  b)"
    );
}
#[test]
fn original_parentheses_and_precedence_are_never_removed_or_invented() {
    for (source, expected) in [
        (" (( value )) ", "((value))"),
        ("( (left+1) * (right-2) )", "((left + 1) * (right - 2))"),
        ("a + b * c", "a + b * c"),
        ("(a ?? b) || c", "(a ?? b) || c"),
    ] {
        assert_eq!(
            format(
                source,
                Lang::Js,
                false,
                PrintOptions {
                    width: 200,
                    ..PrintOptions::default()
                }
            ),
            expected
        );
    }
}
#[test]
fn every_original_binary_and_logical_operator_spelling_stays_exact() {
    for operator in [
        "+",
        "-",
        "*",
        "/",
        "%",
        "**",
        "==",
        "!=",
        "===",
        "!==",
        "<",
        "<=",
        ">",
        ">=",
        "<<",
        ">>",
        ">>>",
        "|",
        "&",
        "^",
        "in",
        "instanceof",
        "&&",
        "||",
        "??",
    ] {
        let source = std::format!("a   {operator}   b");
        assert_eq!(
            format(&source, Lang::Js, false, PrintOptions::default()),
            std::format!("a {operator} b")
        );
    }
}

#[test]
fn typed_leading_inner_and_trailing_comments_keep_their_exact_authored_gaps() {
    for (source, expected) in [
        ("/*l*/ a   +b //t", "/*l*/ a + b //t"),
        ("a /*x*/  +  b", "a /*x*/  + b"),
        ("a+ /*x*/  b", "a + /*x*/  b"),
        ("(/*x*/ a+b /*y*/)", "(/*x*/ a + b /*y*/)"),
        ("a + //x\n b", "a + //x\n b"),
        ("a //x\n + b", "a //x\n +\n  b"),
        ("a /*x\r\ny*/ + b", "a /*x\r\ny*/ +\n  b"),
    ] {
        assert_eq!(
            format(source, Lang::Js, false, PrintOptions::default()),
            expected
        );
    }
}

#[test]
fn decoded_entities_keep_literal_operator_parenthesis_and_comment_spelling() {
    for (source, expected) in [
        (
            "&fjlig;   &amp;&amp; ( tr&#117;e )",
            "&fjlig; &amp;&amp; (tr&#117;e)",
        ),
        ("&#40; a + b &#41;", "&#40;a + b&#41;"),
        ("a &#47;*x*&#47; + b", "a &#47;*x*&#47; + b"),
        ("'a&amp;b'", "'a&amp;b'"),
    ] {
        assert_eq!(
            format(source, Lang::Js, true, PrintOptions::default()),
            expected
        );
    }
}

#[test]
fn encoded_whitespace_gaps_remain_authored_at_every_width() {
    for source in ["a&#32;+&#9;b", "a +&#10;b", "&#32;(a)&#32;"] {
        for width in [0, 1, 80] {
            assert_eq!(
                format(
                    source,
                    Lang::Ts,
                    true,
                    PrintOptions {
                        width,
                        ..PrintOptions::default()
                    }
                ),
                source
            );
        }
    }
}

#[test]
fn selected_nonzero_blocks_and_original_decode_map_storage_keep_custody() {
    let allocator = Allocator::default();
    let root = "é &fjlig;  +  1 tail";
    let selected = root.get(3..16).unwrap();
    let original = retained(&allocator, root, Span::new(3, 16), Lang::Ts, true);
    let map = original.source().decode_map().unwrap().segments().as_ptr();
    let block = SourceRoot::new(root).unwrap().block(selected, 3).unwrap();
    let document = expression_document(&original, block, &allocator).unwrap();
    assert_eq!(
        print(document.document(), &PrintOptions::default()),
        "&fjlig; + 1"
    );
    assert_eq!(
        document
            .original()
            .source()
            .decode_map()
            .unwrap()
            .segments()
            .as_ptr(),
        map
    );
    assert_eq!(original.source().authored_root().as_ptr(), root.as_ptr());
}

#[test]
fn supported_whole_documents_are_fixed_points_without_replacing_parse_observations() {
    for (source, decode) in [
        (" ( left+right ) * 2 ", false),
        ("a /*x*/ + (b&&c) //t", false),
        ("&fjlig; &amp;&amp; ( tr&#117;e )", true),
        ("a&#32;+&#9;b", true),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 7, 20, 80] {
                let options = PrintOptions {
                    width,
                    ..PrintOptions::default()
                };
                let output = format(source, lang, decode, options);
                assert_eq!(format(&output, lang, decode, options), output);
            }
        }
    }
}

#[test]
fn generated_crlf_does_not_rewrite_authored_comment_newlines() {
    assert_eq!(
        format(
            "a + b",
            Lang::Js,
            false,
            PrintOptions {
                width: 0,
                line_ending: LineEnding::CrLf,
                ..PrintOptions::default()
            }
        ),
        "a +\r\n  b"
    );
    assert_eq!(
        format(
            "a /*x\ny*/ + b",
            Lang::Js,
            false,
            PrintOptions {
                width: 0,
                line_ending: LineEnding::CrLf,
                ..PrintOptions::default()
            }
        ),
        "a /*x\ny*/ +\r\n  b"
    );
}

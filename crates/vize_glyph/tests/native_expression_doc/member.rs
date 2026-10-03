//! Whole original ordinary-member documents and independent custody laws.

use oxc_ast::ast::Expression;
use vize_glyph::native_doc::{LineEnding, PrintOptions, expression_document, print};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::Lang;

use super::{format, preservation::assert_preserved, refusals::assert_unsupported, retained};

fn options(width: usize) -> PrintOptions {
    PrintOptions {
        width,
        ..PrintOptions::default()
    }
}

#[test]
fn static_computed_and_nested_members_keep_complete_js_ts_output() {
    for (source, expected) in [
        ("obj.key", "obj . key"),
        ("obj.default", "obj . default"),
        ("obj.\\u0061", "obj . \\u0061"),
        ("日本.値", "日本 . 値"),
        ("obj[0]", "obj [ 0 ]"),
        ("obj['x/*y*/']", "obj [ 'x/*y*/' ]"),
        ("obj.key[+ +i].value", "obj . key [ + + i ] . value"),
        ("(-a).b", "(- a) . b"),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            assert_eq!(format(source, lang, false, options(200)), expected);
        }
    }
}

#[test]
fn member_dots_never_change_original_numeric_literal_token_boundaries() {
    for (source, expected) in [
        ("1 .value", "1 . value"),
        ("1..value", "1. . value"),
        ("1.0.value", "1.0 . value"),
        ("0x10.value", "0x10 . value"),
        ("1[0]", "1 [ 0 ]"),
        ("1/*x*/.value", "1/*x*/. value"),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 7, 200] {
                assert_eq!(format(source, lang, false, options(width)), expected);
            }
        }
    }
}

#[test]
fn member_children_keep_parentheses_and_original_binary_layout_at_each_width() {
    for (source, flat, narrow) in [
        ("(a+b).c", "(a + b) . c", "(a +\n  b) . c"),
        ("obj[a+b]", "obj [ a + b ]", "obj [ a +\n  b ]"),
        ("obj[(a+b)]", "obj [ (a + b) ]", "obj [ (a +\n  b) ]"),
        ("a+obj[i]", "a + obj [ i ]", "a +\n  obj [ i ]"),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for (width, expected) in [(200, flat), (0, narrow)] {
                assert_eq!(format(source, lang, false, options(width)), expected);
            }
        }
    }
}

#[test]
fn typed_member_comments_and_complete_encoded_tokens_keep_authored_gaps() {
    for (source, decode, expected) in [
        ("obj/*a*/./*b*/key", false, "obj/*a*/./*b*/key"),
        ("obj/*a*/[/*b*/i/*c*/]", false, "obj/*a*/[/*b*/i/*c*/]"),
        ("obj.//x\r\n key", false, "obj .//x\r\n key"),
        ("obj //x\r\n .key", false, "obj //x\r\n . key"),
        ("obj[ //x\r\n i ]", false, "obj [ //x\r\n i ]"),
        ("obj&#46;key", true, "obj &#46; key"),
        ("obj&#91;i&#93;", true, "obj &#91; i &#93;"),
        ("1&#46;&#46;value", true, "1&#46; &#46; value"),
        ("obj&#32;.&#9;key", true, "obj&#32;.&#9;key"),
        ("obj&#32;[&#9;i&#10;]", true, "obj&#32;[&#9;i&#10;]"),
        ("obj&#47;*x*&#47;.key", true, "obj&#47;*x*&#47;. key"),
        ("obj.&fjlig;", true, "obj . &fjlig;"),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 7, 200] {
                assert_eq!(format(source, lang, decode, options(width)), expected);
            }
        }
    }
    assert_eq!(
        format(
            "obj[a /*x\ny*/ + b]",
            Lang::Js,
            false,
            PrintOptions {
                width: 0,
                line_ending: LineEnding::CrLf,
                ..PrintOptions::default()
            }
        ),
        "obj [ a /*x\ny*/ +\r\n  b ]"
    );
}

#[test]
fn original_mapped_member_object_key_property_and_comment_storage_keep_custody() {
    let allocator = Allocator::default();
    let root = "é obj&#46;&#47;*x*&#47;日本[+i] tail";
    let selected = "obj&#46;&#47;*x*&#47;日本[+i]";
    let span = Span::new(3, (3 + selected.len()) as u32);
    let original = retained(&allocator, root, span, Lang::Ts, true);
    let Expression::ComputedMemberExpression(member) = original.expression().unwrap() else {
        panic!("original computed member")
    };
    let Expression::StaticMemberExpression(field) = &member.object else {
        panic!("original static object")
    };
    let key = core::ptr::from_ref(&member.expression);
    let property = core::ptr::from_ref(&field.property);
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
    assert_eq!(
        print(&document, &options(200)),
        "obj &#46;&#47;*x*&#47;日本 [ + i ]"
    );
    let Expression::ComputedMemberExpression(same_member) = same.expression().unwrap() else {
        panic!("same computed member")
    };
    let Expression::StaticMemberExpression(same_field) = &same_member.object else {
        panic!("same static object")
    };
    assert!(core::ptr::eq(same_member, member));
    assert_eq!(core::ptr::from_ref(&same_member.expression), key);
    assert_eq!(core::ptr::from_ref(&same_field.property), property);
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
fn ordinary_members_are_fixed_points_and_keep_independent_static_computed_js_ts_fingerprints() {
    for (source, decode) in [
        ("obj.key", false),
        ("obj.default", false),
        ("obj.\\u0061", false),
        ("日本.値", false),
        ("obj['x/*y*/']", false),
        ("obj.key[+ +i].value", false),
        ("(-a).b", false),
        ("1 .value", false),
        ("1..value", false),
        ("1.0.value", false),
        ("0x10.value", false),
        ("1[0]", false),
        ("1/*x*/.value", false),
        ("(a+b).c", false),
        ("obj[a+b]", false),
        ("obj[(a+b)]", false),
        ("a+obj[i]", false),
        ("obj/*a*/./*b*/key", false),
        ("obj/*a*/[/*b*/i/*c*/]", false),
        ("obj.//x\r\n key", false),
        ("obj //x\r\n .key", false),
        ("obj[ //x\r\n i ]", false),
        ("obj[a /*x\ny*/ + b]", false),
        ("obj&#46;key", true),
        ("obj&#91;i&#93;", true),
        ("1&#46;&#46;value", true),
        ("obj&#32;.&#9;key", true),
        ("obj&#32;[&#9;i&#10;]", true),
        ("obj&#47;*x*&#47;.key", true),
        ("obj.&fjlig;", true),
    ] {
        assert_preserved(source, decode);
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 7, 20, 200] {
                let output = format(source, lang, decode, options(width));
                assert_eq!(
                    format(&output, lang, decode, options(width)),
                    output,
                    "{source}"
                );
            }
        }
    }
}

#[test]
fn chains_private_fields_calls_and_unsupported_member_descendants_refuse_whole_documents() {
    for content in [
        "a?.b",
        "a?.[b]",
        "(a?.b).c",
        "a.#x",
        "call().x",
        "a[call()]",
        "a.x()",
        "a[b,c]",
        "a[await b]",
        "a[x=1]",
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            let allocator = Allocator::default();
            let source = std::format!("/*kept*/ {content}");
            let original = retained(
                &allocator,
                &source,
                Span::new(0, source.len() as u32),
                lang,
                false,
            );
            if content == "a.#x" {
                assert!(matches!(
                    original.expression(),
                    Some(Expression::PrivateFieldExpression(_))
                ));
            } else if matches!(content, "a?.b" | "a?.[b]") {
                assert!(matches!(
                    original.expression(),
                    Some(Expression::ChainExpression(_))
                ));
            }
            assert_unsupported(&original, &source, &allocator);
        }
    }
}

#[test]
fn actual_ts_non_null_and_assertion_descendants_remain_explicit_refusals() {
    for content in ["a!.b", "(a as T).b", "a[b!]", "a[b as T]"] {
        let allocator = Allocator::default();
        let source = std::format!("/*kept*/ {content}");
        let original = retained(
            &allocator,
            &source,
            Span::new(0, source.len() as u32),
            Lang::Ts,
            false,
        );
        assert_unsupported(&original, &source, &allocator);
    }
}

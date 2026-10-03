//! Complete selected-template member documents retain genuine original facts.

use super::{format, operands, preservation::assert_preserved, selected};
use vize_glyph::native_doc::{
    ExpressionRefusal, LineEnding, NativeTemplateRefusal, PrintOptions, native_template_document,
    print,
};
use vize_l0::Allocator;

#[test]
fn genuine_selected_static_and_computed_members_keep_complete_outputs_and_numeric_dots() {
    for (content, expected) in [
        ("obj.key", "obj . key"),
        ("obj . default", "obj . default"),
        ("日本.\\u0061", "日本 . \\u0061"),
        ("obj[key]", "obj [ key ]"),
        ("obj['x\\x20y']", "obj [ 'x\\x20y' ]"),
        ("obj[-1]", "obj [ - 1 ]"),
        ("obj[a+b]", "obj [ a + b ]"),
        ("obj.a[b].default", "obj . a [ b ] . default"),
        ("(a+b).key", "(a + b) . key"),
        ("1 .value", "1 . value"),
        ("1..value", "1. . value"),
        ("0xCA_FE.value", "0xCA_FE . value"),
        ("obj&#46;key", "obj &#46; key"),
        ("obj&#91;key&#93;", "obj &#91; key &#93;"),
        ("obj&#32;&#46;&#9;key", "obj&#32;&#46;&#9;key"),
        ("obj/*a*/./*b*/key", "obj/*a*/./*b*/key"),
        ("obj/*a*/[/*b*/key/*c*/]", "obj/*a*/[/*b*/key/*c*/]"),
    ] {
        for script in ["", "<script setup lang=ts>let a=1</script>"] {
            let source = vize_l0::cstr!(
                "<!--前--><template><p>{}{content}{}</p></template>{script}",
                "{{",
                "}}"
            );
            assert_eq!(
                format(&source, PrintOptions::default()),
                vize_l0::cstr!("<p>{} {expected} {}</p>", "{{", "}}"),
                "{source}"
            );
        }
        assert_preserved(content);
    }
    for content in [
        "obj /*原\r\n*/ . key",
        "obj //before\n . key",
        "obj &#47;*encoded*&#47; [key]",
        "obj[a&#32;+&#9;b]",
    ] {
        assert_preserved(content);
    }
    assert_eq!(
        format(
            "<template>{{obj[a+b]}}</template>",
            PrintOptions {
                width: 0,
                ..PrintOptions::default()
            }
        ),
        "{{\n  obj [ a +\n    b ]\n}}"
    );
    assert_eq!(
        format(
            "<template><p>{{obj /*原\r\n*/ . key}}</p></template>",
            PrintOptions::default()
        ),
        "<p>{{\n    obj /*原\r\n*/ . key\n  }}</p>"
    );
}

#[test]
fn selected_member_documents_keep_original_owner_ast_comments_map_and_content_storage() {
    let arena = Allocator::default();
    let source = "<!--前--><template><p>{{日本&#46; /*原*/ key[&#45;1]}}</p></template>";
    let owner = selected(&arena, source);
    let original = operands(&owner);
    let operand = original.first().unwrap();
    let syntax = operand.syntax();
    let ast = core::ptr::from_ref(syntax.expression().unwrap());
    let comments = syntax
        .comments()
        .map(|comment| {
            (
                comment.decoded_span().unwrap(),
                comment.authored_span().unwrap(),
                comment.text().unwrap().as_ptr(),
            )
        })
        .collect::<std::vec::Vec<_>>();
    let map = syntax.source().decode_map().unwrap().segments();
    let raw = operand.raw_content().as_ptr();
    let refs = original.iter().collect::<std::vec::Vec<_>>();
    let document = native_template_document(&owner, &refs, &arena).unwrap();
    assert_eq!(
        print(document.document(), &PrintOptions::default()),
        "<p>{{ 日本 &#46; /*原*/ key [ &#45; 1 ] }}</p>"
    );
    for width in [0, 1, 7, 80] {
        for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
            print(
                document.document(),
                &PrintOptions {
                    width,
                    line_ending,
                    ..PrintOptions::default()
                },
            );
            assert!(core::ptr::eq(document.original(), &owner));
            assert!(core::ptr::eq(
                *document.operands().first().unwrap(),
                operand
            ));
            assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), ast);
            assert_eq!(
                syntax
                    .comments()
                    .map(|comment| (
                        comment.decoded_span().unwrap(),
                        comment.authored_span().unwrap(),
                        comment.text().unwrap().as_ptr(),
                    ))
                    .collect::<std::vec::Vec<_>>(),
                comments
            );
            assert!(core::ptr::eq(
                syntax.source().decode_map().unwrap().segments(),
                map
            ));
            assert_eq!(operand.raw_content().as_ptr(), raw);
            assert_eq!(operand.content_span().slice(source), operand.raw_content());
        }
    }
}

#[test]
fn optional_private_chain_and_unsupported_member_descendants_refuse_whole_selected_documents() {
    for (content, ts_only) in [
        ("obj?.key", false),
        ("obj?.[key]", false),
        ("(obj?.key).x", false),
        ("obj.#key", false),
        ("call().key", false),
        ("obj[call()]", false),
        ("(obj as T).key", true),
        ("obj!.key", true),
    ] {
        for script in ["", "<script setup lang=ts>let a=1</script>"] {
            if ts_only && script.is_empty() {
                continue;
            }
            let arena = Allocator::default();
            let source = vize_l0::cstr!(
                "<template>{}/*原*/ {content}{}</template>{script}",
                "{{",
                "}}"
            );
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let operand = original.first().unwrap();
            let syntax = operand.syntax();
            assert!(syntax.admitted_expression().is_some(), "{source}");
            let ast = core::ptr::from_ref(syntax.expression().unwrap());
            let comments = syntax
                .comments()
                .map(|comment| comment.text().unwrap().as_ptr())
                .collect::<std::vec::Vec<_>>();
            let refs = original.iter().collect::<std::vec::Vec<_>>();
            assert!(
                matches!(
                    native_template_document(&owner, &refs, &arena),
                    Err(NativeTemplateRefusal::Expression {
                        refusal: ExpressionRefusal::UnsupportedNode { .. },
                        ..
                    })
                ),
                "{source}"
            );
            assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), ast);
            assert_eq!(
                syntax
                    .comments()
                    .map(|comment| comment.text().unwrap().as_ptr())
                    .collect::<std::vec::Vec<_>>(),
                comments
            );
            assert_eq!(operand.content_span().slice(&source), operand.raw_content());
        }
    }
}

#[test]
fn genuine_mixed_unary_member_depth_keeps_original_provider_and_document_limits() {
    let members = ".key".repeat(8);
    let content = vize_l0::cstr!("{}obj{members}", "!".repeat(8));
    let source = vize_l0::cstr!("<template>{}{content}{}</template>", "{{", "}}");
    assert_eq!(
        format(
            &source,
            PrintOptions {
                width: 200,
                ..PrintOptions::default()
            }
        ),
        vize_l0::cstr!(
            "{} {}obj{} {}",
            "{{",
            "! ".repeat(8),
            " . key".repeat(8),
            "}}"
        )
    );
    assert_preserved(&content);
    let arena = Allocator::default();
    let content = vize_l0::cstr!("{}obj{members}", "!".repeat(9));
    let source = vize_l0::cstr!("<template>{}{content}{}</template>", "{{", "}}");
    let owner = selected(&arena, &source);
    let original = operands(&owner);
    let operand = original.first().unwrap();
    let syntax = operand.syntax();
    assert!(syntax.admitted_expression().is_some());
    assert_eq!(syntax.hole(), None);
    let ast = core::ptr::from_ref(syntax.expression().unwrap());
    let refs = original.iter().collect::<std::vec::Vec<_>>();
    assert!(matches!(
        native_template_document(&owner, &refs, &arena),
        Err(NativeTemplateRefusal::Expression {
            refusal: ExpressionRefusal::DepthLimit { .. },
            ..
        })
    ));
    assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), ast);
    assert_eq!(operand.content_span().slice(&source), operand.raw_content());
}

#[test]
fn selected_members_keep_encoded_quote_internal_lf_boundary_and_fixed_points() {
    for (content, expected) in [
        ("obj[&#39;//x&#39;\n]", "obj [ &#39;//x&#39;\n]"),
        ("obj[&quot;//x&quot;\r\n]", "obj [ &quot;//x&quot;\r\n]"),
        ("&#39;//x&#39;\n.length", "&#39;//x&#39;\n. length"),
        ("(&#39;//x&#39;\n).length", "(&#39;//x&#39;\n) . length"),
        ("1&#46;\n.value", "1&#46;\n. value"),
        (
            "obj[&#39;//x&#39;\n /*原\r\n*/]",
            "obj [ &#39;//x&#39;\n /*原\r\n*/]",
        ),
    ] {
        for script in ["", "<script setup lang=ts>let a=1</script>"] {
            let source = vize_l0::cstr!(
                "<!--前--><template>{}{}{}</template>{script}",
                "{{",
                content,
                "}}"
            );
            for width in [0, 1, 7, 80, 200] {
                for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                    let generated = if line_ending == LineEnding::Lf {
                        "\n"
                    } else {
                        "\r\n"
                    };
                    let options = PrintOptions {
                        width,
                        line_ending,
                        ..PrintOptions::default()
                    };
                    let output = format(&source, options);
                    assert_eq!(
                        output,
                        vize_l0::cstr!("{}{generated}  {expected}{generated}{}", "{{", "}}"),
                        "{source}"
                    );
                    let replay = vize_l0::cstr!("<template>{output}</template>{script}");
                    assert_eq!(format(&replay, options), output, "{replay}");
                }
            }
        }
        assert_preserved(content);
    }
}

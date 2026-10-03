//! Whole selected-template unary output through genuine original operands.

use super::{format, operands, selected};
use vize_glyph::native_doc::{
    ExpressionRefusal, LineEnding, NativeTemplateRefusal, PrintOptions, native_template_document,
    print,
};
use vize_l0::Allocator;

#[test]
fn genuine_selected_unary_operators_keep_complete_output_and_token_boundaries() {
    for (content, expected) in [
        ("!ready", "! ready"),
        ("~bits", "~ bits"),
        ("+1", "+ 1"),
        ("-1", "- 1"),
        ("typeof value", "typeof value"),
        ("void 0", "void 0"),
        ("delete 0", "delete 0"),
        ("+ +a", "+ + a"),
        ("- -a", "- - a"),
        ("a+ +b", "a + + b"),
        ("a- -b", "a - - b"),
        ("-(a+b)", "- (a + b)"),
        ("(-a)**b", "(- a) ** b"),
        ("-(a**b)", "- (a ** b)"),
        ("!(a&amp;&amp;b)", "! (a &amp;&amp; b)"),
        ("&#33;ready", "&#33; ready"),
        ("&#43;&#32;&#43;a", "&#43;&#32;&#43; a"),
        ("!&#9;ready", "!&#9;ready"),
        ("! /*kept\r\n*/ ready", "! /*kept\r\n*/ ready"),
    ] {
        for script in ["", "<script setup lang=ts>let a=1</script>"] {
            let source =
                vize_l0::cstr!("<!--前--><template><p>{{{{{content}}}}}</p></template>{script}");
            assert_eq!(
                format(&source, PrintOptions::default()),
                vize_l0::cstr!("<p>{{{{ {expected} }}}}</p>"),
                "{source}"
            );
        }
    }
    assert_eq!(
        format(
            "<template>{{a+ +b}}</template>",
            PrintOptions {
                width: 0,
                ..PrintOptions::default()
            }
        ),
        "{{\n  a +\n    + b\n}}"
    );
}

#[test]
fn unary_selected_documents_keep_original_operand_and_ast_source_custody() {
    let arena = Allocator::default();
    let source = "<!--前--><template><p>{{&#33; /*原*/ (a&amp;&amp;ok)}}</p></template>";
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
fn update_await_and_unsupported_unary_operands_refuse_the_whole_selected_document() {
    for content in ["++a", "a++", "--a", "a--", "await a", "!call()", "-obj.key"] {
        let arena = Allocator::default();
        let source = vize_l0::cstr!("<template>{{{{/*kept*/ {content}}}}}</template>");
        let owner = selected(&arena, &source);
        let original = operands(&owner);
        let syntax = original.first().unwrap().syntax();
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
        assert_eq!(
            original.first().unwrap().content_span().slice(&source),
            original.first().unwrap().raw_content()
        );
    }
}

#[test]
fn admitted_unary_depth_keeps_the_existing_whole_document_limit() {
    let supported = "!".repeat(16);
    let source = vize_l0::cstr!("<template>{{{{{supported}a}}}}}</template>");
    assert_eq!(
        format(&source, PrintOptions::default()),
        vize_l0::cstr!("{{{{ {}a }}}}", "! ".repeat(16))
    );
    let arena = Allocator::default();
    let prefixes = "!".repeat(17);
    let source = vize_l0::cstr!("<template>{{{{{prefixes}a}}}}}</template>");
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

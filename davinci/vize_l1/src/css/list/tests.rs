use crate::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    css::{StyleIssueCode, StyleSyntax},
};
use cssparser::Token;
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

#[test]
fn class_list_receipts_borrow_every_actual_original_token_end_once() {
    let source = "<!-- 雪🌸 -->\r\n<style scoped>/* original */ .雪,\r\n .b,.c { content:'雪'; }</style><template><p/></template>";
    let arena = Allocator::default();
    let descriptor = Vue.observe_descriptor(&arena, source, options());
    let original = descriptor.admitted().unwrap().styles().next().unwrap();
    let syntax = StyleSyntax::observe(original);
    assert!(syntax.simple_class().is_err());
    let receipt = syntax.simple_class_list().unwrap();
    assert_eq!(receipt.class_count(), 3);
    assert!(core::ptr::eq(receipt.syntax(), &syntax));
    assert!(core::ptr::eq(syntax.source().root_source(), source));
    let expected =
        syntax.rule().unwrap().prelude().iter().filter_map(|token| {
            matches!(token.token(), Token::Ident(_)).then_some(token.span().end)
        });
    assert!(receipt.insertions().eq(expected));
    for (end, name) in receipt.insertions().zip(["雪", "b", "c"]) {
        assert_eq!(&source[end as usize - name.len()..end as usize], name);
        assert!(
            syntax
                .source()
                .contains_block_span(vize_l0::Span::new(end, end))
        );
    }
    let moved = syntax;
    assert_eq!(moved.simple_class_list().unwrap().class_count(), 3);
    assert!(core::ptr::eq(moved.source().root_source(), source));
}

#[test]
fn every_unproven_list_shape_retains_original_syntax_and_refuses_whole_receipt() {
    for css in [
        ".a{}",
        ".a,.b:hover{}",
        ".a,p{}",
        ".a,,.b{}",
        ".a,.b,{}",
        ".a,. b{}",
        ".a , .b{}",
        ".a/* comment */,.b{}",
        ".a,/* comment */.b{}",
        ".a,.b/* comment */{}",
        ".\\61,.b{}",
        ".a,.b{color:v-bind(x)}",
        ".a,.b{content:'v-bind(x)'}",
        ".a,.b{content:'v/**/-bind(x)'}",
        ".a,.b{color:red",
        "@charset 'utf-8';.a,.b{}",
        ".a,.b{} .c{}",
    ] {
        let source = alloc::format!("<style scoped>{css}</style><template><p/></template>");
        let arena = Allocator::default();
        let descriptor = Vue.observe_descriptor(&arena, &source, options());
        let syntax = StyleSyntax::observe(descriptor.admitted().unwrap().styles().next().unwrap());
        assert!(syntax.simple_class_list().is_err(), "{css}");
        assert!(core::ptr::eq(
            syntax.source().root_source(),
            source.as_str()
        ));
        assert_eq!(syntax.source().source(), css);
        if let Some(error) = syntax.parser_error() {
            assert_eq!(
                syntax.simple_class_list().unwrap_err(),
                syntax.issue().unwrap()
            );
            assert!(matches!(
                error.kind,
                cssparser::ParseErrorKind::Basic(_) | cssparser::ParseErrorKind::Custom(_)
            ));
        } else {
            assert!(syntax.rule().is_some());
            assert_eq!(
                syntax.simple_class_list().unwrap_err().code,
                StyleIssueCode::UnsupportedSelector
            );
        }
    }
}

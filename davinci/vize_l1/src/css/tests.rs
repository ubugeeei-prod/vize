use super::{StyleIssueCode, StyleSyntax};
use crate::{SurfaceParseOptions, container::Vue, container::vue::DescriptorOptions};
use cssparser::Token;
use vize_l0::{
    Allocator, Span,
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
fn original_css_parser_tokens_declarations_and_absolute_spans_survive_once() {
    let source = "<!-- 雪🌸 -->\r\n<template><p class='foo'>雪</p></template>\r\n<style scoped lang=css>/* hi */\r\n.\\66 oo { color: red; content: '雪🌸'; --size: 2px; }</style>";
    let arena = Allocator::default();
    let observation = Vue.observe_descriptor(&arena, source, options());
    let original = observation.admitted().unwrap().styles().next().unwrap();
    let syntax = StyleSyntax::observe(original);
    assert!(
        syntax.parser_error().is_none(),
        "{:?}",
        syntax.parser_error()
    );
    assert!(core::ptr::eq(
        syntax.source().source(),
        original.block().source()
    ));
    assert!(core::ptr::eq(syntax.source().root_source(), source));
    assert_eq!(syntax.container_index(), 1);
    let rule = syntax.rule().unwrap();
    assert_eq!(rule.declarations().len(), 3);
    assert_eq!(rule.declarations()[0].name(), "color");
    assert_eq!(rule.declarations()[2].name(), "--size");
    let name = rule
        .prelude()
        .iter()
        .find(|token| matches!(token.token(), Token::Ident(_)))
        .unwrap();
    assert!(matches!(name.token(), Token::Ident(value) if value == "foo"));
    assert_eq!(
        &source[name.span().start as usize..name.span().end as usize],
        "\\66 oo"
    );
    let receipt = syntax.simple_class().unwrap();
    assert!(core::ptr::eq(receipt.syntax(), &syntax));
    assert_eq!(receipt.insertion(), name.span().end);
    for token in rule.prelude().iter().chain(
        rule.declarations()
            .iter()
            .flat_map(|declaration| declaration.value()),
    ) {
        assert!(syntax.source().contains_block_span(token.span()));
        assert!(!source[token.span().start as usize..token.span().end as usize].is_empty());
    }
}

#[test]
fn selector_admission_reads_actual_tokens_instead_of_hiding_whitespace_or_shape() {
    for css in [
        ".a{}",
        ".雪 {}",
        ".a/**/{}",
        "./**/a{}",
        ".a {color:red!important}",
    ] {
        let source = alloc::format!("<template><p/></template><style scoped>{css}</style>");
        let arena = Allocator::default();
        let observation = Vue.observe_descriptor(&arena, &source, options());
        let syntax = StyleSyntax::observe(observation.admitted().unwrap().styles().next().unwrap());
        assert!(syntax.simple_class().is_ok(), "{css}: {:?}", syntax.issue());
    }
    for css in [
        ". a{}",
        ".a .b{}",
        ".a,.b{}",
        "p{}",
        ".a:hover{}",
        ":deep(.a){}",
        ".a[data-x]{}",
    ] {
        let source = alloc::format!("<template><p/></template><style scoped>{css}</style>");
        let arena = Allocator::default();
        let observation = Vue.observe_descriptor(&arena, &source, options());
        let syntax = StyleSyntax::observe(observation.admitted().unwrap().styles().next().unwrap());
        assert!(syntax.simple_class().is_err(), "{css}");
        assert!(syntax.rule().is_some());
        assert!(core::ptr::eq(
            syntax.source().root_source(),
            source.as_str()
        ));
    }
}

#[test]
fn real_parser_refusals_retain_partial_rule_tokens_and_original_complete_source() {
    for css in [
        "",
        ".a{} .b{}",
        "@media screen{.a{}}",
        "@charset 'utf-8';.a{}",
        ".a{color}",
        ".a{color:red",
        ".a{color:v-bind(color)}",
        ".a{color:rgb(1,2,3)}",
        ".a{& .b{color:red}}",
        ".a{content:'}",
        ".a{content:'x\\' }",
    ] {
        let source =
            alloc::format!("<!-- 雪 -->\r\n<template><p/></template><style scoped>{css}</style>");
        let arena = Allocator::default();
        let observation = Vue.observe_descriptor(&arena, &source, options());
        let syntax = StyleSyntax::observe(observation.admitted().unwrap().styles().next().unwrap());
        assert!(syntax.simple_class().is_err(), "{css}");
        assert!(syntax.parser_error().is_some(), "{css}");
        assert!(
            syntax
                .source()
                .contains_block_span(syntax.issue().unwrap().span)
        );
        assert!(core::ptr::eq(
            syntax.source().root_source(),
            source.as_str()
        ));
    }
    let source = "<template><p/></template><style scoped>.a{color:v\\2d bind(color)}</style>";
    let arena = Allocator::default();
    let observation = Vue.observe_descriptor(&arena, source, options());
    let syntax = StyleSyntax::observe(observation.admitted().unwrap().styles().next().unwrap());
    let token = syntax.rule().unwrap().declarations()[0]
        .value()
        .iter()
        .find(|token| matches!(token.token(), Token::Function(_)))
        .unwrap();
    assert!(matches!(token.token(), Token::Function(value) if value == "v-bind"));
    assert_eq!(
        syntax.issue().unwrap().code,
        StyleIssueCode::UnsupportedValue
    );
    assert_eq!(syntax.issue().unwrap().span, token.span());
    assert_eq!(
        &source[token.span().start as usize..token.span().end as usize],
        "v\\2d bind("
    );
}

#[test]
fn non_css_style_profile_is_retained_without_parsing_the_wrong_grammar() {
    let source = "<style scoped lang=scss>$x:red;.a{color:$x}</style><template><p/></template>";
    let arena = Allocator::default();
    let observation = Vue.observe_descriptor(&arena, source, options());
    let style = observation.admitted().unwrap().styles().next().unwrap();
    let syntax = StyleSyntax::observe(style);
    assert_eq!(
        syntax.issue().unwrap().code,
        StyleIssueCode::UnsupportedLanguage
    );
    assert_eq!(syntax.issue().unwrap().span, style.block().span());
    assert!(syntax.rule().is_none());
    assert!(syntax.parser_error().is_none());
    assert!(core::ptr::eq(syntax.source().root_source(), source));
    assert_eq!(
        Span::new(style.block().start(), style.block().end()),
        syntax.source().span()
    );
}

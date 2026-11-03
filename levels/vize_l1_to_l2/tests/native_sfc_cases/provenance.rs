use super::support::options;
use vize_l0::Allocator;
use vize_l1::embed::Lang;
use vize_l1_to_l2::native_file::lower_sfc_native;

#[test]
fn setup_only_ts_absolute_spans_and_exact_ast_scope_identity_survive() {
    let arena = Allocator::default();
    let source = "\r\n<template><div :title=\"value\">{{value}}</div></template>\r\n<script setup lang=ts>/* 日本語 */ const value = 1;</script>";
    let observed = lower_sfc_native(&arena, source, options());
    let native = observed.admitted().unwrap();
    let script = observed.scripts().first().unwrap();
    assert_eq!(script.container_index(), 1);
    let syntax = script.syntax().unwrap();
    assert_eq!(syntax.source().span(), script.block().span());
    assert!(core::ptr::eq(
        syntax.source().text(),
        script.block().source()
    ));
    assert_eq!(syntax.comments().count(), 1);
    assert!(syntax.diagnostics().next().is_none());
    let receipt = native.file().setup().unwrap();
    assert_eq!(receipt.span(), script.block().span());
    let scope = native
        .file()
        .file()
        .scopes()
        .iter()
        .find(|scope| scope.id == receipt.scope())
        .unwrap();
    assert_eq!(scope.span, script.block().span());
    let template = observed.template().unwrap();
    assert_eq!(template.lang(), Lang::Ts);
    assert_eq!(template.container_index(), 0);
    assert!(template.block().span().end <= scope.span.start);
    let produced = template.produced().unwrap();
    assert_eq!(produced.embeds.len(), 2);
    for embed in &produced.embeds {
        let resolution = native
            .file()
            .file()
            .expression(embed.node.unwrap())
            .unwrap();
        assert_eq!(resolution.scope(), Some(receipt.scope()));
        let table = resolution.table().unwrap();
        assert!(core::ptr::eq(
            table.expression().ast,
            embed.syntax.expression().unwrap()
        ));
        assert_eq!(table.expression().span, embed.syntax.source().span());
        assert_eq!(table.occurrences().len(), 1);
        let occurrence = table.occurrences().first().unwrap();
        let binding = native.file().file().binding(occurrence.binding).unwrap();
        assert!(resolution.accepts(binding));
        assert!(native.file().exposure(binding).is_some());
    }
}

#[test]
fn decoded_template_entities_and_unicode_keep_exact_whole_file_origin() {
    let arena = Allocator::default();
    let source = "\r\n<template><div :title=\"'a &amp; b'\">{{'日本語'}}</div></template>";
    let observed = lower_sfc_native(&arena, source, options());
    let native = observed.admitted().unwrap();
    let produced = observed.template().unwrap().produced().unwrap();
    assert_eq!(produced.embeds.len(), 2);
    for embed in &produced.embeds {
        let resolution = native
            .file()
            .file()
            .expression(embed.node.unwrap())
            .unwrap();
        let table = resolution.table().unwrap();
        assert!(table.occurrences().is_empty());
        assert!(core::ptr::eq(
            table.expression().ast,
            embed.syntax.expression().unwrap()
        ));
        let span = embed.syntax.source().span();
        assert_eq!(table.expression().span, span);
        assert!(source.get(span.start as usize..span.end as usize).is_some());
        assert!(span.start > 0);
    }
}

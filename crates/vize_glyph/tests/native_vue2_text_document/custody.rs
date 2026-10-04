//! Original complete source/AST/profile/map authority survives Doc/drop/unwind.

use super::*;
use vize_l0::{SourceRoot, Span};
use vize_l1::SurfaceChild;
use vize_l1::embed::{Grammar, Lang, Shape, syntax::NativeSyntax};

#[derive(Debug, PartialEq)]
struct Facts {
    syntax: *const (),
    root: *const (),
    stock: *const (),
    stock_comments: (*const (), usize),
    stock_diagnostics: (*const (), usize),
    authored: (*const u8, usize),
    content: (*const u8, usize),
    span: Span,
    grammar: Grammar,
    prefix: u32,
    profile: oxc_span::SourceType,
    options: std::string::String,
    legacy: bool,
    map: std::vec::Vec<(Span, Span, vize_l1::embed::source::DecodeSegmentKind)>,
    map_pointer: Option<*const vize_l1::embed::source::DecodeSegment>,
    comments: std::vec::Vec<(Span, *const u8, usize)>,
    diagnostics: std::vec::Vec<(*const u8, usize)>,
}
fn facts(original: &NativeSyntax<'_>) -> Facts {
    let view = original.borrow_expression().unwrap();
    let source = view.source();
    Facts {
        syntax: original as *const _ as *const (),
        root: view.expression() as *const _ as *const (),
        stock: view.admitted_expression().original() as *const _ as *const (),
        stock_comments: (
            view.admitted_expression().comments().as_ptr() as *const (),
            view.admitted_expression().comments().len(),
        ),
        stock_diagnostics: (
            view.admitted_expression().diagnostics().as_ptr() as *const (),
            view.admitted_expression().diagnostics().len(),
        ),
        authored: (
            source.authored_root().as_ptr(),
            source.authored_root().len(),
        ),
        content: (source.text().as_ptr(), source.text().len()),
        span: source.span(),
        grammar: view.grammar(),
        prefix: view.parser_prefix(),
        profile: view.source_type(),
        options: std::format!("{:?}", view.options()),
        legacy: view.has_legacy_literals(),
        map: source
            .decode_map()
            .map(|map| {
                map.segments()
                    .iter()
                    .map(|s| (s.decoded(), s.authored(), s.kind()))
                    .collect()
            })
            .unwrap_or_default(),
        map_pointer: source.decode_map().map(|map| map.segments().as_ptr()),
        comments: view
            .comments()
            .map(|c| {
                (
                    c.decoded_span().unwrap(),
                    c.text().unwrap().as_ptr(),
                    c.text().unwrap().len(),
                )
            })
            .collect(),
        diagnostics: view
            .diagnostics()
            .map(|d| (d.message().as_ptr(), d.message().len()))
            .collect(),
    }
}
fn chain_facts(owner: &surface::ComponentParse<'_>) -> std::vec::Vec<Facts> {
    owner
        .bindings()
        .iter()
        .flat_map(|b| {
            let chain = b.admitted().unwrap();
            core::iter::once(chain.base()).chain(chain.filters().iter().flat_map(|f| f.arguments()))
        })
        .map(facts)
        .collect()
}

#[test]
fn nonzero_unicode_nested_owner_short_borrows_and_unwind_keep_all_original_facts() {
    let arena = Allocator::default();
    let doc_arena = Allocator::default();
    let root = "前🙂<template>甲<div>{{ &#38634; + 1 | upper () | 后缀(2+3, '後',) }}</div>乙</template>後";
    let start = root.find("甲").unwrap();
    let end = root.find("</template>").unwrap();
    let block = SourceRoot::new(root)
        .unwrap()
        .block(&root[start..end], start as u32)
        .unwrap();
    let initial = surface::parse_component_block(&arena, block);
    let owner = std::boxed::Box::new(initial); // Move only before any returned borrow.
    assert!(owner.errors().is_empty());
    assert!(owner.unsupported().is_empty());
    let before = chain_facts(&owner);
    assert_eq!(before.len(), 3);
    assert!(before[0].map_pointer.is_some());
    for fact in &before {
        assert_eq!(
            fact.grammar,
            Grammar {
                shape: Shape::Expr,
                lang: Lang::Js
            }
        );
        assert_eq!(fact.authored, (root.as_ptr(), root.len()));
    }
    for _ in 0..3 {
        let element = owner.children().nth(1).unwrap();
        let parent = element.surface();
        let child = element.children().unwrap().next().unwrap();
        let node = child.surface();
        let view = owner.text_for(child).unwrap();
        let chain = view.chain() as *const _;
        let binding = view.binding() as *const _;
        let document = vue2_text_document(view, &doc_arena).unwrap();
        assert!(core::ptr::eq(
            document.original().child().component(),
            owner.as_ref()
        ));
        assert_eq!(document.original().child().ordinal(), 0);
        let SurfaceChild::Element(element) = parent else {
            panic!("original parent")
        };
        assert!(core::ptr::eq(
            document.original().child().parent_element().unwrap(),
            &**element
        ));
        assert!(core::ptr::eq(document.original().child().surface(), node));
        assert_eq!(document.original().chain() as *const _, chain);
        assert_eq!(document.original().binding() as *const _, binding);
        assert_eq!(
            print(document.document(), &PrintOptions::default()),
            "{{ &#38634; + 1 | upper () | 后缀(2 + 3, '後',) }}"
        );
        assert_eq!(chain_facts(&owner), before);
    }
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let child = owner
            .children()
            .nth(1)
            .unwrap()
            .children()
            .unwrap()
            .next()
            .unwrap();
        let document = vue2_text_document(owner.text_for(child).unwrap(), &doc_arena).unwrap();
        assert_eq!(document.original().child().component().block(), block);
        panic!("law-only unwind");
    }));
    assert!(caught.is_err());
    assert_eq!(chain_facts(&owner), before);
    drop(owner); // Ordinary owned observations are dropped after every view.
}

#[test]
fn same_buffer_equal_heap_foreign_roots_and_distinct_occurrences_use_real_l1_seam() {
    let source = std::string::String::from("{{ a+b }}{{ a+b }}");
    let copied = source.clone();
    assert_ne!(source.as_ptr(), copied.as_ptr());
    let arena = Allocator::default();
    let owner = surface::parse_component(&arena, &source).unwrap();
    for foreign in [
        surface::parse_component(&arena, &source).unwrap(),
        surface::parse_component(&arena, &copied).unwrap(),
    ] {
        let before = chain_facts(&foreign);
        for child in foreign.children() {
            assert_eq!(
                owner.text_for(child).unwrap_err(),
                surface::TextRefusal::ForeignComponent
            );
        }
        assert_eq!(chain_facts(&foreign), before);
    }
    let before = chain_facts(&owner);
    let first = vue2_text_document(
        owner.text_for(owner.children().next().unwrap()).unwrap(),
        &arena,
    )
    .unwrap();
    let second = vue2_text_document(
        owner.text_for(owner.children().nth(1).unwrap()).unwrap(),
        &arena,
    )
    .unwrap();
    assert_eq!(
        (
            first.original().child().ordinal(),
            second.original().child().ordinal()
        ),
        (0, 1)
    );
    assert_ne!(
        first.original().binding().span(),
        second.original().binding().span()
    );
    assert!(!core::ptr::eq(
        first.original().chain().base(),
        second.original().chain().base()
    ));
    assert_eq!(
        print(first.document(), &PrintOptions::default()),
        "{{ a + b }}"
    );
    assert_eq!(
        print(second.document(), &PrintOptions::default()),
        "{{ a + b }}"
    );
    assert_eq!(chain_facts(&owner), before);
}

#[test]
fn original_module_profile_legacy_fact_and_literal_escapes_stay_syntax_only() {
    for (source, legacy) in [
        ("{{ 010 | upper () }}", true),
        ("{{ '\\x41' | upper () }}", false),
    ] {
        let arena = Allocator::default();
        let owner = surface::parse_component(&arena, source).unwrap();
        let before = chain_facts(&owner);
        assert_eq!(before[0].legacy, legacy);
        assert!(before[0].profile.is_module());
        let document = vue2_text_document(
            owner.text_for(owner.children().next().unwrap()).unwrap(),
            &arena,
        )
        .unwrap();
        assert_eq!(print(document.document(), &PrintOptions::default()), source);
        assert_eq!(chain_facts(&owner), before);
    }
}

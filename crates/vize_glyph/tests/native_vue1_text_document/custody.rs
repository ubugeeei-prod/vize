//! Same original callback/source/stock AST remains owned through print/unwind.

use vize_glyph::native_doc::{PrintOptions, vue1_text_document};
use vize_l0::{Allocator, SourceRoot, Span, String};
use vize_l1::dialect::vue1::surface;
use vize_l1::embed::{Grammar, source::DecodeSegmentKind, syntax::NativeSyntax};

#[derive(Debug, PartialEq)]
struct Facts {
    syntax: *const (),
    root: *const (),
    stock: *const (),
    source: (*const u8, usize),
    content: (*const u8, usize),
    span: Span,
    grammar: Grammar,
    profile: oxc_span::SourceType,
    prefix: u32,
    options: std::string::String,
    legacy: bool,
    stock_comments: (*const (), usize),
    stock_diagnostics: (*const (), usize),
    map_pointer: Option<*const vize_l1::embed::source::DecodeSegment>,
    map: std::vec::Vec<(Span, Span, DecodeSegmentKind)>,
    comments: std::vec::Vec<(Span, Span, *const u8, usize)>,
    diagnostics: std::vec::Vec<(*const u8, usize)>,
}

fn facts(syntax: &NativeSyntax<'_>) -> Facts {
    let view = syntax.borrow_expression().unwrap();
    let source = view.source();
    Facts {
        syntax: syntax as *const _ as *const (),
        root: view.expression() as *const _ as *const (),
        stock: view.admitted_expression().original() as *const _ as *const (),
        source: (
            source.authored_root().as_ptr(),
            source.authored_root().len(),
        ),
        content: (source.text().as_ptr(), source.text().len()),
        span: source.span(),
        grammar: view.grammar(),
        profile: view.source_type(),
        prefix: view.parser_prefix(),
        options: std::format!("{:?}", view.options()),
        legacy: view.has_legacy_literals(),
        stock_comments: (
            view.admitted_expression().comments().as_ptr() as *const (),
            view.admitted_expression().comments().len(),
        ),
        stock_diagnostics: (
            view.admitted_expression().diagnostics().as_ptr() as *const (),
            view.admitted_expression().diagnostics().len(),
        ),
        map_pointer: source.decode_map().map(|map| map.segments().as_ptr()),
        map: source
            .decode_map()
            .map(|map| {
                map.segments()
                    .iter()
                    .map(|segment| (segment.decoded(), segment.authored(), segment.kind()))
                    .collect()
            })
            .unwrap_or_default(),
        comments: view
            .comments()
            .map(|comment| {
                let text = comment.text().unwrap();
                (
                    comment.decoded_span().unwrap(),
                    comment.authored_span().unwrap(),
                    text.as_ptr(),
                    text.len(),
                )
            })
            .collect(),
        diagnostics: view
            .diagnostics()
            .map(|diagnostic| (diagnostic.message().as_ptr(), diagnostic.message().len()))
            .collect(),
    }
}

#[test]
fn nonzero_unicode_nested_original_callback_keeps_every_owner_fact_after_unwind() {
    let source = String::from("前<template><p>雪{{ /*kept*/ a&#43;b }}</p></template>後");
    let start = source.find("<p>").unwrap();
    let end = source.find("</template>").unwrap();
    let block = SourceRoot::new(&source)
        .unwrap()
        .block(&source[start..end], start as u32)
        .unwrap();
    let arena = Allocator::default();
    let doc_arena = Allocator::default();
    let owner = surface::parse_component_block(&arena, block);
    let syntax = owner.bindings()[0].syntax().unwrap();
    let before = facts(syntax);
    assert_eq!(before.comments.len(), 1);
    assert!(
        before
            .map
            .iter()
            .any(|row| row.2 == DecodeSegmentKind::Entity)
    );
    assert!(before.profile.is_module());
    assert!(!before.profile.is_typescript());
    let parent = owner.children().next().unwrap();
    let child = parent.children().unwrap().nth(1).unwrap();
    let surface = core::ptr::from_ref(child.surface());
    let parent_element = child.parent_element().unwrap();
    let document = vue1_text_document(owner.text_for(child).unwrap(), &doc_arena).unwrap();
    assert!(core::ptr::eq(
        document.original().child().component(),
        &owner
    ));
    assert!(core::ptr::eq(
        document.original().binding(),
        &owner.bindings()[0]
    ));
    assert_eq!(
        core::ptr::from_ref(document.original().child().surface()),
        surface
    );
    assert!(core::ptr::eq(
        document.original().child().parent_element().unwrap(),
        parent_element
    ));
    assert_eq!(document.original().child().ordinal(), 1);
    assert_eq!(document.original().child().component().block(), block);
    assert_eq!(
        document.print(&PrintOptions::default()).unwrap(),
        "{{ /*kept*/ a &#43; b }}"
    );
    assert_eq!(facts(syntax), before);
    drop(document);
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let child = owner
            .children()
            .next()
            .unwrap()
            .children()
            .unwrap()
            .nth(1)
            .unwrap();
        let document = vue1_text_document(owner.text_for(child).unwrap(), &doc_arena).unwrap();
        assert!(core::ptr::eq(
            document.original().expression(),
            syntax.expression().unwrap()
        ));
        panic!("law-only unwind");
    }));
    assert!(caught.is_err());
    assert_eq!(facts(syntax), before);
    assert!(core::mem::needs_drop::<surface::ComponentParse<'_>>());
    drop(owner);
}

#[test]
fn repeated_occurrences_and_foreign_same_buffer_parses_cannot_replace_original_views() {
    let source = String::from("{{ left+right }}{{ left+right }}");
    let copied = String::from(source.as_str());
    assert_ne!(source.as_ptr(), copied.as_ptr());
    let arena = Allocator::default();
    let owner = surface::parse_component(&arena, &source).unwrap();
    for foreign in [
        surface::parse_component(&arena, &source).unwrap(),
        surface::parse_component(&arena, &copied).unwrap(),
    ] {
        let before = facts(foreign.bindings()[0].syntax().unwrap());
        assert_eq!(
            owner
                .text_for(foreign.children().next().unwrap())
                .unwrap_err(),
            surface::TextRefusal::ForeignComponent
        );
        assert_eq!(facts(foreign.bindings()[0].syntax().unwrap()), before);
    }
    let first = vue1_text_document(
        owner.text_for(owner.children().next().unwrap()).unwrap(),
        &arena,
    )
    .unwrap();
    let second = vue1_text_document(
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
        first.original().expression(),
        second.original().expression()
    ));
    assert_eq!(
        first.print(&PrintOptions::default()).unwrap(),
        second.print(&PrintOptions::default()).unwrap()
    );
}

#[test]
fn moves_before_borrow_and_owned_print_after_drop_preserve_original_authority() {
    let printed = {
        let arena = Allocator::default();
        let source = String::from("{{ value }}");
        let owner = surface::parse_component(&arena, &source).unwrap();
        let root = core::ptr::from_ref(owner.bindings()[0].syntax().unwrap().expression().unwrap());
        let mut parked = vec![owner];
        parked.reserve(64);
        let moved = parked.pop().unwrap();
        let document = vue1_text_document(
            moved.text_for(moved.children().next().unwrap()).unwrap(),
            &arena,
        )
        .unwrap();
        assert_eq!(core::ptr::from_ref(document.original().expression()), root);
        document.print(&PrintOptions::default()).unwrap()
    };
    assert_eq!(printed, "{{ value }}");
}

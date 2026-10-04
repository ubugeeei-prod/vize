use alloc::vec::Vec;
use oxc_span::GetSpan;
use vize_l0::{Allocator, SourceRoot, Span};

use crate::dialect::{LegacyVueVersion, vue1, vue2};
use crate::embed::{Grammar, Lang, Shape};
use crate::render::check_fidelity;

#[test]
fn vue2_original_child_chain_borrows_every_existing_operand_without_storage_or_source_changes() {
    let arena = Allocator::default();
    let root = "前🙂<template>{{ &#160;&#38634; | upper () | 后缀(1 + 2, '後',) }}</template>後";
    let start = root.find("{{").unwrap();
    let end = root.find("</template>").unwrap();
    let block = SourceRoot::new(root)
        .unwrap()
        .block(&root[start..end], start as u32)
        .unwrap();
    let owner = vue2::surface::parse_component_with_authored_block(&arena, block);
    let foreign = vue2::surface::parse_component_block(&arena, block);
    assert_eq!(owner.version(), LegacyVueVersion::V2);
    assert_eq!(check_fidelity(owner.tree()), Ok(()));
    assert_eq!(owner.bindings().len(), 1);
    assert!(owner.errors().is_empty());
    assert!(owner.unsupported().is_empty());
    assert_eq!(
        owner
            .text_for(foreign.children().next().unwrap())
            .unwrap_err(),
        vue2::surface::TextRefusal::ForeignComponent
    );
    let child = owner.children().next().unwrap();
    let original_surface = core::ptr::from_ref(child.surface());
    let text = owner.text_for(child).unwrap();
    assert!(core::ptr::eq(text.binding(), &owner.bindings()[0]));
    let chain = text.chain();
    let chain_pointer = core::ptr::from_ref(chain);
    let filters_pointer = chain.filters().as_ptr();
    let before = arena.allocated_bytes();
    let base = chain.base().borrow_expression().unwrap();
    assert!(core::ptr::eq(base.original(), chain.base()));
    assert_eq!(base.source().text(), "雪");
    assert!(base.source().decode_map().is_some());
    assert_eq!(
        base.authored_span(base.expression().span())
            .unwrap()
            .slice(root),
        "&#38634;"
    );
    assert!(core::ptr::eq(base.source().authored_root(), root));
    assert_eq!(
        base.grammar(),
        Grammar {
            shape: Shape::Expr,
            lang: Lang::Js
        }
    );
    assert!(core::ptr::eq(
        base.expression(),
        chain.base().expression().unwrap()
    ));
    let names: Vec<_> = chain
        .filters()
        .iter()
        .map(|filter| filter.name().text())
        .collect();
    assert_eq!(names, ["upper ", "后缀"]);
    assert!(chain.filters()[0].arguments().is_empty());
    let arguments = chain.filters()[1].arguments();
    assert_eq!(arguments.len(), 2);
    let sources: Vec<_> = arguments
        .iter()
        .map(|argument| argument.source().text())
        .collect();
    assert_eq!(sources, ["1 + 2", "'後'"]);
    for argument in arguments {
        let view = argument.borrow_expression().unwrap();
        assert!(core::ptr::eq(view.original(), argument));
        assert!(core::ptr::eq(
            view.expression(),
            argument.expression().unwrap()
        ));
        assert!(core::ptr::eq(view.source().authored_root(), root));
        assert_eq!(
            view.authored_span(view.expression().span())
                .unwrap()
                .slice(root),
            argument.source().text()
        );
        assert_eq!(view.comments().count(), 0);
        assert_eq!(view.diagnostics().count(), 0);
    }
    assert_eq!(
        core::ptr::from_ref(text.child().surface()),
        original_surface
    );
    assert_eq!(core::ptr::from_ref(text.chain()), chain_pointer);
    assert_eq!(chain.filters().as_ptr(), filters_pointer);
    assert_eq!(
        text.binding().raw_content(),
        " &#160;&#38634; | upper () | 后缀(1 + 2, '後',) "
    );
    assert_eq!(
        text.binding().span().slice(root),
        "{{ &#160;&#38634; | upper () | 后缀(1 + 2, '後',) }}"
    );
    assert_eq!(arena.allocated_bytes(), before);
    assert_eq!(check_fidelity(owner.tree()), Ok(()));
}

#[test]
fn vue1_original_escaped_child_borrows_same_root_comments_and_complete_maps_without_modern_carrier()
{
    let arena = Allocator::default();
    let root = "前🍣<template>雪{{ /*keep*/ msg &amp;&amp; 条件 }}</template>後";
    let start = root.find('雪').unwrap();
    let end = root.find("</template>").unwrap();
    let block = SourceRoot::new(root)
        .unwrap()
        .block(&root[start..end], start as u32)
        .unwrap();
    let owner = vue1::surface::parse_component_with_authored_block(&arena, block);
    let foreign = vue1::surface::parse_component_block(&arena, block);
    assert_eq!(owner.version(), LegacyVueVersion::V1);
    assert_eq!(check_fidelity(owner.tree()), Ok(()));
    assert_eq!(owner.bindings().len(), 1);
    assert_eq!(
        owner
            .text_for(foreign.children().nth(1).unwrap())
            .unwrap_err(),
        vue1::surface::TextRefusal::ForeignComponent
    );
    let text = owner.text_for(owner.children().nth(1).unwrap()).unwrap();
    let original = text.binding().syntax().unwrap();
    let map_pointer = original.source().decode_map().unwrap().segments().as_ptr();
    let before = arena.allocated_bytes();
    let view = original.borrow_expression().unwrap();
    assert!(core::ptr::eq(view.original(), original));
    assert!(core::ptr::eq(view.expression(), text.expression()));
    assert!(core::ptr::eq(view.source().authored_root(), root));
    assert_eq!(view.source().text(), "/*keep*/ msg && 条件");
    assert_eq!(
        view.source().decode_map().unwrap().segments().as_ptr(),
        map_pointer
    );
    assert_eq!(
        view.authored_span(view.expression().span())
            .unwrap()
            .slice(root),
        "msg &amp;&amp; 条件"
    );
    let comments: Vec<_> = view.comments().collect();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].text(), Ok("/*keep*/"));
    assert_eq!(comments[0].authored_span().unwrap().slice(root), "/*keep*/");
    assert_eq!(view.diagnostics().count(), 0);
    assert_eq!(
        view.grammar(),
        Grammar {
            shape: Shape::Expr,
            lang: Lang::Js
        }
    );
    assert_eq!(view.parser_prefix(), 2);
    assert_eq!(
        text.binding().content_span().slice(root),
        " /*keep*/ msg &amp;&amp; 条件 "
    );
    assert_eq!(
        text.binding().span().slice(root),
        "{{ /*keep*/ msg &amp;&amp; 条件 }}"
    );
    assert_eq!(arena.allocated_bytes(), before);
    assert_eq!(check_fidelity(owner.tree()), Ok(()));
    assert_eq!(
        view.decoded_span(oxc_span::Span::new(0, 2)),
        Err(crate::embed::SourceError::InvalidDecodedSpan)
    );
    assert_eq!(
        view.source().span().start,
        text.binding().content_span().start + 1
    );
    assert!(view.source().span().end < text.binding().content_span().end);
    assert_eq!(
        view.decoded_span(oxc_span::Span::new(2, 10)),
        Ok(Span::new(0, 8))
    );
}

use super::*;
use crate::dialect::vue2::surface::{
    ComponentParse, TextChildren, parse_component, parse_component_with_authored_block,
};
use vize_l0::{Allocator, SourceRoot, String};

type Record<'a> = (
    Option<*const Element<'a>>,
    usize,
    *const SurfaceChild<'a>,
    String,
    Result<String, TextRefusal>,
);

fn before<'o, 'a>(
    component: &'o ComponentParse<'a>,
    children: TextChildren<'o, 'a>,
    records: &mut alloc::vec::Vec<Record<'a>>,
) {
    for child in children {
        let view = component.text_for(child.reborrow()).map(|view| {
            let filters: alloc::vec::Vec<_> = view
                .chain()
                .filters()
                .iter()
                .map(|filter| {
                    (
                        filter.span(),
                        filter.name(),
                        filter
                            .arguments()
                            .iter()
                            .map(|arg| {
                                (
                                    arg.grammar(),
                                    arg.source_type(),
                                    arg.source(),
                                    arg.hole(),
                                    vize_l0::cstr!("{:?}", arg.expression()),
                                )
                            })
                            .collect::<alloc::vec::Vec<_>>(),
                    )
                })
                .collect();
            vize_l0::cstr!(
                "{:?}|{}|{:?}|{:?}|{filters:?}",
                view.binding().span(),
                view.binding().raw_content(),
                view.chain().base().source(),
                view.chain().base().expression()
            )
        });
        records.push((
            child.parent_element().map(core::ptr::from_ref),
            child.ordinal(),
            core::ptr::from_ref(child.surface()),
            vize_l0::cstr!("{:?}", child.surface()),
            view,
        ));
        if let Some(children) = child.children() {
            before(component, children, records);
        }
    }
}

fn after<'o, 'a>(
    component: &'o ComponentExpressionPool<'a>,
    children: RetainedTextChildren<'o, 'a>,
    records: &mut alloc::vec::Vec<Record<'a>>,
) {
    for child in children {
        let view = component
            .text_for(child.reborrow())
            .map(|view| {
                let filters: alloc::vec::Vec<_> = view
                    .chain()
                    .filters()
                    .iter()
                    .map(|filter| {
                        (
                            filter.span(),
                            filter.name(),
                            filter
                                .arguments()
                                .iter()
                                .map(|arg| {
                                    (
                                        arg.grammar(),
                                        arg.source_type(),
                                        arg.source(),
                                        arg.hole(),
                                        vize_l0::cstr!("{:?}", arg.expression()),
                                    )
                                })
                                .collect::<alloc::vec::Vec<_>>(),
                        )
                    })
                    .collect();
                assert!(core::ptr::eq(view.child().component(), component));
                vize_l0::cstr!(
                    "{:?}|{}|{:?}|{:?}|{filters:?}",
                    view.binding().span(),
                    view.binding().raw_content(),
                    view.chain().base().source(),
                    view.chain().base().expression()
                )
            })
            .map_err(|refusal| match refusal {
                RetainedTextRefusal::Body(refusal) => refusal,
                RetainedTextRefusal::ExpressionHandoff => {
                    panic!("current sealed producer emits Expr only")
                }
            });
        records.push((
            child.parent_element().map(core::ptr::from_ref),
            child.ordinal(),
            core::ptr::from_ref(child.surface()),
            vize_l0::cstr!("{:?}", child.surface()),
            view,
        ));
        if let Some(children) = child.children() {
            after(component, children, records);
        }
    }
}

#[test]
fn sealed_retained_children_keep_every_original_parent_ordinal_callback_and_refusal() {
    let arena = Allocator::default();
    for body in [
        "<i>{{ value | wrap(雪, &#x31;) }}{{ value | wrap(雪, &#x31;) }}</i><b>{{ last }}</b>",
        "{{b+}}",
        "{{ }}",
        "<a v-pre>{{x}}</a>",
        "{{x}}&#123;",
        "<a><span><a>{{x}}</a></span></a>",
        "{{x",
    ] {
        let source = vize_l0::cstr!("前🦀<template>{body}</template>後");
        let prefix = "前🦀<template>".len();
        let window = source.get(prefix..prefix + body.len()).unwrap();
        let block = SourceRoot::new(&source)
            .unwrap()
            .block(window, prefix as u32)
            .unwrap();
        let component = parse_component_with_authored_block(&arena, block);
        let mut original = alloc::vec::Vec::new();
        before(&component, component.children(), &mut original);
        let mut original_authored = alloc::vec::Vec::new();
        if let Some(children) = component.authored_children() {
            before(&component, children, &mut original_authored);
        }
        let pool = component.into_expression_pool();
        let allocated = arena.allocated_bytes();
        for _ in 0..3 {
            let mut retained = alloc::vec::Vec::new();
            after(&pool, pool.children(), &mut retained);
            assert_eq!(retained, original);
            let mut retained_authored = alloc::vec::Vec::new();
            if let Some(children) = pool.authored_children() {
                after(&pool, children, &mut retained_authored);
            }
            assert_eq!(retained_authored, original_authored);
        }
        assert_eq!(arena.allocated_bytes(), allocated);
    }
}

#[test]
fn original_same_buffer_equal_bytes_and_foreign_arena_cannot_attach_child_membership() {
    let arena = Allocator::default();
    let other_arena = Allocator::default();
    let source = String::from("{{value}}");
    let copy = String::from(source.as_str());
    let original = parse_component(&arena, &source)
        .unwrap()
        .into_expression_pool();
    let same_buffer = parse_component(&arena, &source)
        .unwrap()
        .into_expression_pool();
    let equal_bytes = parse_component(&arena, &copy)
        .unwrap()
        .into_expression_pool();
    let foreign_arena = parse_component(&other_arena, &source)
        .unwrap()
        .into_expression_pool();
    for foreign in [&same_buffer, &equal_bytes, &foreign_arena] {
        assert_eq!(
            original
                .text_for(foreign.children().next().unwrap())
                .unwrap_err(),
            RetainedTextRefusal::Body(TextRefusal::ForeignComponent)
        );
    }
    let view = original
        .text_for(original.children().next().unwrap())
        .unwrap();
    assert!(core::ptr::eq(
        view.binding(),
        original.bindings().first().unwrap()
    ));
    assert!(core::ptr::eq(
        view.child().surface(),
        original.tree().children.first().unwrap()
    ));
    assert!(core::ptr::eq(
        view.chain(),
        original.bindings().first().unwrap().admitted().unwrap()
    ));
    assert_eq!(view.child().ordinal(), 0);
    assert!(view.child().parent_element().is_none());
    assert_eq!(view.chain().base().source().text(), "value");
}

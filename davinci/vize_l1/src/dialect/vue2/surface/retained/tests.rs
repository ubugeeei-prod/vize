use crate::check_fidelity;
use crate::dialect::vue2::surface::parse_component_with_authored_block;
use vize_l0::{Allocator, SourceRoot};

#[test]
fn genuine_authored_repair_errors_and_unsupported_facts_move_with_the_whole_body() {
    let arena = Allocator::default();
    for (body, authored) in [
        (
            "<a @click='x' v-for='x in y'><span><a>{{x}}</a></span></a>",
            Some(true),
        ),
        ("<div>&#123;value&#125;</div>", Some(false)),
        ("<div>{{x}}</span>", None),
    ] {
        let source = vize_l0::cstr!("前🍣<template>{body}</template>後");
        let prefix = "前🍣<template>".len();
        let original_body = source.get(prefix..prefix + body.len()).unwrap();
        let block = SourceRoot::new(&source)
            .unwrap()
            .block(original_body, prefix as u32)
            .unwrap();
        let component = parse_component_with_authored_block(&arena, block);
        if let Some(expected) = authored {
            assert_eq!(component.authored().is_some(), expected);
        }
        let trees = vize_l0::cstr!("{:?}|{:?}", component.tree(), component.authored());
        let errors = vize_l0::cstr!("{:?}", component.errors());
        let unsupported = vize_l0::cstr!("{:?}", component.unsupported());
        let tree_pointer = component.tree().children.as_ptr();
        let authored_pointer = component.authored().map(|tree| tree.children.as_ptr());
        let errors_pointer = component.errors().as_ptr();
        let unsupported_pointer = component.unsupported().as_ptr();
        let pool = component.into_expression_pool();
        assert_eq!(pool.block(), block);
        assert!(core::ptr::eq(pool.block().root_source(), source.as_str()));
        assert!(core::ptr::eq(pool.tree().source, original_body));
        assert_eq!(
            vize_l0::cstr!("{:?}|{:?}", pool.tree(), pool.authored()),
            trees
        );
        assert_eq!(vize_l0::cstr!("{:?}", pool.errors()), errors);
        assert_eq!(vize_l0::cstr!("{:?}", pool.unsupported()), unsupported);
        assert_eq!(pool.tree().children.as_ptr(), tree_pointer);
        assert_eq!(
            pool.authored().map(|tree| tree.children.as_ptr()),
            authored_pointer
        );
        assert_eq!(pool.errors().as_ptr(), errors_pointer);
        assert_eq!(pool.unsupported().as_ptr(), unsupported_pointer);
        assert_eq!(check_fidelity(pool.tree()), Ok(()));
        if let Some(tree) = pool.authored() {
            assert_eq!(check_fidelity(tree), Ok(()));
        }
    }
}

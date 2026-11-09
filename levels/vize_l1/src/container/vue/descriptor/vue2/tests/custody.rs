use super::*;

#[test]
fn whole_unicode_source_keeps_original_slot_frame_and_filter_ast() {
    let prefix = "雪🦀\r\n<!--head-->\n";
    let open = "<template lang='html'>";
    let body = "{{ value | wrap(雪, &#x31;) }}";
    let source = String::from(
        "雪🦀\r\n<!--head-->\n<template lang='html'>{{ value | wrap(雪, &#x31;) }}</TeMpLaTe >尾",
    );
    let arena = Allocator::default();
    let owner = Vue.observe_vue2_descriptor(&arena, &source, options());
    let selected = owner.selected().unwrap();
    let original = owner.container().blocks.first().unwrap();
    assert!(owner.issues().is_empty() && owner.container().errors.is_empty());
    assert!(core::ptr::eq(owner.source(), source.as_str()));
    assert!(core::ptr::eq(selected.observation(), &owner));
    assert_eq!(selected.container_index(), 0);
    assert_eq!(
        selected.opening_name(),
        Span::new(prefix.len() as u32 + 1, prefix.len() as u32 + 9)
    );
    let close_start = (prefix.len() + open.len() + body.len()) as u32;
    assert_eq!(
        selected.closing_name(),
        Span::new(close_start + 2, close_start + 10)
    );
    assert_eq!(selected.closing_name().slice(&source), "TeMpLaTe");
    assert_eq!(selected.block().span(), original.content);
    assert!(core::ptr::eq(
        selected.block().root_source(),
        source.as_str()
    ));
    assert!(core::ptr::eq(
        selected.block().source(),
        original.content.slice(&source)
    ));
    let component = selected.component();
    assert!(core::ptr::eq(component, owner.component().unwrap()));
    assert_eq!(component.version(), crate::dialect::LegacyVueVersion::V2);
    assert!(core::ptr::eq(
        component.tree().source,
        selected.block().source()
    ));
    assert_eq!(check_fidelity(component.tree()), Ok(()));
    let text = component
        .text_for(component.children().next().unwrap())
        .unwrap();
    assert!(core::ptr::eq(
        text.binding(),
        component.bindings().first().unwrap()
    ));
    let chain = text.chain();
    let base = chain.base();
    assert_eq!(base.source().text(), "value");
    assert!(core::ptr::eq(
        base.source().authored_root(),
        source.as_str()
    ));
    assert!(core::ptr::eq(
        base.expression().unwrap(),
        base.borrow_expression().unwrap().expression()
    ));
    assert_eq!(base.comments().count(), 0);
    assert_eq!(base.diagnostics().count(), 0);
    assert_eq!(base.source_type(), oxc_span::SourceType::mjs());
    assert_eq!(
        base.grammar(),
        crate::embed::Grammar {
            shape: crate::embed::Shape::Expr,
            lang: crate::embed::Lang::Js,
        }
    );
    assert_eq!(
        base.borrow_expression().unwrap().options(),
        oxc_parser::ParseOptions::default()
    );
    let [filter] = chain.filters() else {
        panic!("whole original filter");
    };
    let [first, second] = filter.arguments() else {
        panic!("two original arguments");
    };
    assert_eq!(filter.name().text(), "wrap");
    assert_eq!((first.source().text(), second.source().text()), ("雪", "1"));
    let authored_entity = source.find("&#x31;").unwrap() as u32;
    assert_eq!(
        second.source().span(),
        Span::new(authored_entity, authored_entity + 6)
    );
    assert_eq!(
        second.source().authored_span(Span::new(0, 1)),
        Ok(second.source().span())
    );
    let [segment] = second.source().decode_map().unwrap().segments() else {
        panic!("whole original entity map");
    };
    assert_eq!(segment.decoded(), Span::new(0, 1));
    assert_eq!(segment.authored(), second.source().span());
    assert_eq!(segment.kind(), crate::embed::DecodeSegmentKind::Entity);
    for syntax in [base, first, second] {
        assert!(syntax.hole().is_none());
        assert!(selected.block().contains_block_span(syntax.source().span()));
        assert!(core::ptr::eq(
            syntax.source().authored_root(),
            owner.source()
        ));
    }
}

#[test]
fn same_buffer_foreign_allocation_and_arena_cannot_pair_component_children() {
    let source = String::from("<template>{{ value | upper }}</template>");
    let copy = String::from(source.as_str());
    assert_eq!(source, copy);
    assert_ne!(source.as_ptr(), copy.as_ptr());
    let arena = Allocator::default();
    let other_arena = Allocator::default();
    let owner = Vue.observe_vue2_descriptor(&arena, &source, options());
    let reparse = Vue.observe_vue2_descriptor(&arena, &source, options());
    let copied = Vue.observe_vue2_descriptor(&arena, &copy, options());
    let foreign_arena = Vue.observe_vue2_descriptor(&other_arena, &source, options());
    let component = owner.selected().unwrap().component();
    for foreign in [&reparse, &copied, &foreign_arena] {
        let other = foreign.selected().unwrap().component();
        assert!(!core::ptr::eq(component, other));
        assert_eq!(
            component
                .text_for(other.children().next().unwrap())
                .unwrap_err(),
            TextRefusal::ForeignComponent
        );
    }
    let block = component.block();
    let detached = String::from(block.source());
    assert_eq!(
        owner.root().unwrap().block(&detached, block.start()),
        Err(vize_l0::SourceFrameError::BlockNotRootSlice)
    );
}

#[test]
fn direct_sibling_parent_and_ordinal_keep_the_actual_repeated_callbacks() {
    let arena = Allocator::default();
    let source = "<template><i>{{ value | upper }}{{ value | upper }}</i><b>{{ value | upper }}</b></template>";
    let owner = Vue.observe_vue2_descriptor(&arena, source, options());
    let component = owner.selected().unwrap().component();
    let mut roots = component.children();
    let first = roots.next().unwrap();
    let SurfaceChild::Element(parent) = first.surface() else {
        panic!("actual parent");
    };
    let mut children = first.children().unwrap();
    let a = component.text_for(children.next().unwrap()).unwrap();
    let b = component.text_for(children.next().unwrap()).unwrap();
    let c = component
        .text_for(roots.next().unwrap().children().unwrap().next().unwrap())
        .unwrap();
    assert!(children.next().is_none() && roots.next().is_none());
    assert_eq!(
        (
            a.child().ordinal(),
            b.child().ordinal(),
            c.child().ordinal()
        ),
        (0, 1, 0)
    );
    assert!(core::ptr::eq(
        a.child().parent_element().unwrap(),
        &**parent
    ));
    assert!(core::ptr::eq(
        b.child().parent_element().unwrap(),
        &**parent
    ));
    let [first_binding, second_binding, third_binding] = component.bindings() else {
        panic!("three original bindings");
    };
    for (view, binding) in [
        (&a, first_binding),
        (&b, second_binding),
        (&c, third_binding),
    ] {
        assert!(core::ptr::eq(view.binding(), binding));
        assert_eq!(view.chain().base().source().text(), "value");
    }
    assert_ne!(a.binding().span(), b.binding().span());
    assert!(!core::ptr::eq(
        a.chain().base().expression().unwrap(),
        b.chain().base().expression().unwrap()
    ));
}

#[test]
fn moved_normal_owner_keeps_original_ast_and_empty_or_static_body_custody() {
    fn move_owner<'a>(owner: Vue2DescriptorObservation<'a>) -> Vue2DescriptorObservation<'a> {
        owner
    }
    let arena = Allocator::default();
    let source = "<template>{{ value | upper }}</template>";
    let original = Vue.observe_vue2_descriptor(&arena, source, options());
    let ast = original
        .component()
        .unwrap()
        .bindings()
        .first()
        .unwrap()
        .chain()
        .unwrap()
        .base()
        .expression()
        .unwrap() as *const _;
    let moved = move_owner(original);
    let component = moved.selected().unwrap().component();
    let view = component
        .text_for(component.children().next().unwrap())
        .unwrap();
    assert_eq!(view.chain().base().expression().unwrap() as *const _, ast);
    drop(moved);
    for source in [
        "<template></template>",
        "<template>雪🦀 fixed<!--kept--></template>",
        "<template lang=html>fixed</template>",
        "<template lang=\"html\">fixed</template>",
    ] {
        let owner = Vue.observe_vue2_descriptor(&arena, source, options());
        let selected = owner.selected().unwrap();
        assert_eq!(check_fidelity(selected.component().tree()), Ok(()));
        assert!(selected.component().bindings().is_empty());
        assert_eq!(
            selected.block().source(),
            owner
                .container()
                .blocks
                .first()
                .unwrap()
                .content
                .slice(source)
        );
    }
}

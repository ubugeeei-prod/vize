use super::*;

#[test]
fn whole_unicode_source_keeps_original_slot_frame_callback_ast_and_prepared_map() {
    let prefix = "前🍣\n<!--head-->\n";
    let open = "<template>";
    let body = "<p>雪{{ /*keep*/ msg &amp;&amp; 条件 }}</p>";
    let source = vize_l0::cstr!("{prefix}{open}{body}</TeMpLaTe >後");
    let arena = Allocator::default();
    let owner = Vue.observe_vue1_descriptor(&arena, &source, options());
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
    assert_eq!(component.version(), crate::dialect::LegacyVueVersion::V1);
    assert_eq!(check_fidelity(component.tree()), Ok(()));
    assert!(core::ptr::eq(
        component.tree().source,
        selected.block().source()
    ));
    let root = component.children().next().unwrap();
    let mut children = root.children().unwrap();
    assert_eq!(children.next().unwrap().ordinal(), 0);
    let text = component.text_for(children.next().unwrap()).unwrap();
    assert!(core::ptr::eq(
        text.binding(),
        component.bindings().first().unwrap()
    ));
    assert_eq!(text.child().ordinal(), 1);
    assert_eq!(
        text.binding().span().slice(&source),
        "{{ /*keep*/ msg &amp;&amp; 条件 }}"
    );
    assert_eq!(
        text.binding().content_span().slice(&source),
        text.binding().raw_content()
    );
    let syntax = text.binding().syntax().unwrap();
    assert_eq!(syntax.source().text(), "/*keep*/ msg && 条件");
    assert!(core::ptr::eq(
        syntax.source().authored_root(),
        source.as_str()
    ));
    assert!(core::ptr::eq(
        syntax.expression().unwrap(),
        text.expression()
    ));
    assert!(core::ptr::eq(
        syntax.borrow_expression().unwrap().expression(),
        text.expression()
    ));
    assert_eq!(syntax.source_type(), oxc_span::SourceType::mjs());
    assert_eq!(
        syntax.grammar(),
        crate::embed::Grammar {
            shape: crate::embed::Shape::Expr,
            lang: crate::embed::Lang::Js
        }
    );
    assert_eq!(
        syntax.borrow_expression().unwrap().options(),
        oxc_parser::ParseOptions::default()
    );
    assert_eq!(syntax.comments().count(), 1);
    assert_eq!(
        syntax
            .comments()
            .next()
            .unwrap()
            .authored_span()
            .unwrap()
            .slice(&source),
        "/*keep*/"
    );
    assert_eq!(syntax.diagnostics().count(), 0);
    assert_eq!(
        syntax
            .authored_span(text.expression().span())
            .unwrap()
            .slice(&source),
        "msg &amp;&amp; 条件"
    );
    let map = syntax.source().decode_map().unwrap();
    let entities = map
        .segments()
        .iter()
        .filter(|segment| segment.kind() == crate::embed::DecodeSegmentKind::Entity);
    assert_eq!(entities.clone().count(), 2);
    for segment in entities {
        assert_eq!(segment.authored().slice(&source), "&amp;");
        assert_eq!(segment.decoded().slice(syntax.source().text()), "&");
    }
    assert!(selected.block().contains_block_span(syntax.source().span()));
    assert!(syntax.hole().is_none() && children.next().is_none());
}

#[test]
fn same_buffer_equal_bytes_and_foreign_arena_cannot_pair_original_children() {
    let source = String::from("<template>{{ value }}</template>");
    let copy = String::from(source.as_str());
    assert_eq!(source, copy);
    assert_ne!(source.as_ptr(), copy.as_ptr());
    let arena = Allocator::default();
    let other_arena = Allocator::default();
    let owner = Vue.observe_vue1_descriptor(&arena, &source, options());
    let reparse = Vue.observe_vue1_descriptor(&arena, &source, options());
    let copied = Vue.observe_vue1_descriptor(&arena, &copy, options());
    let foreign_arena = Vue.observe_vue1_descriptor(&other_arena, &source, options());
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
fn actual_repeated_callbacks_keep_direct_parent_ordinal_and_allocation_identity() {
    let arena = Allocator::default();
    let source = "<template><i>{{ value }}{{ value }}</i><b>{{ value }}</b></template>";
    let owner = Vue.observe_vue1_descriptor(&arena, source, options());
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
        assert_eq!(binding.source().unwrap().text(), "value");
        assert!(core::ptr::eq(
            view.expression(),
            binding.syntax().unwrap().expression().unwrap()
        ));
    }
    assert_ne!(a.binding().span(), b.binding().span());
    assert!(!core::ptr::eq(a.expression(), b.expression()));
}

#[test]
fn normal_owner_move_preserves_original_ast_and_empty_or_static_body_observation() {
    fn move_owner<'a>(owner: Vue1DescriptorObservation<'a>) -> Vue1DescriptorObservation<'a> {
        owner
    }
    let arena = Allocator::default();
    let source = "<template>{{ value }}</template>";
    let original = Vue.observe_vue1_descriptor(&arena, source, options());
    let ast = core::ptr::from_ref(
        original
            .component()
            .unwrap()
            .bindings()
            .first()
            .unwrap()
            .syntax()
            .unwrap()
            .expression()
            .unwrap(),
    );
    let moved = move_owner(original);
    let component = moved.selected().unwrap().component();
    let view = component
        .text_for(component.children().next().unwrap())
        .unwrap();
    assert_eq!(core::ptr::from_ref(view.expression()), ast);
    assert!(core::mem::needs_drop::<surface::ComponentParse<'_>>());
    drop(moved);
    for source in [
        "<template></template>",
        "<template>雪🦀 fixed<!--kept--></template>",
    ] {
        let owner = Vue.observe_vue1_descriptor(&arena, source, options());
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

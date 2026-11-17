use super::{Allocator, selected};
use crate::embed::Lang;
use crate::markup::NativeTemplateGrammar;
use oxc_span::GetSpan;

#[test]
fn same_root_child_observes_then_joins_after_pending_growth_without_iteration() {
    let arena = Allocator::default();
    let source = "<template>{{ msg &amp;&amp; ok }}<!--kept--></template>";
    let owner = selected(&arena, source).unwrap();
    let mut body = owner.children();
    let child = body.next().unwrap();
    let surface = child.surface();
    let operand = owner
        .observe_interpolation_expression(child.reborrow())
        .unwrap();
    let original = operand.syntax().expression().unwrap();
    let mut parked = alloc::vec::Vec::new();
    parked.push(operand);
    parked.reserve(32);
    let view = parked.first().unwrap().admitted_for(&owner, child).unwrap();
    assert!(core::ptr::eq(view.child().surface(), surface));
    assert!(view.child().parent_element().is_none());
    assert_eq!(view.child().ordinal(), 0);
    assert!(core::ptr::eq(
        view.expression().unwrap().expression(),
        original
    ));
    assert_eq!(
        view.operand().full_span().slice(source),
        "{{ msg &amp;&amp; ok }}"
    );
    assert_eq!(body.len(), 1);
    assert_eq!(body.next().unwrap().ordinal(), 1);
    assert_eq!(body.len(), 0);
}

#[test]
fn same_nested_child_keeps_parent_ts_profile_and_complete_observations() {
    let arena = Allocator::default();
    let source = "<template><p>prefix{{ /*kept*/ value as boolean }}</p></template><script setup lang=ts>const value=true</script>";
    let owner = selected(&arena, source).unwrap();
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let mut body = parent.children();
    assert_eq!(body.next().unwrap().ordinal(), 0);
    let child = body.next().unwrap();
    let surface = child.surface();
    let operand = owner
        .observe_interpolation_expression(child.reborrow())
        .unwrap();
    assert_eq!(operand.syntax().comments().count(), 1);
    assert_eq!(operand.syntax().diagnostics().count(), 0);
    assert_eq!(operand.syntax().grammar().lang, Lang::Ts);
    let original = operand.syntax().expression().unwrap();
    let view = operand.admitted_for(&owner, child).unwrap();
    assert!(core::ptr::eq(view.child().surface(), surface));
    assert!(core::ptr::eq(
        view.child().parent_element().unwrap(),
        parent.surface()
    ));
    assert_eq!(view.child().ordinal(), 1);
    assert!(core::ptr::eq(
        view.expression().unwrap().expression(),
        original
    ));
    assert_eq!(view.operand().raw_content(), " /*kept*/ value as boolean ");
    assert_eq!(body.len(), 0);
}

#[test]
fn original_root_unicode_entities_full_window_and_stock_pointer_are_retained() {
    let arena = Allocator::default();
    let source = "<!--前--><template>{{ \tmsg &amp;&amp; 条件 \n}}</template>";
    let owner = selected(&arena, source).unwrap();
    let operand = owner
        .observe_interpolation_expression(owner.children().next().unwrap())
        .unwrap();
    assert_eq!(
        operand.full_span().slice(source),
        "{{ \tmsg &amp;&amp; 条件 \n}}"
    );
    assert_eq!(operand.content_span().slice(source), operand.raw_content());
    assert_eq!(operand.raw_content(), " \tmsg &amp;&amp; 条件 \n");
    assert_eq!(operand.syntax().source().text(), "msg && 条件");
    assert!(operand.syntax().source().decode_map().is_some());
    assert!(core::ptr::eq(
        operand.syntax().source().authored_root(),
        source
    ));
    let original = operand.syntax().expression().unwrap();
    assert_eq!(
        operand
            .syntax()
            .authored_span(original.span())
            .unwrap()
            .slice(source),
        "msg &amp;&amp; 条件"
    );
    let view = operand
        .admitted_for(&owner, owner.children().next().unwrap())
        .unwrap();
    assert!(view.child().parent_element().is_none());
    assert_eq!(view.child().ordinal(), 0);
    assert!(core::ptr::eq(view.selected(), &owner));
    assert!(core::ptr::eq(
        view.expression().unwrap().expression(),
        original
    ));
}

#[test]
fn nested_original_parent_and_ordinal_survive_owner_moves_and_pending_growth() {
    let arena = Allocator::default();
    let source = "<template><p>prefix{{ value }}</p></template>";
    let owner = selected(&arena, source).unwrap();
    let operand = {
        let parent = owner.children().next().unwrap().into_element().unwrap();
        owner
            .observe_interpolation_expression(parent.children().nth(1).unwrap())
            .unwrap()
    };
    let original = operand.syntax().expression().unwrap();
    let mut parked = alloc::vec::Vec::new();
    parked.push(operand);
    parked.reserve(32);
    let moved = core::hint::black_box(owner);
    let parent = moved.children().next().unwrap().into_element().unwrap();
    let view = parked
        .first()
        .unwrap()
        .admitted_for(&moved, parent.children().nth(1).unwrap())
        .unwrap();
    assert_eq!(view.child().ordinal(), 1);
    assert!(core::ptr::eq(
        view.child().parent_element().unwrap(),
        parent.surface()
    ));
    assert!(core::ptr::eq(
        view.expression().unwrap().expression(),
        original
    ));
    assert_eq!(view.operand().raw_content(), " value ");
}

#[test]
fn setup_typescript_profile_is_selected_without_caller_language() {
    let arena = Allocator::default();
    let source = "<template>{{ value as boolean }}</template><script setup lang=ts>const value=true</script>";
    let owner = selected(&arena, source).unwrap();
    let operand = owner
        .observe_interpolation_expression(owner.children().next().unwrap())
        .unwrap();
    assert_eq!(owner.grammar(), NativeTemplateGrammar::TypeScriptModule);
    assert_eq!(operand.syntax().grammar().lang, Lang::Ts);
    assert!(operand.syntax().source_type().is_typescript());
    assert!(operand.syntax().source_type().is_module());
    assert!(
        operand
            .admitted_for(&owner, owner.children().next().unwrap())
            .is_some()
    );
    let js = selected(&arena, "<template>{{ value as boolean }}</template>").unwrap();
    let refused = js
        .observe_interpolation_expression(js.children().next().unwrap())
        .unwrap();
    assert!(refused.syntax().hole().is_some());
    assert!(refused.syntax().diagnostics().count() > 0);
    assert!(
        refused
            .admitted_for(&js, js.children().next().unwrap())
            .is_none()
    );
}

#[test]
fn original_comments_maps_and_root_remain_after_raw_transfer_and_selection_drop() {
    let arena = Allocator::default();
    let source = "<template>{{ /*kept*/ msg &amp;&amp; ok }}</template>";
    let (syntax, original) = {
        let owner = selected(&arena, source).unwrap();
        let operand = owner
            .observe_interpolation_expression(owner.children().next().unwrap())
            .unwrap();
        let original = operand.syntax().expression().unwrap();
        (operand.into_syntax(), original)
    };
    assert!(core::ptr::eq(syntax.expression().unwrap(), original));
    assert_eq!(syntax.comments().count(), 1);
    assert_eq!(syntax.diagnostics().count(), 0);
    assert_eq!(syntax.source().text(), "/*kept*/ msg && ok");
    assert!(syntax.source().decode_map().is_some());
    assert!(core::ptr::eq(syntax.source().authored_root(), source));
}

#[test]
fn normally_owned_hole_observations_survive_pending_callback_unwind() {
    extern crate std;
    let arena = Allocator::default();
    let source = "<template>{{ /*kept*/ value + }}</template>";
    let mut pending = None;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let owner = selected(&arena, source).unwrap();
        pending = Some(
            owner
                .observe_interpolation_expression(owner.children().next().unwrap())
                .unwrap(),
        );
        panic!("fixture callback unwind");
    }));
    assert!(result.is_err());
    let retained = pending.take().unwrap();
    assert_eq!(retained.raw_content(), " /*kept*/ value + ");
    assert_eq!(retained.full_span().slice(source), "{{ /*kept*/ value + }}");
    assert_eq!(retained.syntax().comments().count(), 1);
    assert!(retained.syntax().diagnostics().count() > 0);
    assert!(retained.syntax().hole().is_some());
    assert!(core::mem::needs_drop::<
        crate::markup::NativeInterpolationOperand<'_>,
    >());
}

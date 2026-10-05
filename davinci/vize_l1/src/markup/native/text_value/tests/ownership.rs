use super::*;

#[test]
fn one_actual_child_visit_parks_grows_and_short_joins_without_repreparation() {
    let arena = Allocator::default();
    let source = "<template><p>雪 &amp;lt;<!--kept-->tail</p></template>";
    let owner = selected(&arena, source);
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let mut parked = alloc::vec::Vec::new();
    let mut text_visits = 0;
    for child in parent.children() {
        if !matches!(child.surface(), SurfaceChild::Text(_)) {
            continue;
        }
        text_visits += 1;
        let value = owner.observe_text_value(child.reborrow()).unwrap();
        let original = child.surface();
        let decoded = value.source().text();
        let map = value.source().decode_map().map(|map| map.segments());
        parked.push(value);
        parked.reserve(32);
        let before = arena.allocated_bytes();
        let view = parked.last().unwrap().admitted_for(&owner, child).unwrap();
        assert!(core::ptr::eq(view.child().surface(), original));
        assert!(core::ptr::eq(view.observation().source().text(), decoded));
        match (map, view.observation().source().decode_map()) {
            (Some(map), Some(actual)) => assert!(core::ptr::eq(map, actual.segments())),
            (None, None) => {}
            _ => panic!("same complete preparation map"),
        }
        assert_eq!(arena.allocated_bytes(), before);
    }
    assert_eq!(text_visits, 2);
    let moved = core::hint::black_box(owner);
    let parent = moved.children().next().unwrap().into_element().unwrap();
    assert!(
        parked[0]
            .admitted_for(&moved, parent.children().next().unwrap())
            .is_some()
    );
    assert!(
        parked[1]
            .admitted_for(&moved, parent.children().nth(2).unwrap())
            .is_some()
    );
    assert_eq!(parked[0].source().text(), "雪 &lt;");
    assert_eq!(parked[1].source().text(), "tail");
    crate::check_fidelity(&moved.component().carrier().tree).unwrap();
}

#[test]
fn equal_sibling_foreign_parent_shared_source_reparse_and_copied_source_never_join() {
    let arena = Allocator::default();
    let source = "<template>same<!--root-->same<p>same<!--nested-->same</p><p>same</p></template>";
    let owner = selected(&arena, source);
    let root_value = owner
        .observe_text_value(owner.children().next().unwrap())
        .unwrap();
    assert!(
        root_value
            .admitted_for(&owner, owner.children().nth(2).unwrap())
            .is_none()
    );
    let parent = owner.children().nth(3).unwrap().into_element().unwrap();
    let nested = owner
        .observe_text_value(parent.children().next().unwrap())
        .unwrap();
    assert!(
        nested
            .admitted_for(&owner, parent.children().nth(2).unwrap())
            .is_none()
    );
    assert!(
        root_value
            .admitted_for(&owner, parent.children().next().unwrap())
            .is_none()
    );
    assert!(
        nested
            .admitted_for(&owner, owner.children().next().unwrap())
            .is_none()
    );
    let sibling_parent = owner.children().nth(4).unwrap().into_element().unwrap();
    assert!(
        nested
            .admitted_for(&owner, sibling_parent.children().next().unwrap())
            .is_none()
    );
    let copied = vize_l0::String::from(source);
    for original in [source, copied.as_str()] {
        let foreign = selected(&arena, original);
        let foreign_child = foreign.children().next().unwrap();
        assert!(
            root_value
                .admitted_for(&foreign, foreign_child.reborrow())
                .is_none()
        );
        let failure = owner.observe_text_value(foreign_child).err().unwrap();
        assert_eq!(failure.kind(), NativeTextValueError::ForeignComponent);
        assert!(core::ptr::eq(failure.block().root_source(), original));
        assert_eq!(failure.span().unwrap().slice(original), "same");
        assert_eq!(failure.token().unwrap().text, "same");
        assert_eq!(failure.ordinal(), 0);
        assert!(failure.parent_tag().is_none());
        assert!(
            foreign
                .observe_text_value(foreign.children().next().unwrap())
                .is_ok()
        );
    }
}

#[test]
fn original_js_ts_selection_and_neutral_source_transfer_do_not_authorize_a_new_owner() {
    let arena = Allocator::default();
    for (source, grammar) in [
        (
            "<template>&lt; not { javascript</template>",
            NativeTemplateGrammar::JavaScriptModule,
        ),
        (
            "<template>&lt; not { javascript</template><script setup lang=ts>const n=1</script>",
            NativeTemplateGrammar::TypeScriptModule,
        ),
    ] {
        let owner = selected(&arena, source);
        let child = owner.children().next().unwrap();
        let value = owner.observe_text_value(child.reborrow()).unwrap();
        let view = value.admitted_for(&owner, child).unwrap();
        assert_eq!(view.selected().grammar(), grammar);
        assert_eq!(view.observation().source().text(), "< not { javascript");
        let prepared = value.into_source();
        assert!(core::ptr::eq(prepared.authored_root(), source));
        assert!(prepared.decode_map().is_some());
        let foreign = selected(&arena, source);
        let actual = foreign
            .observe_text_value(foreign.children().next().unwrap())
            .unwrap();
        assert_eq!(actual.source().text(), prepared.text());
        assert!(!core::ptr::eq(actual.source().text(), prepared.text()));
        crate::check_fidelity(&owner.component().carrier().tree).unwrap();
    }
}

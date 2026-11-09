use super::{
    Allocator, Lang, NativeAttributeForHead, NativeAttributeOperandError, NativeTemplateGrammar,
    first_attribute, selected,
};

#[test]
fn same_actual_event_keeps_full_unicode_source_and_both_original_stock_roots() {
    let arena = Allocator::default();
    let source = "<!--前--><template><p v-for='(項目, 位置) of 項目群'>kept</p></template>";
    let owner = selected(&arena, source).unwrap();
    let attribute = first_attribute(&owner);
    let original_attribute = attribute.surface();
    let operand = owner
        .observe_attribute_for_head(attribute.reborrow())
        .unwrap();
    let alias = operand.syntax().aliases().unwrap().unwrap();
    let collection = operand.syntax().collection().unwrap().unwrap();
    let formals = alias.parameters().unwrap().as_ptr();
    let root = collection.expression().unwrap();
    assert_eq!(operand.raw_value(), "(項目, 位置) of 項目群");
    assert_eq!(operand.value_span().slice(source), operand.raw_value());
    assert_eq!(operand.name_span().slice(source), "v-for");
    let projected = operand.admitted_for(&owner, attribute).unwrap();
    assert!(core::ptr::eq(projected.selected(), &owner));
    assert!(core::ptr::eq(
        projected.attribute().surface(),
        original_attribute
    ));
    let head = projected.for_head().unwrap();
    assert!(core::ptr::eq(head.source().authored_root(), source));
    assert!(core::ptr::eq(head.authored_value(), operand.raw_value()));
    assert_eq!(head.authored_value_span(), operand.value_span());
    assert_eq!(head.source_block(), owner.component().block());
    assert_eq!(head.aliases().parameters().items.as_ptr(), formals);
    assert!(core::ptr::eq(head.collection().expression(), root));
    assert_eq!(head.alias_source().text(), "項目, 位置");
    assert_eq!(head.collection_source().text(), "項目群");
    assert_eq!(operand.syntax().diagnostics().count(), 0);
}

#[test]
fn nested_original_parent_and_header_ordinal_survive_owner_moves_and_parking_growth() {
    let arena = Allocator::default();
    let source = "<template><div><p class='x' v-for='item in items'/></div></template>";
    let owner = selected(&arena, source).unwrap();
    let (operand, original_element) = {
        let parent = owner.children().next().unwrap().into_element().unwrap();
        let child = parent.children().next().unwrap().into_element().unwrap();
        assert!(core::ptr::eq(
            child.parent_element().unwrap(),
            parent.surface()
        ));
        let attribute = child.attributes().nth(1).unwrap();
        let original_element = core::ptr::from_ref(attribute.element());
        let operand = owner
            .observe_attribute_for_head(attribute.reborrow())
            .unwrap();
        assert!(operand.admitted_for(&owner, attribute).is_some());
        (operand, original_element)
    };
    let original_collection = operand
        .syntax()
        .collection()
        .unwrap()
        .unwrap()
        .expression()
        .unwrap();
    let mut parked = alloc::vec::Vec::new();
    parked.push(operand);
    parked.reserve(32);
    let moved = core::hint::black_box(owner);
    let parent = moved.children().next().unwrap().into_element().unwrap();
    let child = parent.children().next().unwrap().into_element().unwrap();
    {
        let projected = parked
            .first()
            .unwrap()
            .admitted_for(&moved, child.attributes().nth(1).unwrap())
            .unwrap();
        assert_eq!(projected.attribute().ordinal(), 1);
        assert_eq!(
            core::ptr::from_ref(projected.attribute().element()),
            original_element
        );
        assert!(core::ptr::eq(
            projected.for_head().unwrap().collection().expression(),
            original_collection
        ));
    }
    let moved_again = core::hint::black_box(moved);
    assert_eq!(moved_again.children().len(), 1);
    assert!(parked.pop().unwrap().syntax().admitted_dense().is_some());
}

#[test]
fn sibling_headers_elements_and_equal_input_parses_cannot_replace_the_original_event() {
    let arena = Allocator::default();
    let source = "<template><p v-for='item in items' v-for='item in items'/><p v-for='item in items'/></template>";
    let owner = selected(&arena, source).unwrap();
    let first = owner.children().next().unwrap().into_element().unwrap();
    let operand = owner
        .observe_attribute_for_head(first.attributes().next().unwrap())
        .unwrap();
    assert!(
        operand
            .admitted_for(&owner, first.attributes().nth(1).unwrap())
            .is_none()
    );
    let sibling = owner.children().nth(1).unwrap().into_element().unwrap();
    assert!(
        operand
            .admitted_for(&owner, sibling.attributes().next().unwrap())
            .is_none()
    );
    let equal = selected(&arena, source).unwrap();
    assert!(
        operand
            .admitted_for(&equal, first_attribute(&equal))
            .is_none()
    );
    let failure = owner
        .observe_attribute_for_head(first_attribute(&equal))
        .err()
        .unwrap();
    assert_eq!(
        failure.kind(),
        NativeAttributeOperandError::ForeignComponent
    );
    assert!(failure.input().is_none());
    let copied = vize_l0::String::from(source);
    let foreign = selected(&arena, copied.as_str()).unwrap();
    assert!(
        operand
            .admitted_for(&foreign, first_attribute(&foreign))
            .is_none()
    );
}

#[test]
fn javascript_and_typescript_profiles_are_derived_from_selection() {
    let arena = Allocator::default();
    for (source, lang, grammar) in [
        (
            "<template><p v-for='item in items'/></template>",
            Lang::Js,
            NativeTemplateGrammar::JavaScriptModule,
        ),
        (
            "<template><p v-for='item in items'/></template><script lang=ts>const items=[]</script>",
            Lang::Ts,
            NativeTemplateGrammar::TypeScriptModule,
        ),
    ] {
        let owner = selected(&arena, source).unwrap();
        let attribute = first_attribute(&owner);
        let operand = owner
            .observe_attribute_for_head(attribute.reborrow())
            .unwrap();
        assert_eq!(owner.grammar(), grammar);
        assert_eq!(operand.syntax().grammar().lang, lang);
        let projected = operand.admitted_for(&owner, attribute).unwrap();
        let head = projected.for_head().unwrap();
        for profile in [
            head.aliases().source_type(),
            head.collection().source_type(),
        ] {
            assert!(profile.is_module());
            assert_eq!(profile.is_typescript(), lang == Lang::Ts);
            assert!(!profile.is_jsx());
        }
    }
}

#[test]
fn raw_whole_head_transfer_keeps_stock_owners_after_selected_authority_is_dropped() {
    let arena = Allocator::default();
    let source = "<template><p v-for='item in items'/></template>";
    let (head, original) = {
        let owner = selected(&arena, source).unwrap();
        let operand: NativeAttributeForHead<'_> = owner
            .observe_attribute_for_head(first_attribute(&owner))
            .unwrap();
        let original = operand
            .syntax()
            .collection()
            .unwrap()
            .unwrap()
            .expression()
            .unwrap();
        (operand.into_syntax(), original)
    };
    assert!(core::ptr::eq(
        head.collection().unwrap().unwrap().expression().unwrap(),
        original
    ));
    assert!(core::ptr::eq(head.source().authored_root(), source));
    assert_eq!(head.source().text(), "item in items");
    assert_eq!(head.diagnostics().count(), 0);
    assert!(head.admitted_dense().is_some());
}

#[test]
fn same_header_event_parks_once_observation_then_joins_without_reenumeration() {
    let arena = Allocator::default();
    let source = "<template><p v-for='item in items'/></template>";
    let owner = selected(&arena, source).unwrap();
    let element = owner.children().next().unwrap().into_element().unwrap();
    let mut parked = alloc::vec::Vec::new();
    let mut visits = 0;
    for attribute in element.attributes() {
        visits += 1;
        let original = core::ptr::from_ref(attribute.surface());
        let observed = owner
            .observe_attribute_for_head(attribute.reborrow())
            .unwrap();
        let original_collection = core::ptr::from_ref(
            observed
                .syntax()
                .collection()
                .unwrap()
                .unwrap()
                .expression()
                .unwrap(),
        );
        parked.push(observed);
        parked.reserve(32);
        let projected = parked
            .last()
            .unwrap()
            .admitted_for(&owner, attribute)
            .unwrap();
        assert_eq!(
            core::ptr::from_ref(projected.attribute().surface()),
            original
        );
        assert_eq!(
            core::ptr::from_ref(projected.for_head().unwrap().collection().expression()),
            original_collection
        );
    }
    assert_eq!(visits, 1);
    assert_eq!(parked.len(), 1);
}

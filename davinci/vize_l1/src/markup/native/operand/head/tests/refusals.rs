use super::selected;
use crate::markup::{NativeAttributeOperandError, NativeConditionKind};
use vize_l0::Allocator;

#[test]
fn original_header_recovery_and_verbatim_precede_head_and_expression_selection() {
    for (source, ordinal, expected) in [
        (
            "<template><p v-if='ready'></template>",
            0,
            NativeAttributeOperandError::RecoveredComponent,
        ),
        (
            "<template><p v-pre v-if.foo='ready'/></template>",
            1,
            NativeAttributeOperandError::Verbatim,
        ),
        (
            "<template><p v-pre v-if/></template>",
            1,
            NativeAttributeOperandError::Verbatim,
        ),
        (
            "<template><p v-pre v-if='ready'></template>",
            1,
            NativeAttributeOperandError::RecoveredComponent,
        ),
    ] {
        let arena = Allocator::default();
        let selected = selected(&arena, source);
        let element = selected.children().next().unwrap().into_element().unwrap();
        let attribute = element.attributes().nth(ordinal).unwrap();
        assert_eq!(
            selected.observe_attribute_head(attribute.reborrow()).err(),
            Some(expected)
        );
        let old = selected
            .observe_attribute_expression(attribute)
            .err()
            .unwrap();
        assert_eq!(old.kind(), expected);
        assert!(old.syntax().is_none());
        assert_eq!(selected.component().block().root_source(), source);
    }
}

#[test]
fn unsupported_parts_remain_metadata_and_cannot_prepare_a_conditional_value() {
    for (name, source) in [
        ("id", "<template><p id='a'/></template>"),
        ("v-if.foo", "<template><p v-if.foo='a'/></template>"),
        ("v-if:arg", "<template><p v-if:arg='a'/></template>"),
        ("v-if:", "<template><p v-if:='a'/></template>"),
        ("v-if:[key]", "<template><p v-if:[key]='a'/></template>"),
        ("v-else", "<template><p v-else='a'/></template>"),
        ("v-", "<template><p v-='a'/></template>"),
    ] {
        let arena = Allocator::default();
        let selected = selected(&arena, source);
        let element = selected.children().next().unwrap().into_element().unwrap();
        let attribute = element.attributes().next().unwrap();
        let before = arena.allocated_bytes();
        let head = selected
            .observe_attribute_head(attribute.reborrow())
            .unwrap();
        assert_eq!(head.name_block().source(), name);
        assert_eq!(head.condition_kind(), None);
        let failure = head.observe_expression().err().unwrap();
        assert_eq!(
            failure.kind(),
            NativeAttributeOperandError::UnsupportedDirective
        );
        assert!(failure.syntax().is_none());
        assert_eq!(arena.allocated_bytes(), before);
        let old = selected
            .observe_attribute_expression(attribute)
            .err()
            .unwrap();
        assert_eq!(old.kind(), failure.kind());
        assert!(old.syntax().is_none());
        assert_eq!(selected.component().block().root_source(), source);
    }
}

#[test]
fn head_can_exist_without_value_but_original_complete_value_refusal_stays_first() {
    let arena = Allocator::default();
    let source = "<template><p v-if/></template>";
    let selected = selected(&arena, source);
    let element = selected.children().next().unwrap().into_element().unwrap();
    let attribute = element.attributes().next().unwrap();
    let before = arena.allocated_bytes();
    let head = selected
        .observe_attribute_head(attribute.reborrow())
        .unwrap();
    assert_eq!(head.condition_kind(), Some(NativeConditionKind::If));
    assert_eq!(head.name_block().source(), "v-if");
    let failure = head.observe_expression().err().unwrap();
    assert_eq!(failure.kind(), NativeAttributeOperandError::IncompleteValue);
    assert!(failure.syntax().is_none());
    assert_eq!(arena.allocated_bytes(), before);
    let old = selected
        .observe_attribute_expression(attribute)
        .err()
        .unwrap();
    assert_eq!(old.kind(), failure.kind());
    assert!(old.syntax().is_none());
    assert_eq!(selected.component().block().root_source(), source);
}

#[test]
fn original_directive_capacity_fact_stays_precise_under_earliest_recovery_refusal() {
    let arena = Allocator::default();
    // The initial dynamic argument owns one run; 32 alternating pairs need
    // another 64. Identical delimiters would coalesce and are not this fault.
    let name = alloc::format!("v-if:[{}key{}]", "([".repeat(32), "])".repeat(32));
    let source = alloc::format!("<template><p {name}='ready'/></template>");
    let selected = selected(&arena, &source);
    let element = selected.children().next().unwrap().into_element().unwrap();
    let attribute = element.attributes().next().unwrap();
    assert_eq!(attribute.surface().name.text, name.as_str());
    let unsupported = &selected.component().carrier().unsupported;
    assert_eq!(unsupported.len(), 1);
    assert_eq!(
        unsupported[0].error,
        crate::markup::DirectiveNameError::NestingLimit
    );
    assert_eq!(
        unsupported[0].span,
        vize_l0::Span::new(3, 3 + name.len() as u32)
    );
    assert_eq!(
        unsupported[0]
            .span
            .slice(selected.component().block().source()),
        name.as_str()
    );
    let before = arena.allocated_bytes();
    assert_eq!(
        selected.observe_attribute_head(attribute.reborrow()).err(),
        Some(NativeAttributeOperandError::RecoveredComponent)
    );
    let old = selected
        .observe_attribute_expression(attribute)
        .err()
        .unwrap();
    assert_eq!(old.kind(), NativeAttributeOperandError::RecoveredComponent);
    assert!(old.syntax().is_none());
    assert_eq!(arena.allocated_bytes(), before);
    assert_eq!(selected.component().block().root_source(), source.as_str());
    crate::check_fidelity(&selected.component().carrier().tree).unwrap();
}

#[test]
fn foreign_membership_precedes_the_selected_owners_own_recovery() {
    let arena = Allocator::default();
    let recovered_source = "<template><p v-if='ready'></template>";
    let recovered = selected(&arena, recovered_source);
    let foreign_source = "<template><p v-if='ready'/></template>";
    let foreign = selected(&arena, foreign_source);
    let element = foreign.children().next().unwrap().into_element().unwrap();
    let attribute = element.attributes().next().unwrap();
    assert_eq!(
        recovered.observe_attribute_head(attribute.reborrow()).err(),
        Some(NativeAttributeOperandError::ForeignComponent)
    );
    let old = recovered
        .observe_attribute_expression(attribute)
        .err()
        .unwrap();
    assert_eq!(old.kind(), NativeAttributeOperandError::ForeignComponent);
    assert!(old.syntax().is_none());
    assert_eq!(
        recovered.component().block().root_source(),
        recovered_source
    );
    assert_eq!(foreign.component().block().root_source(), foreign_source);
}

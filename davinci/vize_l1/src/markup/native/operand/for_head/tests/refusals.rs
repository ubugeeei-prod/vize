use super::{
    Allocator, EmbedHole, ForHeadHole, Lang, NativeAttributeOperandError, NativeForInput,
    NativeForRefusal, first_attribute, selected,
};

#[test]
fn complete_attribute_extent_cannot_be_replaced_by_a_locally_admitted_prefix() {
    let arena = Allocator::default();
    let source = "<template><p v-for='item in items trailing'/></template>";
    let owner = selected(&arena, source).unwrap();
    let attribute = first_attribute(&owner);
    let operand = owner
        .observe_attribute_for_head(attribute.reborrow())
        .unwrap();
    assert_eq!(operand.raw_value(), "item in items trailing");
    assert_eq!(operand.value_span().slice(source), operand.raw_value());
    assert!(core::ptr::eq(
        operand.syntax().source().text(),
        operand.raw_value()
    ));
    assert!(operand.syntax().diagnostics().count() > 0);
    assert!(operand.admitted_for(&owner, attribute).is_none());
    // The public local provider can prepare a valid physical prefix; only this
    // selected producer can attach the complete immutable Attribute event.
    let prefix = operand.raw_value().get(..13).unwrap();
    let local = NativeForInput::attribute_in(&arena, owner.component().block(), prefix, Lang::Js)
        .unwrap()
        .observe();
    assert!(local.admitted_dense().is_some());
    assert_ne!(
        local.admitted_dense().unwrap().authored_value_span(),
        operand.value_span()
    );
}

#[test]
fn syntax_and_native_family_refusals_keep_whole_sources_and_original_observations() {
    let arena = Allocator::default();
    for (source, refusal, comments, diagnostics, decoded) in [
        (
            "<template><p v-for='item /*kept*/ in items'/></template>",
            NativeForRefusal::Comment,
            1,
            false,
            false,
        ),
        (
            "<template><p v-for='item in &#105;tems'/></template>",
            NativeForRefusal::EntityOutput,
            0,
            false,
            true,
        ),
        (
            "<template><p v-for='item in values + /*kept*/'/></template>",
            NativeForRefusal::Head(ForHeadHole::CollectionUnavailable(EmbedHole::Syntax)),
            1,
            true,
            false,
        ),
        (
            "<template><p v-for='arguments in items'/></template>",
            NativeForRefusal::Head(ForHeadHole::AliasesUnavailable(
                EmbedHole::InvalidParameterContext,
            )),
            0,
            false,
            false,
        ),
    ] {
        let owner = selected(&arena, source).unwrap();
        let attribute = first_attribute(&owner);
        let operand = owner
            .observe_attribute_for_head(attribute.reborrow())
            .unwrap();
        assert_eq!(operand.syntax().native_refusal(), Some(refusal));
        assert_eq!(operand.syntax().comments().count(), comments);
        assert_eq!(operand.syntax().diagnostics().count() > 0, diagnostics);
        assert_eq!(operand.syntax().source().decode_map().is_some(), decoded);
        assert_eq!(operand.value_span().slice(source), operand.raw_value());
        assert!(operand.syntax().aliases().is_some());
        assert!(operand.syntax().collection().is_some());
        assert!(operand.admitted_for(&owner, attribute).is_none());
    }
}

#[test]
fn empty_and_sparse_values_retain_the_whole_owner_without_dense_admission() {
    let arena = Allocator::default();
    for source in [
        "<template><p v-for=''/></template>",
        "<template><p v-for='(item,) in items'/></template>",
    ] {
        let owner = selected(&arena, source).unwrap();
        let attribute = first_attribute(&owner);
        let operand = owner
            .observe_attribute_for_head(attribute.reborrow())
            .unwrap();
        assert!(matches!(
            operand.syntax().native_refusal(),
            Some(NativeForRefusal::Head(_))
        ));
        assert_eq!(operand.value_span().slice(source), operand.raw_value());
        assert!(operand.admitted_for(&owner, attribute).is_none());
    }
}

#[test]
fn verbatim_unsupported_heads_and_missing_values_are_preparation_refusals() {
    let arena = Allocator::default();
    for (source, ordinal, expected) in [
        (
            "<template><p v-pre v-for='item in items'/></template>",
            1,
            NativeAttributeOperandError::Verbatim,
        ),
        (
            "<template><p v-for.foo='item in items'/></template>",
            0,
            NativeAttributeOperandError::UnsupportedDirective,
        ),
        (
            "<template><p v-for:arg='item in items'/></template>",
            0,
            NativeAttributeOperandError::UnsupportedDirective,
        ),
        (
            "<template><p v-for:='item in items'/></template>",
            0,
            NativeAttributeOperandError::UnsupportedDirective,
        ),
        (
            "<template><p v-if='ready'/></template>",
            0,
            NativeAttributeOperandError::UnsupportedDirective,
        ),
        (
            "<template><p class='item in items'/></template>",
            0,
            NativeAttributeOperandError::UnsupportedDirective,
        ),
        (
            "<template><p v-for/></template>",
            0,
            NativeAttributeOperandError::IncompleteValue,
        ),
    ] {
        let owner = selected(&arena, source).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let failure = owner
            .observe_attribute_for_head(element.attributes().nth(ordinal).unwrap())
            .err()
            .unwrap();
        assert_eq!(failure.kind(), expected, "{source}");
        assert!(failure.input().is_none());
        assert_eq!(owner.component().block().root_source(), source);
    }
}

#[test]
fn real_missing_close_and_recovery_precedence_refuse_before_dialect_selection() {
    let arena = Allocator::default();
    for source in [
        "<template><p v-for='item in items'></template>",
        "<template><p v-pre class='item in items'></template>",
    ] {
        let owner = selected(&arena, source).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        assert!(owner.component().carrier().errors.is_empty());
        assert!(matches!(
            element.surface().close,
            crate::ElementClose::Missing
        ));
        let failure = owner
            .observe_attribute_for_head(element.attributes().last().unwrap())
            .err()
            .unwrap();
        assert_eq!(
            failure.kind(),
            NativeAttributeOperandError::RecoveredComponent
        );
        assert!(failure.input().is_none());
    }
}

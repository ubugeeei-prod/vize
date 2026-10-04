use super::{Allocator, NativeAttributeOperandError, selected};

#[test]
fn sibling_element_ordinal_equal_source_and_foreign_selection_cannot_join() {
    let arena = Allocator::default();
    let source =
        "<template><div title='same&amp;' id='same&amp;'><i title='same&amp;'/></div></template>";
    let selected = selected(&arena, source);
    let element = selected.children().next().unwrap().into_element().unwrap();
    let original = selected
        .observe_attribute_value(element.attributes().next().unwrap())
        .unwrap();
    assert!(
        original
            .admitted_for(&selected, element.attributes().nth(1).unwrap())
            .is_none()
    );
    let nested = element.children().next().unwrap().into_element().unwrap();
    assert!(
        original
            .admitted_for(&selected, nested.attributes().next().unwrap())
            .is_none()
    );
    let foreign = super::selected(&arena, source);
    let other_element = foreign.children().next().unwrap().into_element().unwrap();
    let other_attribute = other_element.attributes().next().unwrap();
    assert!(
        original
            .admitted_for(&foreign, other_attribute.reborrow())
            .is_none()
    );
    let before = arena.allocated_bytes();
    let failure = selected
        .observe_attribute_value(other_attribute)
        .err()
        .unwrap();
    assert_eq!(
        failure.kind(),
        NativeAttributeOperandError::ForeignComponent
    );
    assert_eq!(failure.raw_name(), "title");
    assert_eq!(failure.raw_value(), Some("same&amp;"));
    assert_eq!(failure.value_span().unwrap().slice(source), "same&amp;");
    assert_eq!(arena.allocated_bytes(), before);
}

#[test]
fn recovered_verbatim_boolean_and_missing_values_keep_exact_origin_failure() {
    let arena = Allocator::default();
    for (source, expected) in [
        (
            "<template><div v-pre title='a&amp;'/></template>",
            NativeAttributeOperandError::Verbatim,
        ),
        (
            "<template><div title='a&amp;'></template>",
            NativeAttributeOperandError::RecoveredComponent,
        ),
        (
            "<template><div title/></template>",
            NativeAttributeOperandError::IncompleteValue,
        ),
        (
            "<template><div title= /></template>",
            NativeAttributeOperandError::RecoveredComponent,
        ),
        (
            "<template><div title= ></div></template>",
            NativeAttributeOperandError::RecoveredComponent,
        ),
        (
            "<template><div title='a&amp;</template>",
            NativeAttributeOperandError::RecoveredComponent,
        ),
    ] {
        let selected = selected(&arena, source);
        let element = selected.children().next().unwrap().into_element().unwrap();
        let attribute = element
            .attributes()
            .find(|a| a.surface().name.text == "title")
            .unwrap();
        match source {
            "<template><div title= /></template>" => {
                // Slash is real unquoted data; the original Element is unclosed.
                assert_eq!(
                    attribute.surface().value.as_ref().unwrap().content.text,
                    "/"
                );
                assert!(matches!(
                    element.surface().close,
                    crate::ElementClose::Missing
                ));
            }
            "<template><div title= ></div></template>" => {
                assert!(
                    selected
                        .component()
                        .carrier()
                        .errors
                        .iter()
                        .any(|error| { error.code == vize_l0::ErrorCode::MissingAttributeValue })
                );
                assert!(
                    attribute
                        .surface()
                        .value
                        .as_ref()
                        .unwrap()
                        .content
                        .is_missing()
                );
                assert!(matches!(
                    element.surface().close,
                    crate::ElementClose::Present(_)
                ));
            }
            "<template><div title='a&amp;</template>" => {
                // The actual Descriptor retains the template; the Component
                // reaches EOF inside the quote and keeps its original holes.
                assert_eq!(selected.component().block().source(), "<div title='a&amp;");
                assert!(element.surface().open.gt.is_missing());
                assert!(
                    attribute
                        .surface()
                        .value
                        .as_ref()
                        .unwrap()
                        .close_quote
                        .as_ref()
                        .unwrap()
                        .is_missing()
                );
            }
            _ => {}
        }
        let raw_value = attribute.surface().value.as_ref().map(|v| v.content.text);
        let ordinal = attribute.ordinal();
        let before = arena.allocated_bytes();
        let failure = selected.observe_attribute_value(attribute).err().unwrap();
        assert_eq!(failure.kind(), expected, "{source}");
        assert_eq!(failure.raw_name(), "title");
        assert_eq!(failure.raw_value(), raw_value);
        assert_eq!(failure.ordinal(), ordinal);
        assert!(core::ptr::eq(failure.block().root_source(), source));
        assert_eq!(failure.name_span().unwrap().slice(source), "title");
        assert_eq!(arena.allocated_bytes(), before);
        let mut parked = alloc::vec::Vec::new();
        parked.push(failure);
        parked.reserve(32);
        assert_eq!(parked[0].kind(), expected);
        assert_eq!(parked[0].raw_value(), raw_value);
    }
}

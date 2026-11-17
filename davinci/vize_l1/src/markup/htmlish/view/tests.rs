use super::NativeComponent;
use alloc::vec::Vec;
use vize_l0::{Allocator, SourceRoot};

#[test]
fn projections_retain_original_parent_order_and_complete_values() {
    let arena = Allocator::default();
    let source = "<div a=\"one\" :b=\"two\"><p c='three'>text</p><!--kept--></div><span/>";
    let component =
        NativeComponent::parse_in(&arena, SourceRoot::new(source).unwrap().whole_block()).unwrap();
    let mut roots = component.children();
    assert_eq!(roots.len(), 2);
    let div = roots.next().unwrap().into_element().unwrap();
    assert_eq!(div.ordinal(), 0);
    assert!(div.parent_element().is_none());
    let attributes: Vec<_> = div.attributes().collect();
    assert_eq!(
        attributes
            .iter()
            .map(|a| (a.ordinal(), a.surface().name.text))
            .collect::<Vec<_>>(),
        [(0, "a"), (1, ":b")]
    );
    for attribute in &attributes {
        assert!(core::ptr::eq(attribute.component(), &component));
        assert!(core::ptr::eq(attribute.element(), div.surface()));
        let original = div.surface().open.attrs.get(attribute.ordinal()).unwrap();
        assert!(core::ptr::eq(attribute.surface(), original));
    }
    assert_eq!(
        attributes[1].surface().value.as_ref().unwrap().content.text,
        "two"
    );
    let mut children = div.children();
    let p = children.next().unwrap().into_element().unwrap();
    assert!(core::ptr::eq(p.parent_element().unwrap(), div.surface()));
    assert_eq!(
        p.attributes()
            .next()
            .unwrap()
            .surface()
            .value
            .as_ref()
            .unwrap()
            .content
            .text,
        "three"
    );
    assert_eq!(children.next().unwrap().ordinal(), 1);
    assert_eq!(roots.next().unwrap().into_element().unwrap().ordinal(), 1);
}

#[test]
fn empty_original_owner_moves_without_address_or_empty_vec_identity() {
    let arena = Allocator::default();
    let block = SourceRoot::new("").unwrap().whole_block();
    let component = NativeComponent::parse_in(&arena, block).unwrap();
    assert_eq!(component.children().len(), 0);
    let moved = core::hint::black_box(component);
    assert_eq!(moved.children().len(), 0);
    assert_eq!(moved.into_carrier().tree.children.len(), 0);
}

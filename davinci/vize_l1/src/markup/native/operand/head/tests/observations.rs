use super::selected;
use crate::markup::{NativeAttributeExpression, NativeTemplateComponent};

// Every operand below comes from a Descriptor-selected original Attribute.
// The two public routes parse independently; neither is an AST oracle for the
// other. Literal coordinates and complete streams are authored expectations.
fn assert_own_original_root<'a>(
    operand: &NativeAttributeExpression<'a>,
    owner: &NativeTemplateComponent<'a>,
    ordinal: usize,
) {
    let element = owner.children().next().unwrap().into_element().unwrap();
    let attribute = element.attributes().nth(ordinal).unwrap();
    let original = operand.syntax().expression().unwrap();
    let admitted = operand.admitted_for(owner, attribute.reborrow()).unwrap();
    assert!(core::ptr::eq(admitted.selected(), owner));
    assert!(core::ptr::eq(admitted.operand(), operand));
    assert_eq!(admitted.attribute().ordinal(), ordinal);
    assert!(core::ptr::eq(
        admitted.attribute().element(),
        element.surface()
    ));
    assert!(core::ptr::eq(
        admitted.attribute().surface(),
        attribute.surface()
    ));
    assert!(core::ptr::eq(
        admitted.expression().unwrap().expression(),
        original
    ));
    assert!(core::ptr::eq(
        operand.syntax().source().authored_root(),
        owner.component().block().root_source()
    ));
}

mod heads;
mod holes;
mod profiles;
mod source_geometry;

use super::{NativeComponent, NativeTemplateComponent};
use crate::container::Vue;
use crate::container::vue::DescriptorOptions;
use vize_l0::{
    Allocator, SourceRoot,
    config::{VueDialect, VueVersion},
};

fn selected<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateComponent<'a>, &'static str> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: crate::SurfaceParseOptions::default(),
        },
    );
    let admitted = descriptor
        .admitted()
        .map_err(|_| "fixture descriptor refused")?;
    NativeTemplateComponent::parse_in(arena, admitted)
        .map_err(|_| "fixture component parse failed")?
        .ok_or("fixture has no template")
}

#[test]
fn same_header_token_observes_then_joins_after_pending_growth() {
    let arena = Allocator::default();
    let source = "<template><p v-if='msg &amp;&amp; ok' title=kept /></template>";
    let owner = selected(&arena, source).unwrap();
    let element = owner.children().next().unwrap().into_element().unwrap();
    let mut header = element.attributes();
    let attribute = header.next().unwrap();
    let original = attribute.surface();
    let operand = owner
        .observe_attribute_expression(attribute.reborrow())
        .unwrap();
    let root = operand.syntax().expression().unwrap();
    let mut pending = alloc::vec::Vec::new();
    pending.push(operand);
    pending.reserve(32);
    let view = pending
        .first()
        .unwrap()
        .admitted_for(&owner, attribute)
        .unwrap();
    assert!(core::ptr::eq(view.attribute().surface(), original));
    assert!(core::ptr::eq(view.attribute().element(), element.surface()));
    assert!(core::ptr::eq(view.expression().unwrap().expression(), root));
    assert_eq!(view.attribute().ordinal(), 0);
    assert_eq!(view.operand().syntax().source().text(), "msg && ok");
    assert_eq!(header.len(), 1);
    assert_eq!(header.next().unwrap().surface().name.text, "title");
    assert_eq!(header.len(), 0);
}

#[test]
fn reborrow_preserves_foreign_component_refusal_and_original_event() {
    let arena = Allocator::default();
    let source = "<template><p v-if='ready'/></template>";
    let owner = selected(&arena, source).unwrap();
    let foreign = selected(&arena, source).unwrap();
    let element = foreign.children().next().unwrap().into_element().unwrap();
    let attribute = element.attributes().next().unwrap();
    let failure = match owner.observe_attribute_expression(attribute.reborrow()) {
        Ok(_) => panic!("foreign original Attribute was admitted"),
        Err(failure) => failure,
    };
    assert_eq!(
        failure.kind(),
        super::NativeAttributeOperandError::ForeignComponent
    );
    assert!(core::ptr::eq(attribute.component(), foreign.component()));
    let operand = foreign
        .observe_attribute_expression(attribute.reborrow())
        .unwrap();
    assert!(operand.admitted_for(&foreign, attribute).is_some());
}

#[test]
fn same_child_token_keeps_root_and_nested_parent_before_consumption() {
    let arena = Allocator::default();
    let source = "<p>prefix<span/></p><!--kept-->";
    let component =
        NativeComponent::parse_in(&arena, SourceRoot::new(source).unwrap().whole_block()).unwrap();
    let mut roots = component.children();
    let child = roots.next().unwrap();
    {
        let borrowed = child.reborrow();
        assert!(core::ptr::eq(borrowed.component(), &component));
        assert!(core::ptr::eq(borrowed.surface(), child.surface()));
        assert!(borrowed.parent_element().is_none());
        assert_eq!(borrowed.ordinal(), 0);
    }
    let element = child.into_element().unwrap();
    let mut body = element.children();
    let text = body.next().unwrap();
    assert_eq!(text.reborrow().ordinal(), 0);
    assert!(core::ptr::eq(
        text.reborrow().parent_element().unwrap(),
        element.surface()
    ));
    let nested = body.next().unwrap();
    {
        let borrowed = nested.reborrow();
        assert!(core::ptr::eq(borrowed.surface(), nested.surface()));
        assert_eq!(borrowed.ordinal(), 1);
        assert!(core::ptr::eq(
            borrowed.parent_element().unwrap(),
            element.surface()
        ));
    }
    assert_eq!(
        nested.into_element().unwrap().surface().open.lt_name.text,
        "<span"
    );
    assert_eq!(body.len(), 0);
    assert_eq!(roots.next().unwrap().ordinal(), 1);
    assert_eq!(roots.len(), 0);
}

#[test]
fn reborrow_never_promotes_unquoted_slash_recovery_to_header_admission() {
    let arena = Allocator::default();
    let source = "<template><p v-if='msg &amp;&amp; ok' title=kept/></template>";
    let owner = selected(&arena, source).unwrap();
    let element = owner.children().next().unwrap().into_element().unwrap();
    let attribute = element.attributes().next().unwrap();
    let failure = match owner.observe_attribute_expression(attribute.reborrow()) {
        Ok(_) => panic!("missing original Element close was admitted"),
        Err(failure) => failure,
    };
    assert_eq!(
        failure.kind(),
        super::NativeAttributeOperandError::RecoveredComponent
    );
    assert!(failure.syntax().is_none());
    assert!(matches!(
        element.surface().close,
        crate::ElementClose::Missing
    ));
    assert_eq!(attribute.surface().name.text, "v-if");
    assert!(core::ptr::eq(attribute.component(), owner.component()));
    assert!(core::ptr::eq(
        owner.component().block().root_source(),
        source
    ));
}

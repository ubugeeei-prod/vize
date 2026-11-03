use crate::container::Vue;
use crate::container::vue::DescriptorOptions;
use crate::markup::{NativeAttributeModelExpression, NativeTemplateComponent};
use vize_l0::config::{VueDialect, VueVersion};
use vize_l0::{Allocator, Span};

pub(super) fn selected<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: crate::markup::SurfaceParseOptions::default(),
        },
    );
    let selected = NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap();
    assert!(core::ptr::eq(
        selected.component().block().root_source(),
        source
    ));
    crate::markup::check_fidelity(&selected.component().carrier().tree).unwrap();
    selected
}

pub(super) fn observe<'a>(
    selected: &NativeTemplateComponent<'a>,
    ordinal: usize,
) -> NativeAttributeModelExpression<'a> {
    let element = selected.children().next().unwrap().into_element().unwrap();
    let head = selected
        .observe_attribute_head(element.attributes().nth(ordinal).unwrap())
        .unwrap();
    let model = head.model().unwrap().unwrap();
    assert!(core::ptr::eq(model.head(), &head));
    model.observe_expression().unwrap()
}

pub(super) fn assert_original<'a>(
    operand: &NativeAttributeModelExpression<'a>,
    selected: &NativeTemplateComponent<'a>,
    ordinal: usize,
) {
    let element = selected.children().next().unwrap().into_element().unwrap();
    let attribute = element.attributes().nth(ordinal).unwrap();
    let before = selected.component().allocator().allocated_bytes();
    let view = operand
        .admitted_for(selected, attribute.reborrow())
        .unwrap();
    assert!(core::ptr::eq(view.selected(), selected));
    assert!(core::ptr::eq(view.operand(), operand));
    assert!(core::ptr::eq(view.attribute().element(), element.surface()));
    assert!(core::ptr::eq(
        view.attribute().surface(),
        attribute.surface()
    ));
    assert_eq!(view.attribute().ordinal(), ordinal);
    assert!(core::ptr::eq(
        view.expression().unwrap().expression(),
        operand.syntax().expression().unwrap()
    ));
    assert!(core::ptr::eq(
        operand.syntax().source().authored_root(),
        selected.component().block().root_source()
    ));
    assert_eq!(selected.component().allocator().allocated_bytes(), before);
}

pub(super) fn source_span(source: &str, token: &str) -> Span {
    let start = u32::try_from(source.find(token).unwrap()).unwrap();
    Span::new(start, start + u32::try_from(token.len()).unwrap())
}

pub(super) fn value_span(source: &str, raw: &str) -> Span {
    let attribute = source_span(source, &alloc::format!("v-model='{raw}'"));
    Span::new(attribute.start + 9, attribute.end - 1)
}

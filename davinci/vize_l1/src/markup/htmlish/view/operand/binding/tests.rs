use crate::container::Vue;
use crate::container::vue::DescriptorOptions;
use crate::markup::{AttributeBindingExpression, TemplateComponent};
use vize_l0::Allocator;
use vize_l0::config::{VueDialect, VueVersion};

fn selected<'a>(arena: &'a Allocator, source: &'a str) -> TemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: crate::markup::SurfaceParseOptions::default(),
        },
    );
    let selected = TemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap();
    assert!(core::ptr::eq(
        selected.component().block().root_source(),
        source
    ));
    crate::markup::check_fidelity(&selected.component().carrier().tree).unwrap();
    selected
}

fn observe<'a>(selected: &TemplateComponent<'a>, ordinal: usize) -> AttributeBindingExpression<'a> {
    let element = selected.children().next().unwrap().into_element().unwrap();
    let head = selected
        .observe_attribute_head(element.attributes().nth(ordinal).unwrap())
        .unwrap();
    head.static_binding()
        .unwrap()
        .unwrap()
        .observe_expression()
        .unwrap()
}

fn assert_original<'a>(
    operand: &AttributeBindingExpression<'a>,
    selected: &TemplateComponent<'a>,
    ordinal: usize,
) {
    let element = selected.children().next().unwrap().into_element().unwrap();
    let attribute = element.attributes().nth(ordinal).unwrap();
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
}

mod geometry;
mod holes;
mod observations;
mod ownership;
mod refusals;

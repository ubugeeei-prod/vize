use super::*;
use crate::native::{header, template::NativeTemplateAttributeProfile};
use vize_l0::{Allocator, SourceRoot};

#[test]
fn normally_owned_checked_attribute_reports_only_its_exact_component_and_full_range() {
    let allocator = Allocator::new();
    let source = "界\r\n<div :title.camel='opaque'/>";
    let root =
        NativeLintComponent::parse_in(&allocator, SourceRoot::new(source).unwrap().whole_block())
            .unwrap();
    let mut children = root.children();
    assert!(matches!(
        children.next().unwrap().surface(),
        vize_l1::SurfaceChild::Text(_)
    ));
    let element = children.next().unwrap().into_element().unwrap();
    assert_eq!(element.ordinal(), 1);
    let original = element.attributes().next().unwrap();
    let (checked, binding) = header::wide_binding(&element, original).unwrap();
    let proof = NativeTemplateAttribute::from_checked(
        checked,
        binding,
        NativeTemplateAttributeProfile::Bindings,
    )
    .unwrap();
    let rule = "vue/component-definition-name-casing";
    let configured =
        Linter::new().with_rule_severity_overrides(vec![(rule.into(), Severity::Error)]);
    let mut context = NativeTemplateLintContext::new(&configured, &root, "/元/exact.vue");
    context.current_rule = rule;
    context
        .warn_attribute_with_help(
            &proof,
            "vue/component-definition-name-casing.message",
            &[("name", "actual")],
            "vue/component-definition-name-casing.help",
        )
        .unwrap();
    let result = context.finish();
    assert_eq!(result.filename.as_str(), "/元/exact.vue");
    assert_eq!((result.error_count, result.warning_count), (1, 0));
    let diagnostic = &result.diagnostics[0];
    let start = source.find(":title").unwrap() as u32;
    assert_eq!(
        (diagnostic.start, diagnostic.end),
        (start, start + ":title.camel='opaque'".len() as u32)
    );
    assert_eq!(diagnostic.rule_name, rule);
    assert_eq!(diagnostic.severity, Severity::Error);
    assert!(!diagnostic.message.is_empty());
    assert!(diagnostic.help.is_some());
    assert!(diagnostic.labels.is_empty());
    assert!(diagnostic.fix.is_none());
}

#[test]
fn same_source_buffer_and_equal_span_cannot_substitute_for_the_contexts_original_component() {
    let allocator = Allocator::new();
    let source = "<div title='same'/>";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let root = NativeLintComponent::parse_in(&allocator, block).unwrap();
    let foreign = NativeLintComponent::parse_in(&allocator, block).unwrap();
    assert!(core::ptr::eq(
        root.component().block().source(),
        foreign.component().block().source()
    ));
    assert!(!core::ptr::eq(root.component(), foreign.component()));
    let element = foreign.children().next().unwrap().into_element().unwrap();
    let (checked, binding) =
        header::wide_binding(&element, element.attributes().next().unwrap()).unwrap();
    let proof = NativeTemplateAttribute::from_checked(
        checked,
        binding,
        NativeTemplateAttributeProfile::Bindings,
    )
    .unwrap();
    let configured = Linter::new();
    let mut context = NativeTemplateLintContext::new(&configured, &root, "exact.vue");
    context.current_rule = "vue/component-definition-name-casing";
    assert_eq!(
        context.warn_attribute_with_help(
            &proof,
            "vue/component-definition-name-casing.message",
            &[("name", "foreign")],
            "vue/component-definition-name-casing.help"
        ),
        Err(NativeTemplateLintRefusal::SourceMismatch)
    );
    let result = context.finish();
    assert!(result.diagnostics.is_empty());
    assert_eq!((result.error_count, result.warning_count), (0, 0));
}

#[test]
fn an_authentic_attribute_of_another_element_cannot_join_a_same_component_checked_header() {
    let allocator = Allocator::new();
    let source = "<div title='same'/><span title='same'/>";
    let root =
        NativeLintComponent::parse_in(&allocator, SourceRoot::new(source).unwrap().whole_block())
            .unwrap();
    let mut children = root.children();
    let first = children.next().unwrap().into_element().unwrap();
    let second = children.next().unwrap().into_element().unwrap();
    assert!(core::ptr::eq(first.component(), second.component()));
    let original = first.attributes().next().unwrap();
    assert_eq!(
        header::wide_binding(&second, original).err().unwrap(),
        crate::native::NativeLintRefusal::SourceMismatch
    );
}

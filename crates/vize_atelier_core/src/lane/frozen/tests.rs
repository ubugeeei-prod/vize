use super::{
    CustomElementMatcher, ErrorCode, Span, TransformContext, TransformOptions,
    transform_with_frozen_elements,
};
use crate::{ElementType, PropNode, TemplateChildNode, parser::Parser};
use vize_l0::Allocator;

#[test]
fn foreign_root_or_caller_is_fatal_before_any_root_mutation() {
    for case in 0..3 {
        let arena = Allocator::new();
        let source_a = "<div v-pre><Child is=\"vue:Child\"></Child></div>".to_owned();
        let source_b = if case == 2 {
            source_a.clone()
        } else {
            "<nav v-pre><Other is=\"vue:Other\"></Other></nav>".to_owned()
        };
        let (root_a, errors, frozen) = Parser::new(&arena, &source_a).parse_with_frozen_elements();
        assert!(errors.is_empty(), "{errors:?}");
        let (root_b, errors) = Parser::new(&arena, &source_b).parse();
        assert!(errors.is_empty(), "{errors:?}");
        let (mut root, caller) = if case == 1 {
            (root_a, source_b.as_str())
        } else {
            (root_b, source_a.as_str())
        };
        let before = vize_l0::cstr!("{root:?}");
        let errors = transform_with_frozen_elements(
            &arena,
            caller,
            &mut root,
            TransformOptions::default(),
            None,
            CustomElementMatcher::default(),
            false,
            None,
            frozen,
        );
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ErrorCode::ExtendPoint);
        assert_eq!(
            errors[0].message,
            "Frozen element provenance belongs to a different template source."
        );
        assert!(!errors[0].is_recoverable());
        assert_eq!(vize_l0::cstr!("{root:?}"), before, "foreign case {case}");
    }
}

#[test]
fn literal_own_and_inherited_tags_keep_is_while_outside_casts_still_resolve() {
    let arena = Allocator::new();
    let source = "<div v-pre><span><Child is=\"vue:Child\"></Child></span></div><Child is=\"vue:Child\"></Child><p is=\"vue:Row\"></p>";
    let (mut root, errors, frozen) = Parser::with_options(
        &arena,
        source,
        crate::options::ParserOptions {
            is_native_tag: Some(vize_l0::is_native_tag),
            ..Default::default()
        },
    )
    .parse_with_frozen_elements();
    assert!(errors.is_empty(), "{errors:?}");
    let errors = transform_with_frozen_elements(
        &arena,
        source,
        &mut root,
        TransformOptions {
            prefix_identifiers: true,
            ssr: true,
            ..Default::default()
        },
        None,
        CustomElementMatcher::default(),
        false,
        None,
        frozen,
    );
    assert!(errors.is_empty(), "{errors:?}");
    let TemplateChildNode::Element(div) = &root.children[0] else {
        panic!("literal root")
    };
    let TemplateChildNode::Element(span) = &div.children[0] else {
        panic!("literal span")
    };
    let TemplateChildNode::Element(child) = &span.children[0] else {
        panic!("literal Child")
    };
    assert_eq!(child.tag, "Child");
    assert_eq!(child.tag_type, ElementType::Element);
    assert!(child.props.iter().any(|prop| matches!(prop,
        PropNode::Attribute(attr) if attr.name == "is" && attr.value.as_ref().is_some_and(|v| v.content == "vue:Child")
    )));
    for (index, tag) in [(1, "Child"), (2, "Row")] {
        let TemplateChildNode::Element(el) = &root.children[index] else {
            panic!("outside component")
        };
        assert_eq!(el.tag, tag);
        assert_eq!(el.tag_type, ElementType::Component);
        assert!(
            !el.props
                .iter()
                .any(|prop| matches!(prop, PropNode::Attribute(attr) if attr.name == "is"))
        );
    }
}

#[test]
fn ordinary_transform_context_carries_no_frozen_ownership() {
    let arena = Allocator::new();
    let context = TransformContext::new(&arena, "<Child></Child>", TransformOptions::default());
    assert!(context.frozen_elements.is_empty());
    assert!(!context.element_is_frozen(Span::new(0, 7)));
}

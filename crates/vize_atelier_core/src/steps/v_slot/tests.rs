use super::*;
use crate::SourceLocation;
use crate::errors::{CompilerError, ErrorCode};
use crate::lane::traverse::traverse_children;
use crate::lane::{ParentNode, TransformContext};
use crate::options::TransformOptions;
use crate::parser::parse;
use vize_carton::Allocator;

fn transform_errors(source: &str) -> std::vec::Vec<CompilerError> {
    let allocator = Allocator::new();
    let (mut root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    let mut ctx = TransformContext::new(&allocator, root.source, TransformOptions::default());
    traverse_children(&mut ctx, ParentNode::Root(&mut root as *mut _));
    ctx.errors
}

#[test]
fn test_has_v_slot() {
    let allocator = Allocator::new();
    let (root, _) = parse(&allocator, r#"<template v-slot:header>content</template>"#);

    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert!(has_v_slot(el));
    }
}

#[test]
fn test_default_slot_name() {
    let allocator = Allocator::new();
    let dir = DirectiveNode::new(&allocator, "slot", SourceLocation::STUB);
    assert_eq!(get_slot_name(&dir, "").as_str(), "default");
}

#[test]
fn test_collect_slots() {
    let allocator = Allocator::new();
    let source = r#"<Comp><template #header>H</template><template #footer>F</template></Comp>"#;
    let (root, _) = parse(&allocator, source);

    if let TemplateChildNode::Element(el) = &root.children[0] {
        let slots = collect_slots(el, source);
        assert_eq!(slots.len(), 2);
        assert!(slots.iter().any(|s| s.name == "header"));
        assert!(slots.iter().any(|s| s.name == "footer"));
    }
}

#[test]
fn test_collect_slots_dedupes_static_duplicate_slot_names() {
    let allocator = Allocator::new();
    let source = r#"<Comp><template #header>H1</template><template #header>H2</template></Comp>"#;
    let (root, _) = parse(&allocator, source);

    if let TemplateChildNode::Element(el) = &root.children[0] {
        let slots = collect_slots(el, source);
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].name, "header");
    }
}

#[test]
fn test_v_slot_on_plain_element_reports_misplaced() {
    let errors = transform_errors(r#"<div v-slot="{ item }">Text</div>"#);

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, ErrorCode::VSlotMisplaced);
}

#[test]
fn test_v_slot_on_empty_plain_element_reports_misplaced() {
    let errors = transform_errors(r#"<div v-slot></div>"#);

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, ErrorCode::VSlotMisplaced);
}

#[test]
fn test_duplicate_slot_names_report_error() {
    let errors = transform_errors(
        r#"<Comp><template #header>H1</template><template #header>H2</template></Comp>"#,
    );

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, ErrorCode::VSlotDuplicateSlotNames);
}

#[test]
fn test_mixed_component_and_template_slot_usage_reports_error() {
    let errors = transform_errors(r#"<Comp v-slot><template #header>Header</template></Comp>"#);

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, ErrorCode::VSlotMixedSlotUsage);
}

#[test]
fn test_explicit_default_slot_with_children_reports_error() {
    let errors =
        transform_errors(r#"<Comp><template #default>Default</template><span>Extra</span></Comp>"#);

    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].code,
        ErrorCode::VSlotExtraneousDefaultSlotChildren
    );
}

#[test]
fn test_explicit_default_slot_allows_whitespace_children() {
    let errors = transform_errors("<Comp><template #default>Default</template>\n  \t\n</Comp>");

    assert!(errors.is_empty(), "Unexpected errors: {:?}", errors);
}

#[test]
fn test_custom_directive_on_slot_outlet_reports_error() {
    let errors = transform_errors(r#"<slot v-custom />"#);

    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].code,
        ErrorCode::VSlotUnexpectedDirectiveOnSlotOutlet
    );
}

#[test]
fn test_get_slot_prop_names_from_directive() {
    let allocator = Allocator::new();
    let source = r#"<Comp><template #default="{ item, active }">{{ item.id }}{{ active }}</template></Comp>"#;
    let (root, _) = parse(&allocator, source);

    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let TemplateChildNode::Element(slot_template) = &el.children[0] {
            let dir = slot_template
                .props
                .iter()
                .find_map(|prop| match prop {
                    crate::PropNode::Directive(dir) if dir.name == "slot" => Some(dir),
                    _ => None,
                })
                .expect("expected v-slot directive");
            let names = get_slot_prop_names(dir, source);
            let names: Vec<_> = names.iter().map(|name| name.as_str()).collect();
            assert_eq!(names, vec!["item", "active"]);
        } else {
            panic!("expected slot template element");
        }
    } else {
        panic!("expected component root element");
    }
}

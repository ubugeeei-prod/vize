use super::super::{Drawer, DrawerOptions};
use crate::scope::ScopeData;
use vize_armature::parse;
use vize_carton::{Allocator, cstr};

#[test]
fn v_slot_object_pattern_declaration_offsets_use_local_bindings() {
    let pattern = "  { name: name, label: local }";
    let template =
        cstr!(r#"<p>前置き</p><Child v-slot="{pattern}">{{{{ name }}}}{{{{ local }}}}</Child>"#,);
    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, template.as_str());
    assert!(errors.is_empty(), "template should parse: {errors:?}");
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_template(&root);
    let summary = drawer.finish();
    let scope = summary
        .scopes
        .iter()
        .find(|scope| matches!(scope.data(), ScopeData::VSlot(_)))
        .expect("v-slot scope should be recorded");
    let pattern_offset = template.find(pattern).unwrap() as u32;
    assert_eq!(
        summary.scopes.v_slot_pattern_offset(scope.id),
        Some(pattern_offset)
    );

    assert_eq!(
        scope.get_binding("name").unwrap().declaration_offset,
        pattern_offset + pattern.rfind("name").unwrap() as u32
    );
    assert_eq!(
        scope.get_binding("local").unwrap().declaration_offset,
        pattern_offset + pattern.find("local").unwrap() as u32
    );
}

#[test]
fn slot_declarations_address_complete_authored_attribute_bytes() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/slot-declaration-physical-8142/cases.json"
    ))
    .expect("independently authored complete coordinate vectors");
    let cases = fixture["cases"]
        .as_array()
        .expect("twenty complete sources");
    assert_eq!(cases.len(), 20);
    let mut packets = Vec::new();
    for case in cases {
        let source = case["source"].as_str().expect("whole authored source");
        let allocator = Allocator::new();
        let (root, errors) = parse(&allocator, source);
        let mut drawer = Drawer::with_options(DrawerOptions::full());
        if case["legacy"] == true {
            drawer = drawer.with_legacy_vue2();
        }
        drawer.draw_template(&root);
        let summary = drawer.finish();
        let slots: Vec<_> = summary
            .scopes
            .iter()
            .filter_map(|scope| {
                let ScopeData::VSlot(data) = scope.data() else {
                    return None;
                };
                let mut declarations: Vec<_> = scope
                    .bindings()
                    .map(|(name, binding)| (name, binding.declaration_offset))
                    .collect();
                declarations.sort_unstable();
                Some(serde_json::json!({
                    "name": data.name,
                    "component": data.component,
                    "decodedPattern": data.props_pattern,
                    "declarations": declarations,
                }))
            })
            .collect();
        let parser_errors: Vec<_> = errors
            .iter()
            .map(|error| {
                serde_json::json!({
                    "code": error.code as u16,
                    "message": error.message.as_str(),
                    "loc": &error.loc,
                })
            })
            .collect();
        packets.push(serde_json::json!({
            "id": case["id"], "source": source,
            "parserErrors": parser_errors,
            "parserErrorsDebug": cstr!("{errors:#?}"), "slots": slots,
        }));
    }
    // Keep every original input and complete coordinate packet before judging
    // any result; compiler/runtime entity gaps are qualified separately.
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/differential/slot-declaration-physical-8142");
    std::fs::create_dir_all(&directory).expect("complete coordinate custody");
    std::fs::write(
        directory.join("croquis-packets.json"),
        serde_json::to_vec_pretty(&packets).expect("whole coordinate packets"),
    )
    .expect("retain actual packets before assertions");
    for (case, actual) in cases.iter().zip(&packets) {
        assert_eq!(
            actual["parserErrors"], case["parserErrors"],
            "{}",
            case["id"]
        );
        assert_eq!(
            actual["slots"],
            serde_json::json!([{
                "name": "default", "component": case["component"],
                "decodedPattern": case["decodedPattern"],
                "declarations": case["declarations"],
            }]),
            "{} complete authored binding ownership and byte positions",
            case["id"],
        );
    }
}

#[test]
fn invalid_authored_attribute_windows_do_not_invent_declaration_offsets() {
    use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};

    let source = "<p>🚀</p><Panel v-slot=\"{ item }\">{{ item }}</Panel>";
    for (start, end) in [(4, 37), (26, source.len() as u32 + 1), (37, 26)] {
        let allocator = Allocator::new();
        let (mut root, errors) = parse(&allocator, source);
        assert!(errors.is_empty());
        let slot = root
            .children
            .iter_mut()
            .find_map(|child| {
                let TemplateChildNode::Element(element) = child else {
                    return None;
                };
                element.props.iter_mut().find_map(|prop| {
                    let PropNode::Directive(directive) = prop else {
                        return None;
                    };
                    if directive.name != "slot" {
                        return None;
                    }
                    let Some(ExpressionNode::Simple(value)) = &mut directive.exp else {
                        return None;
                    };
                    Some(value)
                })
            })
            .expect("real parser-produced slot before deliberate invalid window");
        slot.loc.span.start = start;
        slot.loc.span.end = end;
        let mut drawer = Drawer::with_options(DrawerOptions::full());
        drawer.draw_template(&root);
        let summary = drawer.finish();
        let declarations: Vec<_> = summary
            .scopes
            .iter()
            .filter(|scope| matches!(scope.data(), ScopeData::VSlot(_)))
            .flat_map(|scope| scope.bindings())
            .collect();
        assert!(
            declarations.is_empty(),
            "unsupported source window {start}..{end}"
        );
    }
}

/// `<template #name="">` is a slot function without parameters for Vue's
/// compiler. The scope keeps no props pattern for it either, so the generated
/// slot function declares its own parameter instead of an empty pattern.
#[test]
fn empty_v_slot_expression_declares_no_slot_props() {
    for template in [
        r#"<Child><template #text="">{{ label }}</template></Child>"#,
        r#"<Child><template #text="  ">{{ label }}</template></Child>"#,
        r#"<Child><template v-slot:text="">{{ label }}</template></Child>"#,
    ] {
        let allocator = Allocator::new();
        let (root, errors) = parse(&allocator, template);
        assert!(errors.is_empty(), "template should parse: {errors:?}");
        let mut drawer = Drawer::with_options(DrawerOptions::full());
        drawer.draw_template(&root);
        let summary = drawer.finish();
        let (scope, data) = summary
            .scopes
            .iter()
            .find_map(|scope| match scope.data() {
                ScopeData::VSlot(data) => Some((scope, data)),
                _ => None,
            })
            .expect("v-slot scope should be recorded");
        assert_eq!(data.name, "text", "{template}");
        assert_eq!(data.props_pattern, None, "{template}");
        assert!(data.prop_names.is_empty(), "{template}");
        assert_eq!(
            summary.scopes.v_slot_pattern_offset(scope.id),
            None,
            "{template}"
        );
        assert!(scope.bindings().next().is_none(), "{template}");
    }
}

#[test]
fn empty_legacy_slot_scope_declares_no_slot_props() {
    let template = r#"<Child><template slot="text" slot-scope="">{{ label }}</template></Child>"#;
    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, template);
    assert!(errors.is_empty(), "template should parse: {errors:?}");
    let mut drawer = Drawer::with_options(DrawerOptions::full()).with_legacy_vue2();
    drawer.draw_template(&root);
    let summary = drawer.finish();
    let (scope, data) = summary
        .scopes
        .iter()
        .find_map(|scope| match scope.data() {
            ScopeData::VSlot(data) => Some((scope, data)),
            _ => None,
        })
        .expect("slot-scope must create a v-slot semantic scope");
    assert_eq!(data.name, "text");
    assert_eq!(data.props_pattern, None);
    assert!(data.prop_names.is_empty());
    assert_eq!(summary.scopes.v_slot_pattern_offset(scope.id), None);
}

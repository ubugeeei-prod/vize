//! Computed component prop and event names retain their own expression AST.

use super::{VaporL3BridgeStatus, lower_source_for_vapor, options};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[test]
fn computed_component_names_match_retained_generation() {
    for source in [
        r#"<Child :[propName]="value" @[eventName]="record" />"#,
        r#"<Child label="static" :[label]="value" @[eventName]="record" />"#,
        r#"<component :is="view" :[is]="value" @[eventName]="record" />"#,
        r#"<Child :[names[selected]]="value" @[events[selected]]="record(value)" />"#,
        r#"<Child :[name.toLowerCase()]="value" @['saved-'+suffix]="(v) => record(v)" />"#,
        r#"<Child v-slot="{ item }"><Other :[item.prop]="item.value" @[item.event]="save" /></Child>"#,
        r#"<Child :[propName]="value" v-model:title="form.title" @[eventName]="record" />"#,
    ] {
        let allocator = Allocator::new();
        assert!(
            matches!(
                lower_source_for_vapor(&allocator, source, options()),
                VaporL3BridgeStatus::Accepted(_)
            ),
            "{source}"
        );
        for prefix_identifiers in [false, true] {
            let compile = |davinci_retained_lane| {
                compile_vapor(
                    &allocator,
                    source,
                    VaporCompilerOptions {
                        prefix_identifiers,
                        davinci_retained_lane,
                        ..Default::default()
                    },
                )
            };
            let native = compile(false);
            let retained = compile(true);
            assert!(
                native.error_messages.is_empty(),
                "{source}: {:?}",
                native.error_messages
            );
            assert_eq!(
                native.code, retained.code,
                "{source}: prefix={prefix_identifiers}"
            );
            assert_eq!(native.templates, retained.templates, "{source}");
        }
    }
}

#[test]
fn unproved_computed_name_surfaces_remain_legacy() {
    for source in [
        r#"<button :[name].camel="value">text</button>"#,
        r#"<slot :[name].camel="value" />"#,
        r#"<Child @[event].stop="save" />"#,
        r#"<Child :[name].camel="value" />"#,
    ] {
        let allocator = Allocator::new();
        assert!(
            matches!(
                lower_source_for_vapor(&allocator, source, options()),
                VaporL3BridgeStatus::Legacy(_)
            ),
            "{source}"
        );
    }
}

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "insta formats generated-code snapshots"
)]
fn checked_component_name_payload_owns_generated_keys() {
    let allocator = Allocator::new();
    let mut s3 = super::lowered_source(
        &allocator,
        r#"<Child :[propName]="value" @[eventName]="save" />"#,
    );
    for operand in &mut s3.program.operands {
        if operand.role == vize_l3::operand::OperandRole::Name {
            match operand.value.text {
                "propName" => operand.value.text = "otherProp",
                "eventName" => operand.value.text = "otherEvent",
                _ => {}
            }
        }
    }
    let code = super::generated(
        crate::l3::admit(s3, &crate::l3::retained::Retained::new(&allocator)),
        &allocator,
    );
    insta::assert_snapshot!("component_name_payload", code);
}

#[test]
fn native_static_vue_casts_admit_reserved_component_names() {
    for source in [
        "<div is=\"vue:component\" title=\"cast\">x</div>",
        "<div is=\"vue:Component\" title=\"cast\">x</div>",
        "<div is=\"vue:slot\" title=\"cast\">x</div>",
        "<div is=\"vue:template\" title=\"cast\">x</div>",
        "<componentFoo is=\"vue:component\">x</componentFoo>",
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        let VaporL3BridgeStatus::Accepted(artifact) = status else {
            panic!("native static cast {source}: {status:?}")
        };
        let ir = artifact.into_ir(&allocator, source, None).unwrap();
        let [crate::ir::OperationNode::CreateComponent(component)] = ir.block.operation.as_slice()
        else {
            panic!("one static component operation")
        };
        assert_eq!(component.kind, crate::ir::ComponentKind::Regular);
        assert!(component.asset);
        assert!(component.is_expr.is_none());
        assert_eq!(component.slots.len(), 1);
        let native = compile_vapor(&allocator, source, VaporCompilerOptions::default());
        let retained = compile_vapor(
            &allocator,
            source,
            VaporCompilerOptions {
                davinci_retained_lane: true,
                ..Default::default()
            },
        );
        assert!(
            native.error_messages.is_empty(),
            "{:?}",
            native.error_messages
        );
        assert!(
            retained.error_messages.is_empty(),
            "{:?}",
            retained.error_messages
        );
        assert_eq!(native.code, retained.code);
        assert_eq!(native.templates, retained.templates);
        assert_eq!(native.map, retained.map);
    }
}

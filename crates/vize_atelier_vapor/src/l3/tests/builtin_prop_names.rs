//! Computed prop names on Teleport, KeepAlive and Suspense.

use super::{VaporL3BridgeStatus, lower_source_for_vapor, options};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

const SOURCES: &[&str] = &[
    r#"<Teleport to="body" :[disabled]="value"><span>content</span></Teleport>"#,
    r#"<Teleport to="body" disabled :[disabled]="value"><span>{{ label }}</span></Teleport>"#,
    r#"<Teleport :to="targets[selected]" :[flags[index]]="on" defer><span>content</span></Teleport>"#,
    r#"<Teleport to="body" :[name.toLowerCase()]="value" :['extra-'+suffix]="extra"><button>{{ label }}</button></Teleport>"#,
    r#"<KeepAlive :[include]="names"><MyComp /></KeepAlive>"#,
    r#"<KeepAlive include="First" :[include]="names" :[filters[index]]="limit"><MyComp :label="label" /></KeepAlive>"#,
    r#"<KeepAlive :[name.toLowerCase()]="names" :max="limit"><component :is="view" /></KeepAlive>"#,
    r#"<Suspense :[timeout]="limit"><AsyncChild /></Suspense>"#,
    r#"<Suspense timeout="0" :[timeout]="limit" @resolve="record"><template #default><AsyncChild /></template><template #fallback><p>{{ waiting }}</p></template></Suspense>"#,
    r#"<Suspense :[limits[index]]="delay" @pending="record('pending')"><section><AsyncChild :label="label" /></section></Suspense>"#,
];

#[test]
fn builtin_computed_prop_names_match_retained_generation() {
    for source in SOURCES {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporL3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
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
            assert!(
                retained.error_messages.is_empty(),
                "{source}: {:?}",
                retained.error_messages
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
fn unproved_builtin_computed_names_remain_legacy() {
    for source in [
        r#"<Teleport :[to]="target"><span>content</span></Teleport>"#,
        r#"<Teleport :[disabled]="value"><span>content</span></Teleport>"#,
        r#"<Teleport to="body" :[disabled].camel="value"><span>content</span></Teleport>"#,
        r#"<Teleport to="body" @[event]="save"><span>content</span></Teleport>"#,
        r#"<Teleport to="body" :unknown="value"><span>content</span></Teleport>"#,
        r#"<KeepAlive :[include].prop="names"><MyComp /></KeepAlive>"#,
        r#"<KeepAlive @[change]="save"><MyComp /></KeepAlive>"#,
        r#"<Suspense :[timeout].attr="limit"><AsyncChild /></Suspense>"#,
        r#"<Suspense @[pending]="record"><AsyncChild /></Suspense>"#,
        r#"<Suspense suspensible :[timeout]="limit"><AsyncChild /></Suspense>"#,
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporL3BridgeStatus::Legacy(_)),
            "{source}: {status:?}"
        );
    }
}

#[test]
fn checked_builtin_prop_name_payload_owns_the_key() {
    let allocator = Allocator::new();
    let mut s3 = super::lowered_source(
        &allocator,
        r#"<Teleport to="body" :[disabled]="value"><span>content</span></Teleport>"#,
    );
    let name = s3
        .program
        .operands
        .iter_mut()
        .find(|operand| {
            operand.role == vize_l3::operand::OperandRole::Name && operand.value.text == "disabled"
        })
        .unwrap();
    name.value.text = "otherFlag";
    let code = super::generated(
        crate::l3::admit(s3, &crate::l3::retained::Retained::new(&allocator)),
        &allocator,
    );
    assert!(code.contains("otherFlag"), "{code}");
    assert!(!code.contains("disabled"), "{code}");
}

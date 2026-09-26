//! Teleport's checked kind selects the real Vapor runtime component.

use super::{VaporS3BridgeStatus, lower_source_for_vapor, options};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[test]
fn teleport_props_and_default_content_match_retained_generation() {
    for source in [
        r#"<Teleport to="body"><div>content</div></Teleport>"#,
        r#"<Teleport :to="target" :disabled="disabled" defer><button @click="save">{{ label }}</button></Teleport>"#,
        r#"<Teleport :to="targets[selected]" :defer="deferred"><span v-if="visible">{{ value }}</span><span v-else>fallback</span></Teleport>"#,
        r#"<Teleport to="body"><ul><li v-for="item in items" :key="item.id">{{ item.label }}</li></ul></Teleport>"#,
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporS3BridgeStatus::Accepted(_)),
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
            assert!(native.error_messages.is_empty(), "{source}");
            assert_eq!(
                native.code,
                compile(true).code,
                "{source}: prefix={prefix_identifiers}"
            );
        }
    }
}

#[test]
fn unproved_builtin_contracts_keep_explicit_legacy_routes() {
    for source in [
        r#"<teleport to="body"><span>content</span></teleport>"#,
        r#"<Teleport><span>missing target</span></Teleport>"#,
        r#"<Teleport to="body" @click="save"><span>content</span></Teleport>"#,
        r#"<Teleport v-bind="props"><span>content</span></Teleport>"#,
        r#"<Teleport to="body" v-model="value"><span>content</span></Teleport>"#,
        r#"<Teleport to="body" v-slot="p">{{ p.value }}</Teleport>"#,
        r#"<Teleport to="body"><template #default><span>content</span></template></Teleport>"#,
        r#"<KeepAlive><MyComp /><AnotherComp /></KeepAlive>"#,
        r#"<Suspense><MyComp /></Suspense>"#,
        r#"<Transition><span>content</span></Transition>"#,
    ] {
        let allocator = Allocator::new();
        assert!(
            matches!(
                lower_source_for_vapor(&allocator, source, options()),
                VaporS3BridgeStatus::Legacy(_)
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
fn checked_teleport_payload_owns_runtime_kind_and_target() {
    let allocator = Allocator::new();
    let mut s3 = super::lowered_source(
        &allocator,
        r#"<Teleport to="body"><div>content</div></Teleport>"#,
    );
    let target = s3
        .program
        .operands
        .iter_mut()
        .find(|operand| {
            operand.role == vize_s3::operand::OperandRole::Attribute && operand.name == Some("to")
        })
        .unwrap();
    target.value.text = "#replacement";
    let code = super::generated(
        crate::s3::admit(s3, &crate::s3::retained::Retained::new(&allocator)),
        &allocator,
    );
    insta::assert_snapshot!("teleport_payload", code);
}

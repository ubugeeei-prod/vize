//! Checked Suspense slots preserve the renderer primitive and retained output.

use super::{VaporS3BridgeStatus, lower_source_for_vapor, options};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[test]
fn checked_suspense_generation_matches_retained_lane() {
    for source in [
        r#"<Suspense><AsyncChild /></Suspense>"#,
        r#"<Suspense timeout="0"><template #default><AsyncChild :label="label" /></template><template #fallback><p>{{ waiting }}</p></template></Suspense>"#,
        r#"<main><Suspense :timeout="limit" @pending="record('pending')" @fallback="record('fallback')" @resolve="record('resolve')"><template #default><section><AsyncChild /></section></template><template #fallback><p>waiting</p></template></Suspense><i>tail</i></main>"#,
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
            assert!(native.code.contains("_createComponent(_Suspense,"));
            assert!(!native.code.contains("withVaporCtx"));
            assert!(!native.code.contains("_resolveComponent(\"Suspense\")"));
        }
    }
}

#[test]
fn unproved_suspense_shapes_keep_explicit_legacy_routes() {
    for source in [
        r#"<suspense><AsyncChild /></suspense>"#,
        r#"<Suspense />"#,
        r#"<Suspense><AsyncChild /><OtherChild /></Suspense>"#,
        r#"<Suspense><AsyncChild /><template #fallback><p>waiting</p></template></Suspense>"#,
        r#"<Suspense><template #fallback><p>waiting</p></template></Suspense>"#,
        r#"<Suspense><template #default="p"><AsyncChild /></template></Suspense>"#,
        r#"<Suspense><template #[name]><AsyncChild /></template></Suspense>"#,
        r#"<Suspense><template #default><AsyncChild /></template><template #other><p>other</p></template></Suspense>"#,
        r#"<Suspense :[timeout]="limit"><AsyncChild /></Suspense>"#,
        r#"<Suspense suspensible><AsyncChild /></Suspense>"#,
        r#"<Suspense v-bind="props"><AsyncChild /></Suspense>"#,
        r#"<Suspense @unknown="save"><AsyncChild /></Suspense>"#,
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporS3BridgeStatus::Legacy(_)),
            "{source}: {status:?}"
        );
    }
}

#[test]
fn checked_suspense_payload_owns_its_timeout() {
    let allocator = Allocator::new();
    let mut s3 = super::lowered_source(
        &allocator,
        r#"<Suspense timeout="0"><AsyncChild /></Suspense>"#,
    );
    let timeout = s3
        .program
        .operands
        .iter_mut()
        .find(|operand| {
            operand.role == vize_s3::operand::OperandRole::Attribute
                && operand.name == Some("timeout")
        })
        .unwrap();
    timeout.value.text = "12";
    let code = super::generated(
        crate::s3::admit(s3, &crate::s3::retained::Retained::new(&allocator)),
        &allocator,
    );
    assert!(code.contains("timeout: () => (\"12\")"), "{code}");
    assert!(code.contains("_createComponent(_Suspense,"), "{code}");
}

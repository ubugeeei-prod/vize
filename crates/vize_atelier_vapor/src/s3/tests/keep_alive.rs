//! A checked cache boundary preserves the runtime component kind.

use super::{VaporS3BridgeStatus, lower_source_for_vapor, options};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[test]
fn cached_component_generation_matches_retained_lane() {
    for source in [
        r#"<KeepAlive><MyComp /></KeepAlive>"#,
        r#"<KeepAlive include="First" max="2"><component :is="view" :label="label" @send="record" /></KeepAlive>"#,
        r#"<KeepAlive :include="names" :exclude="excluded" :max="limit"><component :is="views[selected]" v-model="value" /></KeepAlive>"#,
    ] {
        let allocator = Allocator::new();
        assert!(
            matches!(
                lower_source_for_vapor(&allocator, source, options()),
                VaporS3BridgeStatus::Accepted(_)
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
fn unproved_cache_boundaries_keep_explicit_legacy_routes() {
    for source in [
        r#"<keep-alive><MyComp /></keep-alive>"#,
        r#"<KeepAlive />"#,
        r#"<KeepAlive><MyComp /><OtherComp /></KeepAlive>"#,
        r#"<KeepAlive><div>content</div></KeepAlive>"#,
        r#"<KeepAlive><MyComp v-if="visible" /></KeepAlive>"#,
        r#"<KeepAlive><component :is="view" :key="key" /></KeepAlive>"#,
        r#"<KeepAlive><template #default><MyComp /></template></KeepAlive>"#,
        r#"<KeepAlive v-bind="props"><MyComp /></KeepAlive>"#,
        r#"<KeepAlive @change="save"><MyComp /></KeepAlive>"#,
        r#"<KeepAlive :unknown="value"><MyComp /></KeepAlive>"#,
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
    reason = "insta formats exact generated-code snapshots"
)]
fn checked_cache_payload_owns_the_filter() {
    let allocator = Allocator::new();
    let mut s3 = super::lowered_source(
        &allocator,
        r#"<KeepAlive include="First"><First /></KeepAlive>"#,
    );
    let include = s3
        .program
        .operands
        .iter_mut()
        .find(|operand| {
            operand.role == vize_s3::operand::OperandRole::Attribute
                && operand.name == Some("include")
        })
        .unwrap();
    include.value.text = "Second";
    let code = super::generated(
        crate::s3::admit(s3, &crate::s3::retained::Retained::new(&allocator)),
        &allocator,
    );
    insta::assert_snapshot!("keep_alive_payload", code);
}

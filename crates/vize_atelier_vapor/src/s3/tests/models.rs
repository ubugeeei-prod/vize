use super::options;
use crate::s3::{LegacyReason, VaporS3BridgeStatus, lower_source_for_vapor};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

/// Input models match the retained lane byte for byte at the template root,
/// with and without prefixing.
#[test]
fn input_models_match_the_retained_lane() {
    for source in [
        r#"<input v-model="msg">"#,
        r#"<input type="checkbox" v-model.lazy.trim="form.agree">"#,
        r#"<input type="radio" value="a" v-model="picked">"#,
        r#"<input type="number" v-model.number="age">"#,
        r#"<input v-model="msg" @keydown.enter="save" :placeholder="hint">"#,
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporS3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
        );
        for prefix_identifiers in [false, true] {
            let compile = |retained| {
                compile_vapor(
                    &allocator,
                    source,
                    VaporCompilerOptions {
                        prefix_identifiers,
                        davinci_retained_lane: retained,
                        ..Default::default()
                    },
                )
                .code
            };
            assert_eq!(compile(false), compile(true), "{source}");
        }
    }
}

#[test]
fn unsupported_models_select_exact_legacy_reasons() {
    for source in [
        r#"<input v-model="items[i]">"#,
        r#"<input v-model="$event">"#,
        r#"<input v-model="_ctx.x">"#,
        r#"<input v-model.foo="x">"#,
        r#"<input :type="kind" v-model="x">"#,
        r#"<input v-model="a" v-model="b">"#,
        r#"<MyComp v-model="x" />"#,
        r#"<MyComp v-model:title="x" />"#,
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(
                status,
                VaporS3BridgeStatus::Legacy(LegacyReason::Binding | LegacyReason::SurfaceSemantics)
            ),
            "{source}: {status:?}"
        );
    }
}

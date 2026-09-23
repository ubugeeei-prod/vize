use super::{lowered_source, options};
use crate::s3::{
    LegacyReason, VaporS3BridgeStatus, admit, lower_source_for_vapor, retained::Retained,
};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;
use vize_s3::operand::OperandRole;

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
fn empty_textarea_models_match_the_retained_lane() {
    for source in [
        r#"<textarea v-model="content"></textarea>"#,
        r#"<textarea v-model.lazy.trim="form.message" placeholder="Type here"></textarea>"#,
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
fn textarea_contents_and_other_shapes_stay_legacy() {
    for source in [
        r#"<textarea v-model="content">initial</textarea>"#,
        r#"<textarea v-model="content">{{ content }}</textarea>"#,
        r#"<textarea v-model="content" value="initial"></textarea>"#,
        r#"<textarea v-model="content" :value="initial"></textarea>"#,
        r#"<textarea v-model="content" v-text="other"></textarea>"#,
        r#"<textarea v-model="content" v-html="other"></textarea>"#,
        r#"<textarea></textarea>"#,
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
fn model_element_kind_must_match_its_target() {
    for (source, stale_kind) in [
        (r#"<input v-model="value">"#, "textarea"),
        (r#"<textarea v-model="value"></textarea>"#, "input"),
    ] {
        let allocator = Allocator::new();
        let mut s3 = lowered_source(&allocator, source);
        let kind = s3
            .program
            .operands
            .iter_mut()
            .find(|operand| {
                operand.role == OperandRole::ModelAttribute && operand.name == Some("element-kind")
            })
            .expect("model element kind");
        kind.value.text = stale_kind;
        let status = admit(s3, &Retained::new(&allocator));
        assert!(
            matches!(status, VaporS3BridgeStatus::Rejected(_)),
            "{source}: {status:?}"
        );
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

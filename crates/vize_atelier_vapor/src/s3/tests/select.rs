//! Model-bound select inputs retain DOM parsing and assignment contracts.

use super::{lowered_source, options};
use crate::s3::{VaporS3BridgeStatus, admit, lower_source_for_vapor, retained::Retained};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "insta formats generated-code snapshots"
)]
fn select_models_match_the_retained_lane() {
    for (case, source) in [
        r#"<select v-model="selected"><option value="a">A</option><option value="b">B</option></select>"#,
        r#"<select multiple v-model="selected"><option :value="first">{{ label }}</option><option :value="second">B</option></select>"#,
        r#"<select v-model.number="form.selected"><option value="1">One</option><option value="2">Two</option></select>"#,
        r#"<select v-model="values[key]"><option value="a">A</option><option value="b">B</option></select>"#,
        r#"<select v-model="selected"><option :value="item" v-text="label"></option></select>"#,
        r#"<select v-model="selected"></select>"#,
    ].into_iter().enumerate() {
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
            assert!(native.error_messages.is_empty(), "{source}");
            let retained = compile(true);
            assert_eq!(native.templates, retained.templates, "{source}");
            if matches!(case, 1 | 4) {
                // The native tree numbers the select before dynamic option
                // children; the retained lane reserves those children first.
                if prefix_identifiers {
                    insta::assert_snapshot!(std::format!("select_model_{case}"), native.code);
                }
            } else {
                assert_eq!(native.code, retained.code, "{source}: prefix={prefix_identifiers}");
            }
        }
    }
}

#[test]
fn unproved_select_parsing_shapes_stay_on_explicit_legacy_routes() {
    for source in [
        r#"<select><option value="a">A</option></select>"#,
        r#"<option value="a">A</option>"#,
        r#"<select v-model="value"><optgroup label="group"><option>A</option></optgroup></select>"#,
        r#"<select v-model="value"><option v-for="item in items">{{ item }}</option></select>"#,
        r#"<select v-model="value"><option v-if="ok">A</option></select>"#,
        r#"<select v-model="value"><MyOption /></select>"#,
        r#"<select v-model="value"><option><span>A</span></option></select>"#,
        r#"<select v-model="value" v-html="html"></select>"#,
        r#"<select v-model="value"><option v-html="html"></option></select>"#,
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
fn checked_select_payload_owns_the_model_assignment() {
    let allocator = Allocator::new();
    let mut s3 = lowered_source(
        &allocator,
        r#"<select v-model="original"><option value="a">A</option></select>"#,
    );
    for operand in &mut s3.program.operands {
        if matches!(
            operand.role,
            vize_s3::operand::OperandRole::ModelRead | vize_s3::operand::OperandRole::ModelWrite
        ) {
            operand.value.text = "replacement";
        }
    }
    let code = super::generated(admit(s3, &Retained::new(&allocator)), &allocator);
    insta::assert_snapshot!("select_model_payload", code);
}

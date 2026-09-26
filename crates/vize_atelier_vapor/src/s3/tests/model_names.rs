//! Computed component model names own the authored key AST.

use super::{VaporS3BridgeStatus, lower_source_for_vapor, options};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[test]
fn computed_component_model_names_match_retained_generation() {
    for source in [
        r#"<Child v-model:[name]="value" />"#,
        r#"<Child v-model:[names[index]].trim.number="form[field]" />"#,
        r#"<Child v-model:[name.toLowerCase()].custom="value" />"#,
        r#"<Child v-model:[enabled?first:second]="items[index]" />"#,
        r#"<Child v-model:['prefix-'+suffix].trim="form.title" />"#,
        r#"<component :is="view" title="static" v-model:[name].trim="value" />"#,
        r#"<Outer v-slot="{ item }"><Child v-model:[item.name].trim="form[item.key]" /></Outer>"#,
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
fn computed_dom_model_arguments_remain_legacy() {
    for source in [
        r#"<input v-model:[name]="value" />"#,
        r#"<textarea v-model:[name]="value"></textarea>"#,
        r#"<select v-model:[name]="value"><option>A</option></select>"#,
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
fn checked_model_name_payload_owns_all_three_generated_keys() {
    let allocator = Allocator::new();
    let mut s3 = super::lowered_source(&allocator, r#"<Child v-model:[name].trim="value" />"#);
    for operand in &mut s3.program.operands {
        if operand.role == vize_s3::operand::OperandRole::Name {
            operand.value.text = "changedName";
        }
    }
    let code = super::generated(
        crate::s3::admit(s3, &crate::s3::retained::Retained::new(&allocator)),
        &allocator,
    );
    insta::assert_snapshot!("computed_model_name_payload", code);
}

//! Preserve the complete pinned group/conditional-template regression.

use super::{
    Report, SfcParseOptions, SsrCompilerExperimentalOptions, SsrCompilerOptions, assert_clean,
    compare_ssr_lanes, parse_sfc, record,
};

#[test]
fn original_group_template_retains_nested_fragment_context() {
    let source = include_str!("../fixtures/transition-group-dynamic-slot-with-v-if.vue");
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("original full SFC");
    let template = descriptor
        .template
        .as_ref()
        .expect("original template owner");
    let mut report = Report::default();
    record(
        "TransitionGroupDynamicSlot.vue",
        &compare_ssr_lanes(
            &template.content,
            &SsrCompilerOptions::default(),
            &SsrCompilerExperimentalOptions::default(),
        ),
        &mut report,
    );
    assert_eq!(report.compared, 1);
    assert_eq!(report.emitted(), 1);
    assert_clean("original group template", &report);
}

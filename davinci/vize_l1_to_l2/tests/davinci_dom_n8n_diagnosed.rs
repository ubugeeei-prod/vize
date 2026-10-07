//! A production-valid prefixed original is fully compared in its diagnosed mode.
mod davinci_dom_corpus_support;
use davinci_dom_corpus_support::{Lane, Report, compare_sfc_template_lane};

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/n8n-if-key-regression/InstanceAiConfirmationPanel.vue.txt"
);

#[test]
fn unchanged_original_has_complete_code_and_diagnostic_comparison_credit() {
    for lane in [Lane::Default, Lane::Prefixed, Lane::Bindings] {
        let mut report = Report::default();
        compare_sfc_template_lane("n8n_original_instance_ai", ORIGINAL, &mut report, lane);
        assert_eq!(report.files, 1);
        assert_eq!(report.templates, 1);
        assert_eq!(report.compared, 1, "{:?}", report.divergences);
        assert_eq!(report.old_error_skips, 0);
        assert_eq!(report.unexpected_old_error_skips, 0);
        assert_eq!(report.s2_refusal_count, 0, "{:?}", report.s2_refusals);
        assert_eq!(report.divergence_count, 0, "{:?}", report.divergences);
        assert_eq!(
            report.diagnosed_compared,
            u64::from(matches!(lane, Lane::Default))
        );
    }
}

#[test]
fn changed_original_bytes_do_not_qualify_as_diagnosed_comparison() {
    let changed = format!("{ORIGINAL}\n");
    let mut report = Report::default();
    compare_sfc_template_lane(
        "n8n_original_instance_ai",
        &changed,
        &mut report,
        Lane::Default,
    );
    assert_eq!(report.compared, 0);
    assert_eq!(report.diagnosed_compared, 0);
    assert_eq!(report.old_error_skips, 1);
    assert_eq!(report.unexpected_old_error_skips, 1);
}

//! A production-valid prefixed original is fully compared in its diagnosed mode.
#![expect(clippy::disallowed_types, reason = "whole differential report vectors")]
mod davinci_dom_corpus_support;
mod n8n_typed_prefixed;
use davinci_dom_corpus_support::{Lane, Report, compare_sfc_template_lane};

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/n8n-if-key-regression/InstanceAiConfirmationPanel.vue.txt"
);

#[test]
fn unchanged_original_has_complete_code_and_diagnostic_comparison_credit() {
    for lane in [Lane::Default, Lane::Bindings] {
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
fn unchanged_original_explicit_authored_ts_prefixed_recipe_is_fully_compared() {
    let mut report = Report::default();
    n8n_typed_prefixed::compare_authored_sfc_ts_prefixed(
        "n8n_original_instance_ai",
        ORIGINAL,
        &mut report,
    );
    assert_eq!(report.files, 1);
    assert_eq!(report.parsed, 1);
    assert_eq!(report.templates, 1);
    assert_eq!(report.compared, 1);
    assert_eq!(report.patch_fact_compared, 1);
    assert_eq!(report.diagnosed_compared, 0);
    assert_eq!(report.old_error_skips, 0);
    assert_eq!(report.unexpected_old_error_skips, 0);
    assert_eq!(report.s2_refusal_count, 0);
    assert_eq!(report.divergence_count, 0);
    assert_eq!(report.s2_refusals, Vec::<String>::new());
    assert_eq!(report.divergences, Vec::<String>::new());
}

#[test]
fn unchanged_original_wrong_js_recipe_retains_complete_refusal_contract() {
    n8n_typed_prefixed::assert_original_js_refusal("n8n_original_instance_ai", ORIGINAL);
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

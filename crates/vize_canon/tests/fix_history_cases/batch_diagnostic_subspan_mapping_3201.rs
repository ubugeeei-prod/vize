use crate::support::fix_history_runner::check_pack;

#[test]
fn batch_diagnostics_map_through_exact_expression_sub_spans() {
    check_pack(
        "batch-diagnostic-subspan-mapping-3201",
        "fix_history_cases::batch_diagnostic_subspan_mapping_3201::batch_diagnostics_map_through_exact_expression_sub_spans",
        "1acad3cafdff185d4a4f3fa5f23166ef3c4840b1",
        Some(3201),
        &[
            "bound-call-matches-prop",
            "bound-call-mismatch-stays-inside-attribute",
            "unknown-name-inside-bound-call-maps-exactly",
        ],
    );
}

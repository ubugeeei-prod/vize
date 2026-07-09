use vize_croquis_cf::ComplexityReport;

pub(crate) fn complexity_report_json(report: &ComplexityReport) -> serde_json::Value {
    serde_json::to_value(report).expect("complexity report should serialize")
}

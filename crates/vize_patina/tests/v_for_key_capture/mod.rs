//! Complete public API packets retained in the existing Rust shard artifact.

use serde_json::{Value, json};
use vize_patina::{LintDiagnostic, LintResult};

fn diagnostic_packet(diagnostic: &LintDiagnostic) -> Value {
    let LintDiagnostic {
        rule_name,
        severity,
        message,
        start,
        end,
        help,
        labels,
        fix,
    } = diagnostic;
    json!({
        "rule_name": rule_name,
        "severity": severity,
        "message": message,
        "start": start,
        "end": end,
        "help": help,
        "labels": labels.iter().map(|label| json!({
            "message": label.message,
            "start": label.start,
            "end": label.end,
        })).collect::<Vec<_>>(),
        "fix": fix,
    })
}

pub(super) fn whole_packet(result: &LintResult) -> Value {
    let LintResult {
        filename,
        diagnostics,
        error_count,
        warning_count,
    } = result;
    json!({
        "filename": filename,
        "diagnostics": diagnostics.iter().map(diagnostic_packet).collect::<Vec<_>>(),
        "error_count": error_count,
        "warning_count": warning_count,
    })
}

pub(super) fn write_capture(filename: &str, packets: &[Value]) {
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/nextest/pr");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join(filename),
        serde_json::to_vec_pretty(packets).unwrap(),
    )
    .unwrap();
}

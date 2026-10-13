//! Every whole real SFC behind the release's 72 component-prop false positives.

use serde_json::{Value, json};
use std::path::Path;
use vize_patina::Linter;

const RULE: &str = "vue/no-unused-components";
const ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/component-positive-template-corpus/",
);
const CASES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/component-positive-template-corpus/cases.json",
));

#[test]
fn all_27_whole_sources_remove_72_false_positives_and_keep_six_unread_warnings() {
    let manifest: Value = serde_json::from_str(CASES).unwrap();
    let cases = manifest["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 27);
    assert_eq!(
        cases
            .iter()
            .map(|case| case["beforeWarnings"].as_u64().unwrap())
            .sum::<u64>(),
        78,
    );
    let linter = Linter::new().with_enabled_rules(Some(vec![RULE.into()]));
    let mut warnings = 0;
    let mut packets = Vec::new();
    let mut results = Vec::new();
    for case in cases {
        let id = case["id"].as_str().unwrap();
        let source =
            std::fs::read_to_string(Path::new(ROOT).join(case["source"].as_str().unwrap()))
                .unwrap();
        let result = linter.lint_sfc(&source, "WholeComponentRead.vue");
        packets.push(json!({"id": id, "packet": whole_packet(&result)}));
        results.push(result);
    }
    let artifact = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/nextest/pr/component-positive-template-corpus.json");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    std::fs::write(artifact, serde_json::to_vec_pretty(&packets).unwrap()).unwrap();
    for (case, result) in cases.iter().zip(&results) {
        let id = case["id"].as_str().unwrap();
        assert_eq!(result.error_count, 0, "{id}: {:?}", result.diagnostics);
        assert_eq!(
            result.warning_count as u64,
            case["expectedWarnings"].as_u64().unwrap(),
            "{id}: {:?}",
            result.diagnostics,
        );
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name == RULE)
        );
        warnings += result.warning_count;
    }
    assert_eq!(
        warnings, 6,
        "five unread setup imports and one Pug registration stay visible"
    );
}

fn whole_packet(result: &vize_patina::LintResult) -> Value {
    let vize_patina::LintResult {
        filename,
        diagnostics,
        error_count,
        warning_count,
    } = result;
    json!({
        "filename": filename,
        "diagnostics": diagnostics.iter().map(|diagnostic| {
            let vize_patina::LintDiagnostic {
                rule_name, severity, message, start, end, help, labels, fix,
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
        }).collect::<Vec<_>>(),
        "error_count": error_count,
        "warning_count": warning_count,
    })
}

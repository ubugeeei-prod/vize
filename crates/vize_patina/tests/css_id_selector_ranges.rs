//! Whole original SFC and authored CSS range controls through the public API.

use serde_json::{Value, json};
use std::{fs, path::Path};
use vize_patina::{HelpLevel, LintPreset, Linter, Locale, OutputFormat, format_results};

#[test]
fn complete_sfc_id_selector_ranges_and_public_reports() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/css-id-selector-ranges");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(fixture.join("cases.json")).expect("whole range manifest"),
    )
    .expect("valid range manifest");
    let cases = manifest["cases"].as_array().expect("all range cases");
    assert_eq!(cases.len(), 16, "every original and authored control");
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["css/no-id-selectors".into()]))
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::None);
    let mut observations = Vec::new();
    for case in cases {
        let source = fs::read_to_string(fixture.join(case["source"].as_str().unwrap()))
            .expect("whole SFC input");
        assert_eq!(Some(source.len() as u64), case["bytes"].as_u64());
        let filename = case["filename"].as_str().unwrap();
        let expected: Value = serde_json::from_str(
            &fs::read_to_string(fixture.join(case["expected"].as_str().unwrap()))
                .expect("complete reviewed output vector"),
        )
        .expect("valid whole output vector");
        let result = linter.lint_sfc(&source, filename);
        let diagnostics: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                let labels: Vec<_> = diagnostic
                    .labels
                    .iter()
                    .map(|label| {
                        json!({
                            "message": label.message, "start": label.start, "end": label.end
                        })
                    })
                    .collect();
                json!({
                    "rule": diagnostic.rule_name, "severity": diagnostic.severity,
                    "message": diagnostic.message, "start": diagnostic.start,
                    "end": diagnostic.end, "help": diagnostic.help,
                    "labels": labels, "fix": diagnostic.fix
                })
            })
            .collect();
        let actual = json!({
            "filename": result.filename, "diagnostics": diagnostics,
            "errorCount": result.error_count, "warningCount": result.warning_count
        });
        assert_eq!(actual, expected["api"], "{} whole API", case["id"]);
        let slices: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                source
                    .get(diagnostic.start as usize..diagnostic.end as usize)
                    .expect("every diagnostic selects complete authored UTF-8")
            })
            .collect();
        assert_eq!(json!(slices), expected["authoredSlices"], "{}", case["id"]);
        let sources = vec![(filename.into(), source.clone().into())];
        let results = [result];
        let public: Value =
            serde_json::from_str(&format_results(&results, &sources, OutputFormat::Json))
                .expect("whole public JSON report");
        assert_eq!(public, expected["json"], "{} whole JSON", case["id"]);
        let plain = format_results(&results, &sources, OutputFormat::Plain);
        assert_eq!(
            Some(plain.as_str()),
            expected["plain"].as_str(),
            "{}",
            case["id"]
        );
        observations.push(json!({
            "case": case, "source": source, "api": actual,
            "json": public, "plain": plain
        }));
    }
    let output = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/differential/css-id-selector-ranges-api.json");
    fs::create_dir_all(output.parent().unwrap()).expect("API receipt directory");
    fs::write(output, serde_json::to_vec_pretty(&observations).unwrap())
        .expect("retain every complete actual API observation");
}

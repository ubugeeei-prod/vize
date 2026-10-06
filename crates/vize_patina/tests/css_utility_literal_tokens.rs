//! Whole original #7980 inputs and independent literal/real-selector controls.

use serde_json::{Value, json};
use std::{fs, path::Path};
use vize_patina::{
    HelpLevel, LintPreset, LintResult, Linter, Locale, OutputFormat, format_results,
};

fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let labels: Vec<_> = diagnostic
                .labels
                .iter()
                .map(|label| {
                    json!({"message": label.message, "start": label.start, "end": label.end})
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
    json!({
        "filename": result.filename, "diagnostics": diagnostics,
        "errorCount": result.error_count, "warningCount": result.warning_count
    })
}

fn retain(observations: &[Value]) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let bytes = serde_json::to_vec_pretty(observations).expect("whole API observation bytes");
    let output = root.join("target/differential/css-utility-literal-tokens-api.json");
    fs::create_dir_all(output.parent().unwrap()).expect("API receipt directory");
    fs::write(output, &bytes).expect("whole API observations");
    if let Ok(profile) = std::env::var("NEXTEST_PROFILE")
        && ["pr", "full"].contains(&profile.as_str())
    {
        let retained = root
            .join("target/nextest")
            .join(profile)
            .join("css-utility-literal-tokens-api.json");
        fs::create_dir_all(retained.parent().unwrap()).expect("existing shard result directory");
        fs::write(retained, bytes).expect("whole API rows in the existing uploaded shard");
    }
}

#[test]
fn complete_utility_findings_exclude_css_comments_strings_and_urls() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/css-utility-literal-tokens");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(fixture.join("cases.json")).expect("whole token manifest"),
    )
    .expect("valid token manifest");
    assert_eq!(manifest["issue"], 7980);
    let cases = manifest["cases"]
        .as_array()
        .expect("all original and authored cases");
    assert_eq!(cases.len(), 10);
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["css/no-utility-classes".into()]))
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
                .expect("independently authored complete output vector"),
        )
        .expect("valid complete vector");
        let result = linter.lint_sfc(&source, filename);
        let api = complete(&result);
        let slices: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                source
                    .get(diagnostic.start as usize..diagnostic.end as usize)
                    .expect("every finding owns whole authored UTF-8")
            })
            .collect();
        let sources = vec![(filename.into(), source.clone().into())];
        let results = [result];
        let json: Value =
            serde_json::from_str(&format_results(&results, &sources, OutputFormat::Json))
                .expect("whole public JSON report");
        let plain = format_results(&results, &sources, OutputFormat::Plain);
        observations.push(json!({
            "case": case, "source": source, "expected": expected,
            "api": api, "json": json, "plain": plain, "authoredSlices": slices
        }));
        retain(&observations);
        assert_eq!(api, expected["api"], "{} whole API", case["id"]);
        assert_eq!(json, expected["json"], "{} whole JSON", case["id"]);
        assert_eq!(
            Some(plain.as_str()),
            expected["plain"].as_str(),
            "{}",
            case["id"]
        );
        assert_eq!(json!(slices), expected["authoredSlices"], "{}", case["id"]);
    }
    assert_eq!(observations.len(), 10);
}

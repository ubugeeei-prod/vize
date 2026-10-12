//! Source-only whole observations. This does not recapture any expected authority.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use vize_atelier_sfc::{CssCompileOptions, compile_css, parse_css_ast, print_css_ast};
use vize_glyph::{FormatError, FormatOptions, format_sfc, format_style};
use vize_l0::{String, cstr};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Locale};

const CORPUS: &str =
    include_str!("../../../tests/_fixtures/differential/css-engine-boundary-3295/cases.json");
const ATELIER_INPUTS: &str = include_str!(
    "../../../tests/_fixtures/differential/css-engine-vendor-exit/atelier-inputs.json"
);

fn digest(source: &str) -> String {
    let mut result = String::with_capacity(64);
    for byte in Sha256::digest(source.as_bytes()) {
        result.push_str(cstr!("{byte:02x}").as_str());
    }
    result
}

fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let labels: Vec<_> = diagnostic
                .labels
                .iter()
                .map(|label| json!({"message": label.message, "start": label.start, "end": label.end}))
                .collect();
            json!({"rule_name": diagnostic.rule_name, "severity": diagnostic.severity,
                "message": diagnostic.message, "start": diagnostic.start, "end": diagnostic.end,
                "help": diagnostic.help, "labels": labels, "fix": diagnostic.fix})
        })
        .collect();
    json!({"filename": result.filename, "error_count": result.error_count,
        "warning_count": result.warning_count, "diagnostics": diagnostics})
}

fn persist(destination: &Path, report: &Value) {
    fs::write(destination, serde_json::to_vec_pretty(report).unwrap())
        .expect("retain complete observed packet before subsequent source execution");
}

fn append(destination: &Path, report: &mut Value, group: &str, row: Value) {
    report[group].as_array_mut().unwrap().push(row);
    persist(destination, report);
}

fn main() {
    assert_eq!(
        digest(CORPUS),
        "32b8bb5632bc0755d1540ff828f3e70eb5e41ec45cb0c81de4df67a787f572c1"
    );
    assert_eq!(
        digest(ATELIER_INPUTS),
        "27a7d73a093c02ec80e81cfc1c8d8918e1bacbe4b2b2350e8c641352bff61786"
    );
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let atelier: Value = serde_json::from_str(ATELIER_INPUTS).unwrap();
    assert_eq!(atelier["observations"].as_array().unwrap().len(), 66);
    assert_eq!(
        atelier["originalLawSha256"],
        "7eefb93c2985526b20fe38ce76ffe09594064a17f08d85e9fcb6b54b6a197df7"
    );
    let destination = PathBuf::from(env::var_os("CSS_ENGINE_OBSERVATIONS").unwrap());
    let revision = env::var("CSS_ENGINE_SOURCE_REVISION").unwrap();
    assert_eq!(revision.len(), 40);
    assert!(revision.bytes().all(|byte| byte.is_ascii_hexdigit()));
    let mut report = json!({
        "schema": "vize.css-engine-vendor-observations", "sourceRevision": revision,
        "debugAssertions": cfg!(debug_assertions), "panicUnwind": cfg!(panic = "unwind"),
        "corpusSha256": digest(CORPUS), "atelierInputsSha256": digest(ATELIER_INPUTS),
        "expectationsRecaptured": false, "publicationQualification": false,
        "rustConsumers": [], "atelier": [], "additionalAtelier": [], "completed": false
    });
    persist(&destination, &report);
    for case in corpus["formatterCases"].as_array().unwrap() {
        let options: FormatOptions = serde_json::from_value(case["options"].clone()).unwrap();
        let mut current = String::from(case["source"].as_str().unwrap());
        for pass in 1..=3 {
            let result = format_sfc(&current, &options).expect("original SFC observer execution");
            append(
                &destination,
                &mut report,
                "rustConsumers",
                json!({
                    "surface": "formatter", "case": case["id"], "pass": pass,
                    "source": current, "options": case["options"], "beforeExpected": case["expected"],
                    "actual": {"code": result.code, "changed": result.changed}
                }),
            );
            current = result.code;
        }
    }
    for case in corpus["styleCases"].as_array().unwrap() {
        let mut current = String::from(case["source"].as_str().unwrap());
        for pass in 1..=3 {
            let before = current.clone();
            let actual = match format_style(&current, &FormatOptions::default()) {
                Ok(code) => {
                    current = code.clone();
                    json!({"code": code})
                }
                Err(error) => {
                    let display = cstr!("{error}");
                    match error {
                        FormatError::StyleFormatError(message) => json!({
                            "variant": "StyleFormatError", "message": message, "display": display
                        }),
                        _ => panic!("unexpected public style error: {display}"),
                    }
                }
            };
            append(
                &destination,
                &mut report,
                "rustConsumers",
                json!({
                    "surface": "style", "case": case["id"], "pass": pass,
                    "source": before, "beforeCase": case, "actual": actual
                }),
            );
        }
    }
    for case in corpus["linterCases"].as_array().unwrap() {
        assert_eq!(case["options"]["preset"], "incremental");
        assert_eq!(case["options"]["locale"], "en");
        assert_eq!(case["options"]["helpLevel"], "full");
        let enabled: Vec<String> = case["options"]["enabledRules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|rule| String::from(rule.as_str().unwrap()))
            .collect();
        let linter = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(enabled))
            .with_locale(Locale::En)
            .with_help_level(HelpLevel::Full);
        let source = case["source"].as_str().unwrap();
        let filename = case["options"]["filename"].as_str().unwrap();
        for pass in 1..=3 {
            let result = linter.lint_sfc(source, filename);
            let mut row = json!({"surface": "linter", "case": case["id"], "pass": pass,
                "source": source, "options": case["options"],
                "beforeExpected": case["expected"], "actual": complete(&result)});
            if let Some(fix) = result
                .diagnostics
                .first()
                .and_then(|finding| finding.fix.as_ref())
            {
                let fixed = fix.apply(source);
                row["fixedActual"] = json!({"source": fixed,
                    "result": complete(&linter.lint_sfc(&fixed, filename))});
            }
            append(&destination, &mut report, "rustConsumers", row);
        }
    }
    let options = CssCompileOptions::default();
    for (input_group, output_group) in [
        ("observations", "atelier"),
        ("additionalInputs", "additionalAtelier"),
    ] {
        for input in atelier[input_group].as_array().unwrap() {
            let source = input["source"].as_str().unwrap();
            assert_eq!(
                digest(source).as_str(),
                input["sourceSha256"].as_str().unwrap()
            );
            let parsed = parse_css_ast(source, &options);
            let printed = parsed.ast.clone().map(|ast| print_css_ast(ast, &options));
            append(
                &destination,
                &mut report,
                output_group,
                json!({
                    "input": input, "parsed": parsed, "compiled": compile_css(source, &options),
                    "printed": printed,
                    "subsequentHealthy": parse_css_ast("@keyframes x { 50% { opacity: 0.5 } }", &options)
                }),
            );
        }
    }
    assert_eq!(report["rustConsumers"].as_array().unwrap().len(), 99);
    assert_eq!(report["atelier"].as_array().unwrap().len(), 66);
    assert_eq!(report["additionalAtelier"].as_array().unwrap().len(), 6);
    report["completed"] = json!(true);
    persist(&destination, &report);
    println!("Observed whole99 Rust consumer and66 Atelier inputs; expectations remain frozen");
}

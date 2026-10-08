//! Temporary, retained-source research driver. This is the Rust Linter API,
//! not the native CLI, installed product, main qualification or a benchmark.
//! Usage: n8n_bound_slot_observer INPUT.json OUTPUT.json
//! INPUT must be a JSON array of {id, filename, source}; no input is inferred.
//! Output retains every native field and every selected or foreign diagnostic.
//! Partial packets are persisted before and after each call; process failures
//! stay visible through the raw exit status/stdout/stderr, never become clean.

use serde_json::{Value, json};
use std::{error::Error, fs, path::Path};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Severity};

const SOURCE_REVISION: &str = "26e56ac6a0de3f9f838db55bbdb71757317f2435";
const CONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/lint/bound-slot-attribute/config.json"
);

fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> =
        result
            .diagnostics
            .iter()
            .map(|d| {
                let labels: Vec<_> = d.labels.iter().map(|label| json!({
            "message": label.message.as_str(), "start": label.start, "end": label.end,
        })).collect();
                json!({
                    "rule_name": d.rule_name, "severity": d.severity, "message": d.message.as_str(),
                    "start": d.start, "end": d.end, "help": d.help.as_ref().map(|h| h.as_str()),
                    "labels": labels, "fix": d.fix,
                })
            })
            .collect();
    json!({
        "filename": result.filename.as_str(), "error_count": result.error_count,
        "warning_count": result.warning_count, "diagnostics": diagnostics,
    })
}

fn persist(output: &Path, evidence: &Value) -> Result<(), Box<dyn Error>> {
    let mut bytes = serde_json::to_vec_pretty(evidence)?;
    bytes.push(b'\n');
    fs::write(output, bytes)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 {
        return Err("usage: n8n_bound_slot_observer INPUT.json OUTPUT.json".into());
    }
    let input = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    if output.exists() {
        return Err("output already exists; use a new path to preserve evidence".into());
    }
    let inputs: Value = serde_json::from_slice(&fs::read(input)?)?;
    let cases = inputs.as_array().ok_or("input must be an array")?;
    let mut ids = std::collections::BTreeSet::new();
    for case in cases {
        let fields = case.as_object().ok_or("each case must be an object")?;
        if fields.len() != 3
            || !["id", "filename", "source"]
                .iter()
                .all(|key| fields.contains_key(*key))
        {
            return Err("each case must contain exactly id, filename and source".into());
        }
        for key in ["id", "filename", "source"] {
            case[key].as_str().ok_or("case fields must be strings")?;
        }
        if !ids.insert(case["id"].as_str().ok_or("id must be a string")?) {
            return Err("duplicate case id".into());
        }
    }
    let config: Value = serde_json::from_str(CONFIG)?;
    let selected = config["linter"]["rules"]
        .as_object()
        .ok_or("invalid frozen config")?;
    if selected.len() != 51 || !selected.values().all(|value| value == "error") {
        return Err("frozen config must select exactly51 explicit error rules".into());
    }
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(
            selected.keys().map(|name| name.as_str().into()).collect(),
        ))
        .with_rule_severity_overrides(
            selected
                .keys()
                .map(|name| (name.as_str().into(), Severity::Error))
                .collect(),
        )
        .with_help_level(HelpLevel::Full)
        .with_attribute_hyphenation(vize_patina::rules::HyphenationStyle::Always)
        .with_component_name_in_template_casing(vize_patina::rules::ComponentCasing::PascalCase)
        .with_sfc_element_order_options(vize_patina::rules::SfcElementOrderOptions {
            order: ["script", "template", "style"]
                .into_iter()
                .map(|name| vize_patina::rules::SfcElementOrderGroup::new(vec![name.into()]))
                .collect(),
        });
    let mut evidence = json!({
        "schema": "vize.n8n.authored-source-api-research.v1",
        "sourceRevision": SOURCE_REVISION, "configuration": config,
        "inputPath": input.to_string_lossy(), "repeatCount": 2,
        "completedCases": 0, "executionStatus": "running", "cases": [],
    });
    persist(output, &evidence)?;
    for case in cases {
        let mut record = case.clone();
        record["observations"] = json!([]);
        evidence["cases"]
            .as_array_mut()
            .ok_or("invalid output cases")?
            .push(record);
        let index = evidence["cases"]
            .as_array()
            .ok_or("invalid output cases")?
            .len()
            - 1;
        persist(output, &evidence)?;
        for _ in 0..2 {
            let result = linter.lint_sfc(
                case["source"].as_str().ok_or("source must be a string")?,
                case["filename"]
                    .as_str()
                    .ok_or("filename must be a string")?,
            );
            evidence["cases"][index]["observations"]
                .as_array_mut()
                .ok_or("invalid observations")?
                .push(complete(&result));
            persist(output, &evidence)?;
        }
        evidence["completedCases"] = json!(index + 1);
        persist(output, &evidence)?;
    }
    evidence["executionStatus"] = json!("completed");
    persist(output, &evidence)?;
    println!(
        "source_api_research source={SOURCE_REVISION} cases={} repeats=2",
        cases.len()
    );
    Ok(())
}

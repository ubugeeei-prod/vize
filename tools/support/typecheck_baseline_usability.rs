use crate::{common, diagnostic_json, project_deadline, project_string, record, sha256};
use regex::Regex;
use serde_json::{Value, json};
use std::path::Path;

pub fn configuration_snapshot(generated: &Path, authored: &Path) -> Result<Value, String> {
    let config = common::read_text(generated)?;
    let source_config = common::read_text(authored)?;
    Ok(json!({
        "configSha256": sha256(&config),
        "sourceConfigSha256": sha256(&source_config),
        "config": config,
        "sourceConfig": source_config,
    }))
}

pub fn retain_failure(
    report_dir: &Path,
    project: &Value,
    evidence: &Value,
    observations: &Path,
    kind: &str,
    reason: &str,
    diagnostics: &[Value],
) -> Result<(), String> {
    let project_id = project_string(project, "id")?;
    project_deadline::atomic_json(
        &report_dir.join(format!("{project_id}-baseline-unusable.json")),
        &json!({
            "schema": "vize.fixtureTypecheckBaselineUnusable",
            "version": 1,
            "project": project_id,
            "revision": project["revision"],
            "evidence": evidence,
            "cause": { "kind": kind, "reason": reason },
            "diagnostics": diagnostics,
            "observations": {
                "file": observations.file_name().and_then(|name| name.to_str()),
                "sha256": sha256(&common::read_text(observations)?),
            },
            "verdict": "unusable",
        }),
    )
}

/// The official CLI reports one-based start positions, but no end spans.
/// Keep all files here: declaration syntax can prevent semantic checking of Vue.
pub fn parse_diagnostics(output: &str) -> Result<Vec<Value>, String> {
    let positioned = Regex::new(r"^(.+)\((\d+),(\d+)\): (error|warning) TS(\d+): (.+)$").unwrap();
    let project = Regex::new(r"^(error|warning) TS(\d+): (.+)$").unwrap();
    let mut diagnostics: Vec<Value> = Vec::new();
    for raw_line in output.replace("\r\n", "\n").split('\n') {
        let line = raw_line.trim_end();
        if line.is_empty() {
            continue;
        }
        let diagnostic = if let Some(captures) = positioned.captures(line) {
            diagnostic_json(&record(
                captures[1].replace('\\', "/"),
                captures[4].to_string(),
                &captures[2],
                &captures[3],
                &captures[5],
                &captures[6],
            )?)
        } else if let Some(captures) = project.captures(line) {
            let mut diagnostic = diagnostic_json(&record(
                "<project>".to_string(),
                captures[1].to_string(),
                "1",
                "1",
                &captures[2],
                &captures[3],
            )?);
            for field in ["file", "line", "column"] {
                diagnostic[field] = Value::Null;
            }
            diagnostic
        } else if line.starts_with(char::is_whitespace) && !diagnostics.is_empty() {
            diagnostics.last_mut().unwrap()["continuation"]
                .as_array_mut()
                .unwrap()
                .push(json!(line));
            continue;
        } else {
            return Err(format!("unparseable vue-tsc diagnostic output: {line}"));
        };
        let mut diagnostic = diagnostic;
        diagnostic["coordinateBase"] = json!(1);
        diagnostic["end"] = Value::Null;
        diagnostic["continuation"] = json!([]);
        diagnostics.push(diagnostic);
    }
    Ok(diagnostics)
}

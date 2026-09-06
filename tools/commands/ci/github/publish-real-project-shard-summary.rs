#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```

use serde_json::Value;
use std::{
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

fn main() -> ExitCode {
    let publish_result = publish();
    let cleanup_result = dehydrate_selected_fixture_shard();
    match (publish_result, cleanup_result) {
        (Ok(()), Ok(())) => ExitCode::SUCCESS,
        (Err(error), Ok(())) | (Ok(()), Err(error)) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
        (Err(publish_error), Err(cleanup_error)) => {
            eprintln!("{publish_error}");
            eprintln!("{cleanup_error}");
            ExitCode::from(1)
        }
    }
}

fn publish() -> Result<(), String> {
    if let Some(extra) = env::args_os().nth(1) {
        return Err(format!(
            "Usage: rust-script tools/commands/ci/github/publish-real-project-shard-summary.rs; unexpected argument {}",
            extra.to_string_lossy()
        ));
    }
    let report_dir = PathBuf::from(
        env::var("FIXTURE_REPORT_DIR").map_err(|_| "FIXTURE_REPORT_DIR is required".to_string())?,
    );
    let summary_path = env::var("GITHUB_STEP_SUMMARY")
        .map_err(|_| "GITHUB_STEP_SUMMARY is required".to_string())?;
    let mut summary = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&summary_path)
        .map_err(|error| format!("failed to open {summary_path}: {error}"))?;

    append_file_or_line(
        &mut summary,
        &report_dir.join("summary.md"),
        "No fixture tool report was produced.",
    )?;
    append_json_line_or(
        &mut summary,
        &report_dir.join("lsp-lifecycle-summary.json"),
        "No LSP lifecycle report was produced.",
        lsp_summary,
        true,
    )?;
    append_json_line_or(
        &mut summary,
        &report_dir.join("syntax-highlighter-summary.json"),
        "No syntax-highlighter report was produced.",
        syntax_summary,
        false,
    )?;
    append_lint_summary(&mut summary, &report_dir)?;
    append_nonempty_file_or_line(
        &mut summary,
        &report_dir.join("syntax-highlighter-divergence.md"),
        "No syntax-highlighter divergence report was produced.",
    )?;
    append_unique_typecheck_divergence(&mut summary, &report_dir)?;
    append_json_line_or(
        &mut summary,
        &report_dir.join("glyph-waiver-issues.json"),
        "No formatter waiver owner report was produced.",
        waiver_summary,
        true,
    )?;
    append_json_line_or(
        &mut summary,
        &report_dir.join("surface-verdict.json"),
        "No real-project surface verdict was produced.",
        surface_summary,
        true,
    )
}

fn dehydrate_selected_fixture_shard() -> Result<(), String> {
    let Some(report_dir) = env::var_os("FIXTURE_REPORT_DIR") else {
        return Ok(());
    };
    let selected_path = PathBuf::from(report_dir).join("selected-fixtures.txt");
    if !is_nonempty(&selected_path) {
        return Ok(());
    }
    let selected = fs::read_to_string(&selected_path)
        .map_err(|error| format!("failed to read {}: {error}", selected_path.display()))?;
    let fixture_paths = selected
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    if fixture_paths.is_empty() {
        return Ok(());
    }
    let status = Command::new("git")
        .args(["submodule", "deinit", "--force", "--"])
        .args(fixture_paths)
        .stdin(Stdio::null())
        .status()
        .map_err(|error| format!("failed to run git submodule deinit: {error}"))?;
    if !status.success() {
        return Err(format!(
            "git submodule deinit failed with exit code {}",
            status.code().unwrap_or(1)
        ));
    }
    Ok(())
}

fn append_file_or_line(out: &mut fs::File, path: &Path, fallback: &str) -> Result<(), String> {
    if is_file(path) {
        let text = fs::read_to_string(path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        out.write_all(text.as_bytes())
            .map_err(|error| format!("failed to write summary: {error}"))?;
    } else {
        append_line(out, fallback)?;
    }
    Ok(())
}

fn append_nonempty_file_or_line(
    out: &mut fs::File,
    path: &Path,
    fallback: &str,
) -> Result<(), String> {
    if is_nonempty(path) {
        let text = fs::read_to_string(path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        out.write_all(text.as_bytes())
            .map_err(|error| format!("failed to write summary: {error}"))?;
    } else {
        append_line(out, fallback)?;
    }
    Ok(())
}

fn append_json_line_or(
    out: &mut fs::File,
    path: &Path,
    fallback: &str,
    render: fn(&Value) -> Result<String, String>,
    require_nonempty: bool,
) -> Result<(), String> {
    let usable = if require_nonempty {
        is_nonempty(path)
    } else {
        is_file(path)
    };
    if !usable {
        return append_line(out, fallback);
    }
    let json = read_json(path)?;
    append_line(out, &render(&json)?)
}

fn append_lint_summary(out: &mut fs::File, report_dir: &Path) -> Result<(), String> {
    let path = report_dir.join("lint-divergence-summary.json");
    if !is_nonempty(&path) {
        return append_line(out, "No lint divergence report was produced.");
    }
    let json = read_json(&path)?;
    append_line(out, &lint_summary(&json)?)?;
    for report in sorted_matches(report_dir, "-lint-divergence.md")? {
        if is_nonempty(&report) {
            let text = fs::read_to_string(&report)
                .map_err(|error| format!("failed to read {}: {error}", report.display()))?;
            out.write_all(text.as_bytes())
                .map_err(|error| format!("failed to write summary: {error}"))?;
        }
    }
    Ok(())
}

fn append_unique_typecheck_divergence(out: &mut fs::File, report_dir: &Path) -> Result<(), String> {
    let reports = sorted_matches(report_dir, "-typecheck-divergence.md")?
        .into_iter()
        .collect::<Vec<_>>();
    if reports.len() == 1 {
        let text = fs::read_to_string(&reports[0])
            .map_err(|error| format!("failed to read {}: {error}", reports[0].display()))?;
        out.write_all(text.as_bytes())
            .map_err(|error| format!("failed to write summary: {error}"))?;
    } else {
        append_line(out, "No unique typecheck divergence report was produced.")?;
    }
    Ok(())
}

fn sorted_matches(dir: &Path, suffix: &str) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    if !dir.exists() {
        return Ok(paths);
    }
    for entry in
        fs::read_dir(dir).map_err(|error| format!("failed to read {}: {error}", dir.display()))?
    {
        let path = entry
            .map_err(|error| format!("failed to read {}: {error}", dir.display()))?
            .path();
        if path.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(suffix))
        {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn append_line(out: &mut fs::File, line: &str) -> Result<(), String> {
    writeln!(out, "{line}").map_err(|error| format!("failed to write summary: {error}"))
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn is_file(path: &Path) -> bool {
    path.is_file()
}

fn is_nonempty(path: &Path) -> bool {
    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.len() > 0)
}

fn at<'a>(json: &'a Value, pointer: &str) -> Result<&'a Value, String> {
    json.pointer(pointer)
        .ok_or_else(|| format!("missing JSON field {pointer}"))
}

fn number(json: &Value, pointer: &str) -> Result<i64, String> {
    at(json, pointer)?
        .as_i64()
        .ok_or_else(|| format!("JSON field {pointer} must be a number"))
}

fn string_array(json: &Value, pointer: &str) -> Result<String, String> {
    let array = at(json, pointer)?
        .as_array()
        .ok_or_else(|| format!("JSON field {pointer} must be an array"))?;
    Ok(array
        .iter()
        .map(|value| value.as_str().unwrap_or_default())
        .collect::<Vec<_>>()
        .join(", "))
}

fn lsp_summary(json: &Value) -> Result<String, String> {
    Ok(format!(
        "LSP lifecycle: {} project(s), {} actual file(s), {} authored feature oracle(s), {} authored anchor(s), {} Vue file(s), {} failed project(s); missing authored oracles: {}",
        number(json, "/summary/projectCount")?,
        number(json, "/summary/actualFileCount")?,
        number(json, "/summary/authoredFeatureProjectCount")?,
        number(json, "/summary/authoredAnchorCount")?,
        number(json, "/summary/vueFileCount")?,
        number(json, "/summary/failedProjectCount")?,
        string_array(json, "/summary/missingAuthoredFeatureProjectIds")?
    ))
}

fn syntax_summary(json: &Value) -> Result<String, String> {
    Ok(format!(
        "Syntax highlighter: {} project(s), {} file(s), {} line(s), {} failed project(s)",
        number(json, "/summary/projectCount")?,
        number(json, "/summary/fileCount")?,
        number(json, "/summary/lineCount")?,
        number(json, "/summary/failedProjectCount")?
    ))
}

fn lint_summary(json: &Value) -> Result<String, String> {
    Ok(format!(
        "Lint divergence: {} project(s), {} shared, {} false positive(s), {} false negative(s), {} patina-only finding(s)",
        number(json, "/projectCount")?,
        number(json, "/totals/sharedCount")?,
        number(json, "/totals/falsePositiveCount")?,
        number(json, "/totals/falseNegativeCount")?,
        number(json, "/totals/patinaOnlyRuleFindingCount")?
    ))
}

fn waiver_summary(json: &Value) -> Result<String, String> {
    Ok(format!(
        "Formatter waivers: {} precise waiver(s), {} open owner Issue(s)",
        number(json, "/waiverCount")?,
        at(json, "/issues")?
            .as_array()
            .ok_or_else(|| "JSON field /issues must be an array".to_string())?
            .len()
    ))
}

fn surface_summary(json: &Value) -> Result<String, String> {
    Ok(format!(
        "Surface verdict: {}; failed: {}",
        at(json, "/status")?.as_str().unwrap_or_default(),
        string_array(json, "/failedSurfaceNames")?
    ))
}

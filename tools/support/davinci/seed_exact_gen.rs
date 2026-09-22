//! Snippet defect classes for exact rules, asserted by diagnostic identity.
//!
//! `[[span]]` marks each expected finding. One file per class, so a new
//! exact/sound snippet is a row in [`SNIPPETS`]. HTML nesting classes are
//! registered here too; their recall stays on `--html-nesting`.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    process::Command,
};

use serde::Serialize;

use crate::common;
use crate::davinci_fpfn::{
    DiagnosticRow, flatten_lint_json, index_to_line_col, line_starts_of, resolve_vize_cli,
};
use crate::seed_exact::{self, Contract};
use crate::seed_exact_snippets::{SNIPPETS, Snippet};
use crate::seed_html::{NESTING, NESTING_RULE};

pub struct ClassBinding {
    pub id: &'static str,
    pub rule: &'static str,
}

pub fn bindings() -> Result<Vec<ClassBinding>, String> {
    let mut bindings = Vec::new();
    for (_, classes) in NESTING {
        for id in *classes {
            bindings.push(ClassBinding {
                id,
                rule: NESTING_RULE,
            });
        }
    }
    for snippet in SNIPPETS {
        unmark(snippet)?;
        bindings.push(ClassBinding {
            id: snippet.id,
            rule: snippet.rule,
        });
    }
    Ok(bindings)
}

pub fn run(
    repo_root: &Path,
    out: &Path,
    assert_recall: bool,
    seeded_lint_json: Option<&Path>,
    report_path: Option<&Path>,
) -> Result<u8, String> {
    let contracts = seed_exact::load_contracts(&repo_root.join(seed_exact::TABLE_REL))?;
    let mut seeded = Vec::new();
    for snippet in SNIPPETS {
        let contract = contract_for(&contracts, snippet.rule)?;
        let (text, spans) = unmark(snippet)?;
        let starts = line_starts_of(&text);
        let expected = spans
            .into_iter()
            .map(|(start, end)| {
                let (line, column) = index_to_line_col(&text, &starts, start);
                let (end_line, end_column) = index_to_line_col(&text, &starts, end);
                DiagnosticRow {
                    path: format!("{}.vue", snippet.id),
                    rule_id: snippet.rule.to_string(),
                    severity: contract.severity.lint_code(),
                    line,
                    column,
                    end_line,
                    end_column,
                }
            })
            .collect::<Vec<_>>();
        seeded.push(Seeded {
            class: snippet.id,
            rule: snippet.rule,
            text,
            expected,
        });
    }
    seeded.sort_by(|left, right| left.class.cmp(right.class));
    for file in &seeded {
        common::write_text(out.join(&file.expected[0].path), &file.text)?;
    }
    let manifest = Manifest {
        schema_version: 1,
        tool: "tools/commands/davinci/seed-defects.rs --exact-classes",
        classes: seeded
            .iter()
            .map(|file| ManifestClass {
                class: file.class,
                rule: file.rule,
                path: file.expected[0].path.clone(),
                expected: file.expected.clone(),
            })
            .collect(),
    };
    common::write_json_pretty(out.join("manifest.json"), &manifest)?;
    let injections = seeded.iter().map(|file| file.expected.len()).sum::<usize>();
    println!(
        "scope-proof: files={} injections={injections}",
        seeded.len()
    );
    if !assert_recall {
        return Ok(0);
    }
    let rules: BTreeSet<&str> = SNIPPETS.iter().map(|snippet| snippet.rule).collect();
    let found = if let Some(path) = seeded_lint_json {
        flatten_lint_json(&common::read_json(path)?)?
    } else {
        lint(repo_root, out)?
    };
    let found = found
        .into_iter()
        .filter(|row| rules.contains(row.rule_id.as_str()))
        .map(|row| DiagnosticRow {
            path: normalize_path(&row.path, out),
            ..row
        })
        .collect::<Vec<_>>();
    let expected = seeded
        .iter()
        .flat_map(|file| file.expected.iter().cloned())
        .collect::<Vec<_>>();
    let (misses, unexpected) = difference(&expected, &found);
    let mut detected_by = BTreeMap::<&str, usize>::new();
    for file in &seeded {
        let hit = file
            .expected
            .iter()
            .filter(|row| !misses.contains(row))
            .count();
        detected_by.insert(file.class, hit);
    }
    let classes = seeded
        .iter()
        .map(|file| ClassRecall {
            class: file.class,
            rule: file.rule,
            expected: file.expected.len(),
            detected: detected_by[file.class],
        })
        .collect::<Vec<_>>();
    let verdict = if misses.is_empty() && unexpected.is_empty() {
        "pass"
    } else {
        "fail"
    };
    let report = Report {
        classes,
        misses,
        unexpected,
        verdict,
    };
    common::write_json_pretty(out.join("exact-report.json"), &report)?;
    if let Some(path) = report_path {
        common::write_json_pretty(path, &report)?;
    }
    for class in &report.classes {
        println!(
            "class {} detected={}/{}",
            class.class, class.detected, class.expected
        );
    }
    for miss in &report.misses {
        println!(
            "MISS {}:{}:{}-{}:{} {}",
            miss.path, miss.line, miss.column, miss.end_line, miss.end_column, miss.rule_id
        );
    }
    for extra in &report.unexpected {
        println!(
            "UNEXPECTED {}:{}:{}-{}:{} {}",
            extra.path, extra.line, extra.column, extra.end_line, extra.end_column, extra.rule_id
        );
    }
    let detected = report
        .classes
        .iter()
        .map(|class| class.detected)
        .sum::<usize>();
    println!(
        "assert: detected={detected}/{} unexpected={} verdict={}",
        injections,
        report.unexpected.len(),
        report.verdict
    );
    Ok(if report.verdict == "pass" { 0 } else { 1 })
}

struct Seeded {
    class: &'static str,
    rule: &'static str,
    text: String,
    expected: Vec<DiagnosticRow>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    schema_version: u32,
    tool: &'static str,
    classes: Vec<ManifestClass>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestClass {
    class: &'static str,
    rule: &'static str,
    path: String,
    expected: Vec<DiagnosticRow>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassRecall {
    class: &'static str,
    rule: &'static str,
    expected: usize,
    detected: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Report {
    classes: Vec<ClassRecall>,
    misses: Vec<DiagnosticRow>,
    unexpected: Vec<DiagnosticRow>,
    verdict: &'static str,
}

fn contract_for<'a>(rows: &'a [Contract], rule: &str) -> Result<&'a Contract, String> {
    let row = rows
        .iter()
        .find(|row| row.name == rule)
        .ok_or_else(|| format!("snippet rule \"{rule}\" has no contract row"))?;
    if !matches!(row.tier, seed_exact::Tier::Exact | seed_exact::Tier::Sound) {
        return Err(format!("snippet rule \"{rule}\" is not exact or sound"));
    }
    Ok(row)
}

fn unmark(snippet: &Snippet) -> Result<(String, Vec<(usize, usize)>), String> {
    let mut clean = String::with_capacity(snippet.source.len());
    let mut spans = Vec::new();
    let mut rest = snippet.source;
    while let Some(start) = rest.find("[[") {
        clean.push_str(&rest[..start]);
        rest = &rest[start + 2..];
        let Some(end) = rest.find("]]") else {
            return Err(format!("class \"{}\" has an unclosed [[ span", snippet.id));
        };
        if rest[..end].contains("[[") {
            return Err(format!("class \"{}\" has nested [[ spans", snippet.id));
        }
        let span_start = clean.len();
        clean.push_str(&rest[..end]);
        if clean.len() == span_start {
            return Err(format!("class \"{}\" has an empty span", snippet.id));
        }
        spans.push((span_start, clean.len()));
        rest = &rest[end + 2..];
    }
    if rest.contains("]]") {
        return Err(format!("class \"{}\" has a stray ]]", snippet.id));
    }
    clean.push_str(rest);
    if spans.is_empty() {
        return Err(format!("class \"{}\" has no span", snippet.id));
    }
    Ok((clean, spans))
}

fn lint(repo_root: &Path, cwd: &Path) -> Result<Vec<DiagnosticRow>, String> {
    let cli = resolve_vize_cli(repo_root);
    let output = Command::new(&cli.command)
        .args(&cli.prefix)
        .args(["lint", "--no-config", "--format", "json", "**/*.vue"])
        .current_dir(cwd)
        .output()
        .map_err(|error| format!("failed to run vize lint: {error}"))?;
    if !matches!(output.status.code(), Some(0 | 1)) {
        return Err(format!(
            "vize lint failed in {}:\n{}",
            cwd.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let json = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("vize lint emitted non-JSON output: {error}"))?;
    flatten_lint_json(&json)
}

fn normalize_path(path: &str, root: &Path) -> String {
    let path = path.replace('\\', "/");
    let trimmed = path.strip_prefix("./").unwrap_or(&path);
    let candidate = PathBuf::from(trimmed);
    candidate
        .strip_prefix(root)
        .ok()
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|| trimmed.to_string())
}

fn difference(
    expected: &[DiagnosticRow],
    found: &[DiagnosticRow],
) -> (Vec<DiagnosticRow>, Vec<DiagnosticRow>) {
    let mut remaining = found.to_vec();
    let mut misses = Vec::new();
    for row in expected {
        match remaining.iter().position(|candidate| candidate == row) {
            Some(index) => {
                remaining.swap_remove(index);
            }
            None => misses.push(row.clone()),
        }
    }
    misses.sort();
    remaining.sort();
    (misses, remaining)
}

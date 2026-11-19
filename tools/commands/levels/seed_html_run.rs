//! `seed-defects.rs --html-nesting`: seed the TS-37 HTML nesting classes
//! (`seed_html.rs`), lint the original and seeded trees with
//! `vize lint --cross-file`, and assert recall by identity.
//!
//! The assertion is exact over the two HTML rules: the seeded tree's
//! diagnostics must equal the original tree's, moved through the insertions,
//! plus every expected injection — no miss, nothing unexpected.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use serde::Serialize;

use crate::common;
use crate::davinci_fpfn::{
    DiagnosticRow, ResolvedSources, VizeCli, flatten_lint_json, index_to_line_col,
    line_col_to_index, line_starts_of, list_vue_files, resolve_vize_cli,
};
use crate::seed_html::{
    BLOCK_FILE, BLOCK_SOURCE, COMPOSED_RULE, Expected, HtmlSeed, NESTING_RULE, plan, shift,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Injection {
    class: &'static str,
    path: String,
    expected: DiagnosticRow,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Scope {
    files_scanned: usize,
    eligible: usize,
    composed_eligible: usize,
    injections: usize,
    skipped: BTreeMap<&'static str, usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClassRecall {
    class: &'static str,
    expected: usize,
    detected: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Report {
    scope: Scope,
    classes: Vec<ClassRecall>,
    baseline_mapped: usize,
    misses: Vec<DiagnosticRow>,
    unexpected: Vec<DiagnosticRow>,
    verdict: &'static str,
}

fn row(path: &str, rule: &str, text: &str, start: usize, end: usize) -> DiagnosticRow {
    let starts = line_starts_of(text);
    let (line, column) = index_to_line_col(text, &starts, start);
    let (end_line, end_column) = index_to_line_col(text, &starts, end);
    DiagnosticRow {
        path: path.to_string(),
        rule_id: rule.to_string(),
        severity: 2,
        line,
        column,
        end_line,
        end_column,
    }
}

fn lint(cli: &VizeCli, cwd: &Path) -> Result<Vec<DiagnosticRow>, String> {
    let output = std::process::Command::new(&cli.command)
        .args(&cli.prefix)
        .args([
            "lint",
            "--no-config",
            "--cross-file",
            "--format",
            "json",
            "**/*.vue",
        ])
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
    Ok(flatten_lint_json(&json)?
        .into_iter()
        .filter(|row| row.rule_id == NESTING_RULE || row.rule_id == COMPOSED_RULE)
        .collect())
}

/// Seed every source root into `out/original` and `out/seeded`; with
/// `assert`, lint both and compare by identity. Returns the exit code.
pub fn run(
    repo_root: &Path,
    source: &ResolvedSources,
    out: &Path,
    assert: bool,
) -> Result<u8, String> {
    let mut files: Vec<(String, String)> = Vec::new();
    for root in &source.roots {
        for rel in list_vue_files(&root.root)? {
            files.push((
                format!("{}{rel}", root.prefix),
                common::read_text(root.root.join(&rel))?,
            ));
        }
    }
    let mut scope = Scope {
        files_scanned: files.len(),
        ..Scope::default()
    };
    let mut seeds: BTreeMap<String, HtmlSeed> = BTreeMap::new();
    let mut injections = Vec::new();
    let mut block_dirs = BTreeSet::new();
    for (path, text) in &files {
        // Any mention of the file stem counts: barrel re-exports
        // (`import { Root } from '..'`) resolve to the file without naming it.
        let stem = Path::new(path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default();
        let imported = files
            .iter()
            .any(|(other, source)| other != path && source.contains(stem));
        match plan(text, imported) {
            Ok(seed) => {
                scope.eligible += 1;
                if seed.composed {
                    scope.composed_eligible += 1;
                    block_dirs.insert(
                        Path::new(path)
                            .parent()
                            .map(Path::to_path_buf)
                            .unwrap_or_default(),
                    );
                }
                for Expected {
                    class,
                    rule,
                    start,
                    end,
                } in &seed.expected
                {
                    let expected = row(path, rule, &seed.seeded, *start, *end);
                    injections.push(Injection {
                        class,
                        path: path.clone(),
                        expected,
                    });
                }
                seeds.insert(path.clone(), seed);
            }
            Err(reason) => *scope.skipped.entry(reason).or_default() += 1,
        }
    }
    scope.injections = injections.len();
    for (path, text) in &files {
        common::write_text(out.join("original").join(path), text)?;
        let seeded = seeds
            .get(path)
            .map_or(text.as_str(), |seed| seed.seeded.as_str());
        common::write_text(out.join("seeded").join(path), seeded)?;
    }
    for dir in &block_dirs {
        common::write_text(out.join("seeded").join(dir).join(BLOCK_FILE), BLOCK_SOURCE)?;
    }
    common::write_json_pretty(out.join("html-manifest.json"), &injections)?;
    println!(
        "scope-proof: files-scanned={} html-eligible={} composed-eligible={} injections={}",
        scope.files_scanned, scope.eligible, scope.composed_eligible, scope.injections
    );
    if !assert {
        return Ok(0);
    }

    let cli = resolve_vize_cli(repo_root);
    let original = lint(&cli, &out.join("original"))?;
    let mut expected: Vec<DiagnosticRow> = Vec::new();
    for found in &original {
        let Some(seed) = seeds.get(&found.path) else {
            expected.push(found.clone());
            continue;
        };
        let text = &files
            .iter()
            .find(|(path, _)| *path == found.path)
            .ok_or("file")?
            .1;
        let starts = line_starts_of(text);
        let at = |line, column| line_col_to_index(text, &starts, line, column).ok_or("span");
        let start = shift(at(found.line, found.column)?, &seed.inserts, false);
        let end = shift(at(found.end_line, found.end_column)?, &seed.inserts, true);
        expected.push(DiagnosticRow {
            severity: found.severity,
            ..row(&found.path, &found.rule_id, &seed.seeded, start, end)
        });
    }
    let baseline_mapped = expected.len();
    expected.extend(
        injections
            .iter()
            .map(|injection| injection.expected.clone()),
    );
    let seeded = lint(&cli, &out.join("seeded"))?;
    let (misses, unexpected) = difference(&expected, &seeded);
    let mut classes: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();
    for injection in &injections {
        let entry = classes.entry(injection.class).or_default();
        entry.0 += 1;
        entry.1 += usize::from(!misses.contains(&injection.expected));
    }
    let verdict = if misses.is_empty() && unexpected.is_empty() {
        "pass"
    } else {
        "fail"
    };
    let report = Report {
        scope,
        classes: classes
            .into_iter()
            .map(|(class, (expected, detected))| ClassRecall {
                class,
                expected,
                detected,
            })
            .collect(),
        baseline_mapped,
        misses,
        unexpected,
        verdict,
    };
    common::write_json_pretty(out.join("html-report.json"), &report)?;
    for class in &report.classes {
        println!(
            "class {} detected={}/{}",
            class.class, class.detected, class.expected
        );
    }
    for miss in &report.misses {
        println!(
            "MISS {}:{}:{} {}",
            miss.path, miss.line, miss.column, miss.rule_id
        );
    }
    for extra in &report.unexpected {
        println!(
            "UNEXPECTED {}:{}:{} {}",
            extra.path, extra.line, extra.column, extra.rule_id
        );
    }
    let detected: usize = report.classes.iter().map(|class| class.detected).sum();
    println!(
        "assert: detected={detected}/{} baseline-mapped={} unexpected={} verdict={}",
        report.scope.injections,
        report.baseline_mapped,
        report.unexpected.len(),
        report.verdict
    );
    Ok(if report.verdict == "pass" { 0 } else { 1 })
}

/// Multiset difference both ways: (expected − found, found − expected).
fn difference(
    expected: &[DiagnosticRow],
    found: &[DiagnosticRow],
) -> (Vec<DiagnosticRow>, Vec<DiagnosticRow>) {
    let mut remaining: Vec<DiagnosticRow> = found.to_vec();
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

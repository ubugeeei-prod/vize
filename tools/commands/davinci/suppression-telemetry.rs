#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    path::{Path, PathBuf},
    process::ExitCode,
};

#[path = "../../support/common.rs"]
mod common;
#[path = "../../support/davinci/fpfn.rs"]
mod davinci_fpfn;

use davinci_fpfn::{
    RuleMapReport, SourceInfo, SuppressionReport, SuppressionScan, SuppressionScope,
    UnmappedSuppression, defuse_suppressions, flatten_lint_json, intersect_suppressions,
    list_vue_files, load_rule_map, resolve_corpus_sources, resolve_fixture_sources,
    resolve_vize_cli, run_vize_lint_json, scan_suppressions,
};

const USAGE: &str = "Usage: rust-script tools/commands/davinci/suppression-telemetry.rs (--fixtures <dir> | --corpus-shard) --out <dir> [--report <path>]\n\nCollects eslint-disable pragmas, maps rule names to vize analogs, and\nreports vize diagnostics on suppressed lines as FP candidates.";

#[derive(Debug)]
struct Args {
    fixtures: Option<PathBuf>,
    corpus_shard: bool,
    out: Option<PathBuf>,
    report: Option<PathBuf>,
    help: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err((code, message)) => {
            eprintln!("{message}");
            ExitCode::from(code)
        }
    }
}

fn run() -> Result<(), (u8, String)> {
    let repo_root = common::repo_root().map_err(|error| (2, error))?;
    let args = parse_args(env::args().skip(1).collect())?;
    if args.help {
        println!("{USAGE}");
        return Ok(());
    }
    let out = args
        .out
        .clone()
        .ok_or_else(|| (2, format!("--out <dir> is required\n\n{USAGE}")))?;
    let out_dir = absolute(&out);
    let source = resolve_sources(&repo_root, &args).map_err(|error| (2, error))?;

    let mut files = Vec::new();
    let mut suppressions_by_path = BTreeMap::<String, SuppressionScan>::new();
    let mut suppression_comments = 0usize;
    let mut named_suppressions = 0usize;
    let mut bare_suppressions = 0usize;
    let mut name_occurrences = BTreeMap::<String, usize>::new();
    for root in &source.roots {
        for rel_path in list_vue_files(&root.root).map_err(|error| (2, error))? {
            let file_path = format!("{}{}", root.prefix, rel_path);
            files.push(file_path.clone());
            let text = common::read_text(root.root.join(&rel_path)).map_err(|error| (2, error))?;
            let scanned = scan_suppressions(&text);
            suppression_comments += scanned.comments.len();
            for comment in &scanned.comments {
                if comment.kind == "enable" {
                    continue;
                }
                if comment.rules.is_empty() {
                    bare_suppressions += 1;
                } else {
                    named_suppressions += comment.rules.len();
                    for rule in &comment.rules {
                        *name_occurrences.entry(rule.clone()).or_insert(0) += 1;
                    }
                }
            }
            if !scanned.ranges.is_empty() {
                suppressions_by_path.insert(file_path.clone(), scanned);
            }
            let (defused, _) = defuse_suppressions(&text);
            common::write_text(out_dir.join("defused").join(&file_path), &defused)
                .map_err(|error| (2, error))?;
        }
    }

    let cli = resolve_vize_cli(&repo_root);
    let lint_json =
        run_vize_lint_json(&cli, &out_dir.join("defused"), &files).map_err(|error| (2, error))?;
    let lint_rows = flatten_lint_json(&lint_json).map_err(|error| (2, error))?;
    let mut diagnostics_by_path = BTreeMap::<String, Vec<davinci_fpfn::DiagnosticRow>>::new();
    for row in &lint_rows {
        diagnostics_by_path
            .entry(row.path.clone())
            .or_default()
            .push(row.clone());
    }

    let rule_map = load_rule_map(&repo_root).map_err(|error| (2, error))?;
    let (candidates, on_bare_lines) =
        intersect_suppressions(&diagnostics_by_path, &suppressions_by_path, &rule_map);
    let unmapped = name_occurrences
        .iter()
        .filter(|(rule, _)| !rule_map.mapped.contains_key(*rule))
        .map(|(rule, occurrences)| UnmappedSuppression {
            rule: rule.clone(),
            occurrences: *occurrences,
        })
        .collect::<Vec<_>>();
    let mapped_seen = name_occurrences
        .keys()
        .filter(|rule| rule_map.mapped.contains_key(*rule))
        .collect::<BTreeSet<_>>()
        .len();
    let report = SuppressionReport {
        schema_version: 1,
        tool: "tools/commands/davinci/suppression-telemetry.rs".to_string(),
        source: SourceInfo {
            kind: source.kind,
            label: source.label,
        },
        rule_map: RuleMapReport {
            fixture: rule_map.fixture_path,
            mapped_rules: rule_map.fixture_mapped_count,
            core_sidecar_rules: rule_map.core_sidecar_count,
        },
        scope: SuppressionScope {
            files_scanned: files.len(),
            suppression_comments,
            named_suppressions,
            bare_suppressions,
            rule_names_seen: name_occurrences.len(),
            mapped_names_seen: mapped_seen,
            unmapped_names_seen: unmapped.len(),
            defused_run_diagnostics: lint_rows.len(),
            diagnostics_on_bare_suppressed_lines: on_bare_lines,
        },
        unmapped,
        candidates,
    };
    if let Some(path) = args.report {
        common::write_json_pretty(absolute(&path), &report).map_err(|error| (2, error))?;
    }

    println!(
        "suppression-telemetry: source={} candidates={}",
        report.source.label,
        report.candidates.len()
    );
    println!(
        "scope-proof: files-scanned={} suppression-comments={} named={} bare={} rules-mapped={} mapped-seen={} unmapped-seen={} fp-candidates={}",
        report.scope.files_scanned,
        report.scope.suppression_comments,
        report.scope.named_suppressions,
        report.scope.bare_suppressions,
        report.rule_map.mapped_rules + report.rule_map.core_sidecar_rules,
        report.scope.mapped_names_seen,
        report.scope.unmapped_names_seen,
        report.candidates.len()
    );
    for entry in &report.unmapped {
        println!("unmapped: {} x{}", entry.rule, entry.occurrences);
    }
    for candidate in &report.candidates {
        println!(
            "fp-candidate: {}:{}:{} {} (suppressed as {} at line {})",
            candidate.path,
            candidate.line,
            candidate.column,
            candidate.vize_rule,
            candidate.eslint_rule,
            candidate.comment_line
        );
    }
    Ok(())
}

fn parse_args(argv: Vec<String>) -> Result<Args, (u8, String)> {
    let mut args = Args {
        fixtures: None,
        corpus_shard: false,
        out: None,
        report: None,
        help: false,
    };
    let mut index = 0;
    while index < argv.len() {
        match argv[index].as_str() {
            "--fixtures" => {
                index += 1;
                args.fixtures = Some(PathBuf::from(value(&argv, index, "--fixtures")?));
            }
            "--corpus-shard" => args.corpus_shard = true,
            "--out" => {
                index += 1;
                args.out = Some(PathBuf::from(value(&argv, index, "--out")?));
            }
            "--report" => {
                index += 1;
                args.report = Some(PathBuf::from(value(&argv, index, "--report")?));
            }
            "--help" | "-h" => args.help = true,
            other => return Err((2, format!("unknown argument {other}\n\n{USAGE}"))),
        }
        index += 1;
    }
    Ok(args)
}

fn value(argv: &[String], index: usize, name: &str) -> Result<String, (u8, String)> {
    argv.get(index)
        .cloned()
        .ok_or_else(|| (2, format!("{name} requires a value")))
}

fn resolve_sources(repo_root: &Path, args: &Args) -> Result<davinci_fpfn::ResolvedSources, String> {
    let picked = usize::from(args.fixtures.is_some()) + usize::from(args.corpus_shard);
    if picked != 1 {
        return Err(format!(
            "exactly one of --fixtures/--corpus-shard is required\n\n{USAGE}"
        ));
    }
    if let Some(fixtures) = &args.fixtures {
        return resolve_fixture_sources(repo_root, &absolute(fixtures));
    }
    resolve_corpus_sources(repo_root)
}

fn absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

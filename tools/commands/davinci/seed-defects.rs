#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use std::{env, path::PathBuf, process::ExitCode};

#[path = "../../support/common.rs"]
mod common;
#[path = "../../support/davinci/fpfn.rs"]
mod davinci_fpfn;
#[path = "../../support/davinci/seed_exact.rs"]
mod seed_exact;
#[path = "../../support/davinci/seed_exact_gen.rs"]
mod seed_exact_gen;
#[path = "../../support/davinci/seed_exact_snippets.rs"]
mod seed_exact_snippets;
#[path = "../../support/davinci/seed_html.rs"]
mod seed_html;
#[path = "../../support/davinci/seed_html_run.rs"]
mod seed_html_run;
#[path = "../../support/davinci/seed_pilot.rs"]
mod seed_pilot;
#[path = "../../support/davinci/seed_print.rs"]
mod seed_print;

use davinci_fpfn::{assert_seeded_tree, resolve_vize_cli};

pub const USAGE: &str = "\
Usage: rust-script tools/commands/davinci/seed-defects.rs [--html-nesting | --exact-classes] (--fixtures <dir> | --matrix | --corpus-shard) --out <dir> [--assert] [--report <path>]
       rust-script tools/commands/davinci/seed-defects.rs --check-classes [--contracts <table.rs>] [--ledger <ledger-fn.md>]

Seeds defect classes and (with --assert) verifies recall by diagnostic
identity. --html-nesting seeds the P4-11 HTML nesting classes.
--exact-classes seeds one file per exact/sound snippet class.
--check-classes fails unless every exact/sound rule in rule_contracts.rs
has a class. A rule listed in ledger-fn.md is triaged, not waived.
";

#[derive(Debug)]
struct Args {
    fixtures: Option<PathBuf>,
    matrix: bool,
    corpus_shard: bool,
    html_nesting: bool,
    exact_classes: bool,
    check_classes: bool,
    contracts: Option<PathBuf>,
    ledger: Option<PathBuf>,
    out: Option<PathBuf>,
    assert: bool,
    report: Option<PathBuf>,
    baseline_lint_json: Option<PathBuf>,
    seeded_lint_json: Option<PathBuf>,
    help: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err((code, message)) => {
            eprintln!("{message}");
            ExitCode::from(code)
        }
    }
}

fn run() -> Result<u8, (u8, String)> {
    let repo_root = common::repo_root().map_err(|error| (2, error))?;
    let args = parse_args(env::args().skip(1).collect())?;
    if args.help {
        println!("{USAGE}");
        return Ok(0);
    }
    if args.check_classes {
        if args.exact_classes
            || args.html_nesting
            || args.fixtures.is_some()
            || args.matrix
            || args.corpus_shard
            || args.assert
            || args.out.is_some()
        {
            return Err((
                2,
                format!("--check-classes does not take a source mode\n\n{USAGE}"),
            ));
        }
        return seed_exact::check(
            &repo_root,
            args.contracts.as_deref(),
            args.ledger.as_deref(),
        )
        .map_err(|error| (2, error));
    }
    if args.contracts.is_some() || args.ledger.is_some() {
        return Err((
            2,
            format!("--contracts and --ledger require --check-classes\n\n{USAGE}"),
        ));
    }
    let out = args
        .out
        .clone()
        .ok_or_else(|| (2, format!("--out <dir> is required\n\n{USAGE}")))?;
    let out_dir = seed_pilot::absolute(&out);
    common::mkdir(&out_dir).map_err(|error| (2, error))?;
    if args.html_nesting {
        if args.exact_classes {
            return Err((
                2,
                format!("--html-nesting and --exact-classes conflict\n\n{USAGE}"),
            ));
        }
        let source = seed_pilot::resolve_sources(
            &repo_root,
            args.fixtures.as_deref(),
            args.matrix,
            args.corpus_shard,
            &out_dir,
        )
        .map_err(|error| (2, error))?;
        return seed_html_run::run(&repo_root, &source, &out_dir, args.assert)
            .map_err(|error| (2, error));
    }
    if args.exact_classes {
        if args.fixtures.is_some()
            || args.matrix
            || args.corpus_shard
            || args.baseline_lint_json.is_some()
        {
            return Err((
                2,
                format!(
                    "--exact-classes seeds its own files and takes only --seeded-lint-json\n\n{USAGE}"
                ),
            ));
        }
        return seed_exact_gen::run(
            &repo_root,
            &out_dir,
            args.assert,
            args.seeded_lint_json.as_deref(),
            args.report.as_deref(),
        )
        .map_err(|error| (2, error));
    }

    let has_source = args.fixtures.is_some() || args.matrix || args.corpus_shard;
    let manifest = if has_source {
        seed_pilot::seed(
            &repo_root,
            args.fixtures.as_deref(),
            args.matrix,
            args.corpus_shard,
            &out_dir,
        )
        .map_err(|error| (2, error))?
    } else if args.assert && out_dir.join("manifest.json").exists() {
        serde_json::from_value(
            common::read_json(out_dir.join("manifest.json")).map_err(|error| (2, error))?,
        )
        .map_err(|error| (2, format!("cannot parse manifest: {error}")))?
    } else {
        return Err((
            2,
            format!(
                "nothing to do: pass a source mode, or --assert with an existing {}",
                out_dir.join("manifest.json").display()
            ),
        ));
    };

    if !args.assert {
        return Ok(0);
    }
    let hooks_present = args.baseline_lint_json.is_some() && args.seeded_lint_json.is_some();
    let cli = (!hooks_present).then(|| resolve_vize_cli(&repo_root));
    let report = assert_seeded_tree(
        &manifest,
        &out_dir,
        cli.as_ref(),
        args.baseline_lint_json.as_deref(),
        args.seeded_lint_json.as_deref(),
    )
    .map_err(|error| (2, error))?;
    if let Some(path) = args.report {
        common::write_json_pretty(seed_pilot::absolute(&path), &report)
            .map_err(|error| (2, error))?;
    }
    seed_print::print_assert_report(&report);
    Ok(if report.verdict == "pass" { 0 } else { 1 })
}

fn parse_args(argv: Vec<String>) -> Result<Args, (u8, String)> {
    let mut args = Args {
        fixtures: None,
        matrix: false,
        corpus_shard: false,
        html_nesting: false,
        exact_classes: false,
        check_classes: false,
        contracts: None,
        ledger: None,
        out: None,
        assert: false,
        report: None,
        baseline_lint_json: None,
        seeded_lint_json: None,
        help: false,
    };
    let mut index = 0;
    while index < argv.len() {
        match argv[index].as_str() {
            "--fixtures" => {
                index += 1;
                args.fixtures = Some(PathBuf::from(value(&argv, index, "--fixtures")?));
            }
            "--matrix" => args.matrix = true,
            "--corpus-shard" => args.corpus_shard = true,
            "--html-nesting" => args.html_nesting = true,
            "--exact-classes" => args.exact_classes = true,
            "--check-classes" => args.check_classes = true,
            "--contracts" => {
                index += 1;
                args.contracts = Some(PathBuf::from(value(&argv, index, "--contracts")?));
            }
            "--ledger" => {
                index += 1;
                args.ledger = Some(PathBuf::from(value(&argv, index, "--ledger")?));
            }
            "--out" => {
                index += 1;
                args.out = Some(PathBuf::from(value(&argv, index, "--out")?));
            }
            "--assert" => args.assert = true,
            "--report" => {
                index += 1;
                args.report = Some(PathBuf::from(value(&argv, index, "--report")?));
            }
            "--baseline-lint-json" => {
                index += 1;
                args.baseline_lint_json =
                    Some(PathBuf::from(value(&argv, index, "--baseline-lint-json")?));
            }
            "--seeded-lint-json" => {
                index += 1;
                args.seeded_lint_json =
                    Some(PathBuf::from(value(&argv, index, "--seeded-lint-json")?));
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

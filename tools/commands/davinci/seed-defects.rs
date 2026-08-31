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
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
    process::ExitCode,
};

#[path = "../../rust/common.rs"]
mod common;
#[path = "../../rust/davinci_fpfn.rs"]
mod davinci_fpfn;

use davinci_fpfn::{
    CLASS_A, CLASS_A_RULE, CLASS_B, EditRecord, Identifier, Injection, SeedFile, SeedManifest,
    SeedScope, SourceInfo, UNUSED_BINDING_NAME, apply_seed, assert_seeded_tree,
    describe_seeded_span, list_vue_files, plan_class_a, plan_class_b, resolve_corpus_sources,
    resolve_fixture_sources, resolve_vize_cli,
};

const USAGE: &str = "Usage: rust-script tools/commands/davinci/seed-defects.rs (--fixtures <dir> | --matrix | --corpus-shard) --out <dir> [--assert] [--report <path>]\n\nSeeds the P0-13 defect classes into copies of .vue sources and (with\n--assert) verifies recall by diagnostic identity against the manifest.";

#[derive(Debug)]
struct Args {
    fixtures: Option<PathBuf>,
    matrix: bool,
    corpus_shard: bool,
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
    let out = args
        .out
        .clone()
        .ok_or_else(|| (2, format!("--out <dir> is required\n\n{USAGE}")))?;
    let out_dir = absolute(&out);
    common::mkdir(&out_dir).map_err(|error| (2, error))?;

    let has_source = args.fixtures.is_some() || args.matrix || args.corpus_shard;
    let manifest = if has_source {
        seed(&repo_root, &args, &out_dir).map_err(|error| (2, error))?
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
        common::write_json_pretty(absolute(&path), &report).map_err(|error| (2, error))?;
    }
    print_assert_report(&report);
    Ok(if report.verdict == "pass" { 0 } else { 1 })
}

fn parse_args(argv: Vec<String>) -> Result<Args, (u8, String)> {
    let mut args = Args {
        fixtures: None,
        matrix: false,
        corpus_shard: false,
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

fn seed(repo_root: &Path, args: &Args, out_dir: &Path) -> Result<SeedManifest, String> {
    let source = resolve_sources(repo_root, args, out_dir)?;
    let mut files = Vec::new();
    let mut injections = Vec::new();
    let mut edits = BTreeMap::<String, Vec<EditRecord>>::new();
    let mut class_a_eligible = 0usize;
    for root in &source.roots {
        for rel_path in list_vue_files(&root.root)? {
            let seed_path = format!("{}{}", root.prefix, rel_path);
            let original = common::read_text(root.root.join(&rel_path))?;
            let (class_a, class_a_reason) = plan_class_a(&original);
            let (class_b, _) = plan_class_b(&original);
            let applied = apply_seed(&original, class_a.as_ref(), class_b.as_ref());
            files.push(SeedFile {
                path: seed_path.clone(),
                class_a: class_a.is_some(),
                class_b: class_b.is_some(),
                class_a_reason: class_a_reason.clone(),
            });
            if let Some(plan) = &class_a {
                class_a_eligible += 1;
                let ref_start = map_template_ref(
                    &applied.seeded,
                    plan.template_ref[0],
                    &plan.name,
                    &applied.edits,
                )?;
                let starts = davinci_fpfn::line_starts_of(&applied.seeded);
                injections.push(Injection {
                    class_name: CLASS_A.to_string(),
                    path: seed_path.clone(),
                    expected_rule: Some(CLASS_A_RULE.to_string()),
                    identifier: Identifier {
                        original: Some(plan.name.clone()),
                        seeded: plan.seeded_name.clone(),
                    },
                    script_rename_count: Some(plan.rename_spans.len()),
                    created_script_setup_block: None,
                    expected: describe_seeded_span(
                        &applied.seeded,
                        &starts,
                        ref_start,
                        ref_start + plan.name.len(),
                    ),
                    note: None,
                });
            }
            if let Some(plan) = &class_b {
                let id_start = applied.seeded.find(UNUSED_BINDING_NAME).ok_or_else(|| {
                    "seed-defects internal error: unused binding not found".to_string()
                })?;
                let starts = davinci_fpfn::line_starts_of(&applied.seeded);
                injections.push(Injection {
                    class_name: CLASS_B.to_string(),
                    path: seed_path.clone(),
                    expected_rule: None,
                    identifier: Identifier {
                        original: None,
                        seeded: UNUSED_BINDING_NAME.to_string(),
                    },
                    script_rename_count: None,
                    created_script_setup_block: Some(plan.created_block),
                    expected: describe_seeded_span(
                        &applied.seeded,
                        &starts,
                        id_start,
                        id_start + UNUSED_BINDING_NAME.len(),
                    ),
                    note: Some(
                        "vize_croquis unused_bindings has no lint consumer (documented FN, ledger-fn.md)"
                            .to_string(),
                    ),
                });
            }
            if !applied.edits.is_empty() {
                edits.insert(seed_path.clone(), applied.edits);
            }
            common::write_text(out_dir.join("original").join(&seed_path), &original)?;
            common::write_text(out_dir.join("seeded").join(&seed_path), &applied.seeded)?;
        }
    }
    injections.sort_by(|a, b| a.path.cmp(&b.path).then(a.class_name.cmp(&b.class_name)));
    let manifest = SeedManifest {
        schema_version: 1,
        tool: "tools/commands/davinci/seed-defects.rs".to_string(),
        source: SourceInfo {
            kind: source.kind,
            label: source.label,
        },
        scope: SeedScope {
            files_copied: files.len(),
            class_a_eligible,
            class_a_injections: injections
                .iter()
                .filter(|injection| injection.class_name == CLASS_A)
                .count(),
            class_b_injections: injections
                .iter()
                .filter(|injection| injection.class_name == CLASS_B)
                .count(),
        },
        files,
        injections,
        edits,
    };
    common::write_json_pretty(out_dir.join("manifest.json"), &manifest)?;
    println!(
        "seed-defects: source={} -> {}",
        manifest.source.label,
        common::relative_path(
            &env::current_dir().map_err(|error| error.to_string())?,
            out_dir
        )
    );
    println!(
        "scope-proof: files-scanned={} class-a-eligible={} class-a-injections={} class-b-injections={}",
        manifest.scope.files_copied,
        manifest.scope.class_a_eligible,
        manifest.scope.class_a_injections,
        manifest.scope.class_b_injections
    );
    Ok(manifest)
}

fn resolve_sources(
    repo_root: &Path,
    args: &Args,
    out_dir: &Path,
) -> Result<davinci_fpfn::ResolvedSources, String> {
    let picked = usize::from(args.fixtures.is_some())
        + usize::from(args.matrix)
        + usize::from(args.corpus_shard);
    if picked != 1 {
        return Err(format!(
            "exactly one of --fixtures/--matrix/--corpus-shard is required\n\n{USAGE}"
        ));
    }
    if let Some(fixtures) = &args.fixtures {
        return resolve_fixture_sources(repo_root, &absolute(fixtures));
    }
    if args.matrix {
        let matrix_dir = out_dir.join("matrix-src");
        common::run_capture_in(
            "rust-script",
            &[
                "tools/commands/davinci/matrix-gen.rs",
                "--write",
                "--out-dir",
                matrix_dir.to_string_lossy().as_ref(),
            ],
            repo_root,
        )?;
        return Ok(davinci_fpfn::ResolvedSources {
            kind: "matrix".to_string(),
            label: "matrix-gen".to_string(),
            roots: vec![davinci_fpfn::SourceRoot {
                root: matrix_dir,
                prefix: String::new(),
            }],
        });
    }
    resolve_corpus_sources(repo_root)
}

fn map_template_ref(
    seeded: &str,
    original_ref_start: usize,
    name: &str,
    edits: &[EditRecord],
) -> Result<usize, String> {
    let mut ref_start = original_ref_start as isize;
    for edit in edits {
        if edit.span[1] <= original_ref_start {
            ref_start += edit.delta;
        }
    }
    let ref_start = usize::try_from(ref_start).map_err(|_| "negative template ref".to_string())?;
    let found = seeded
        .get(ref_start..ref_start + name.len())
        .ok_or_else(|| {
            "seed-defects internal error: template ref relocation out of range".to_string()
        })?;
    if found != name {
        return Err(format!(
            "seed-defects internal error: template ref relocation failed ({found})"
        ));
    }
    Ok(ref_start)
}

fn print_assert_report(report: &davinci_fpfn::SeedAssertReport) {
    println!(
        "assert: class-a detected={}/{} class-b detected={}/{} baseline mapped={} verdict={}",
        report.class_a.detected,
        report.class_a.expected,
        report.class_b.detected,
        report.class_b.expected,
        report.baseline_shift.mapped,
        report.verdict
    );
    for miss in &report.class_a.misses {
        println!(
            "MISS class-a {}:{}:{}-{}:{} {} identifier={}",
            miss.path,
            miss.line,
            miss.column,
            miss.end_line,
            miss.end_column,
            miss.rule_id,
            miss.identifier
        );
    }
    for miss in &report.baseline_shift.misses {
        println!("MISS baseline {}", describe_row(miss));
    }
    for row in &report.baseline_shift.unmappable {
        println!("UNMAPPABLE baseline {}", describe_row(row));
    }
    for row in &report.unexpected {
        println!("UNEXPECTED {}", describe_row(row));
    }
}

fn describe_row(row: &davinci_fpfn::DiagnosticRow) -> String {
    format!(
        "{}:{}:{}-{}:{} {}",
        row.path, row.line, row.column, row.end_line, row.end_column, row.rule_id
    )
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

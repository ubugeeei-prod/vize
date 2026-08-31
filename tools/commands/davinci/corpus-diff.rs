#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! sha2 = "0.10"
//! ```

use serde_json::Value;
use std::{
    collections::BTreeSet,
    env,
    path::PathBuf,
    process::ExitCode,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

#[path = "../../rust/common.rs"]
mod common;
#[path = "../../rust/davinci_corpus.rs"]
mod davinci_corpus;

#[derive(Debug)]
struct Args {
    shards: usize,
    vize_bin: Option<PathBuf>,
    baseline: Option<PathBuf>,
    write_fresh: Option<PathBuf>,
    timeout_ms: Option<u64>,
    keep_raw: bool,
    surfaces: Vec<String>,
    clean_fixtures: bool,
    allow_dirty_fixtures: bool,
}

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let root = common::repo_root()?;
    let args = parse_args(env::args().skip(1).collect())?;
    let surfaces = davinci_corpus::surface_filter(&args.surfaces)?;
    let manifest = davinci_corpus::load_manifest(&root)?;
    let baseline_path = absolutize(
        &root,
        args.baseline
            .unwrap_or_else(|| root.join(davinci_corpus::BASELINE_REL)),
    );
    let baseline_label = common::relative_path(&root, &baseline_path);

    if args.clean_fixtures {
        let removed = davinci_corpus::clean_fixtures(&root)?;
        println!("corpus-diff: cleaned {removed} materialized node_modules from the fixtures");
    }
    davinci_corpus::assert_fixtures_pristine(&root, args.allow_dirty_fixtures)?;

    if !baseline_path.exists() {
        return Err(format!(
            "baseline artifact is missing: {baseline_label}\ngenerate it with: rust-script tools/commands/davinci/corpus-baseline.rs"
        ));
    }
    let baseline = common::read_json(&baseline_path)?;
    let all_surfaces = davinci_corpus::surface_filter(&Vec::new())?;
    let baseline_scope_failures =
        davinci_corpus::verify_scope(&baseline, &manifest, &all_surfaces, "committed baseline")?;
    if !baseline_scope_failures.is_empty() {
        return Err(format!(
            "scope proof failed for the committed baseline:\n{}",
            baseline_scope_failures.join("\n")
        ));
    }
    let vize_bin = davinci_corpus::resolve_vize_bin(&root, args.vize_bin)?;

    println!(
        "corpus-diff: {} projects, surfaces [{}], {} parallel shards",
        manifest["projects"].as_array().map_or(0, Vec::len),
        surfaces.join(", "),
        args.shards
    );
    let started = Instant::now();
    let scratch_dir = davinci_corpus::scratch_root(&root, &format!("diff-{}", process_slug()));
    let shard_dirs = match davinci_corpus::run_matrix(
        &root,
        args.shards,
        &vize_bin,
        &surfaces,
        &scratch_dir,
        args.timeout_ms,
    ) {
        Ok(shard_dirs) => shard_dirs,
        Err(error) => {
            println!(
                "raw shard reports kept for debugging: {}",
                common::relative_path(&root, &scratch_dir)
            );
            return Err(error);
        }
    };
    let fresh_rows = davinci_corpus::reduce_shards(&root, &shard_dirs, &surfaces)?;
    let fresh = davinci_corpus::build_artifact(&fresh_rows, &manifest)?;
    if let Some(write_fresh) = args.write_fresh {
        let fresh_path = absolutize(&root, write_fresh);
        common::write_json_pretty(&fresh_path, &fresh)?;
        println!(
            "wrote fresh artifact: {}",
            common::relative_path(&root, &fresh_path)
        );
    }
    if !args.keep_raw {
        davinci_corpus::cleanup_scratch(&scratch_dir)?;
    } else {
        println!(
            "raw shard reports kept: {}",
            common::relative_path(&root, &scratch_dir)
        );
    }

    let fresh_scope_failures =
        davinci_corpus::verify_scope(&fresh, &manifest, &surfaces, "fresh run")?;
    let baseline_rows = baseline
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| "baseline rows must be an array".to_string())?
        .iter()
        .filter(|row| {
            row.get("surface")
                .and_then(Value::as_str)
                .is_some_and(|surface| surfaces.iter().any(|item| item == surface))
        })
        .cloned()
        .collect::<Vec<_>>();
    let all_drift = davinci_corpus::diff_rows(&baseline_rows, &fresh_rows);
    let unstable_keys = davinci_corpus::load_unstable_rows(&root, &manifest)?
        .iter()
        .filter_map(|row| {
            Some(format!(
                "{} {}",
                row.get("surface")?.as_str()?,
                row.get("project")?.as_str()?
            ))
        })
        .collect::<BTreeSet<_>>();
    let mut drift = Vec::new();
    let mut unstable_drift = Vec::new();
    for record in all_drift {
        let key = format!(
            "{} {}",
            record.get("surface").and_then(Value::as_str).unwrap_or(""),
            record.get("project").and_then(Value::as_str).unwrap_or("")
        );
        if record.get("kind").and_then(Value::as_str) == Some("changed")
            && unstable_keys.contains(&key)
        {
            unstable_drift.push(record);
        } else {
            drift.push(record);
        }
    }
    let elapsed_seconds = started.elapsed().as_secs();

    for record in &unstable_drift {
        println!(
            "unstable (filed in {}, not gating): {}/{} {} -> {}",
            davinci_corpus::UNSTABLE_REL,
            record.get("surface").and_then(Value::as_str).unwrap_or("?"),
            record.get("project").and_then(Value::as_str).unwrap_or("?"),
            prefix12(
                record
                    .get("baseline_hash")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
            prefix12(
                record
                    .get("fresh_hash")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            )
        );
    }
    if !drift.is_empty() {
        println!("drift: {} row(s) differ from {baseline_label}", drift.len());
        for record in &drift {
            let kind = record.get("kind").and_then(Value::as_str).unwrap_or("?");
            let surface = record.get("surface").and_then(Value::as_str).unwrap_or("?");
            let project = record.get("project").and_then(Value::as_str).unwrap_or("?");
            if kind == "changed" {
                let baseline_file_count = record
                    .get("baseline_file_count")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                let fresh_file_count = record
                    .get("fresh_file_count")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                let file_note = if baseline_file_count == fresh_file_count {
                    format!("{fresh_file_count} files")
                } else {
                    format!("files {baseline_file_count} -> {fresh_file_count}")
                };
                println!(
                    "  changed  {surface}/{project} ({file_note}) {} -> {}",
                    prefix12(
                        record
                            .get("baseline_hash")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                    ),
                    prefix12(
                        record
                            .get("fresh_hash")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                    )
                );
            } else {
                println!("  {kind}  {surface}/{project}");
            }
        }
        let mut by_surface = std::collections::BTreeMap::<String, usize>::new();
        for record in &drift {
            let surface = record
                .get("surface")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_string();
            *by_surface.entry(surface).or_default() += 1;
        }
        println!(
            "drift by surface: {}",
            by_surface
                .into_iter()
                .map(|(surface, count)| format!("{surface}={count}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if !fresh_scope_failures.is_empty() {
        println!("scope proof failed for the fresh run:");
        for reason in &fresh_scope_failures {
            println!("  {reason}");
        }
    }
    if !drift.is_empty() || !fresh_scope_failures.is_empty() {
        println!("corpus-diff: FAIL in {elapsed_seconds}s");
        return Err("corpus-diff failed".to_string());
    }
    let expected_fresh_comparisons =
        davinci_corpus::expected_comparison_count(&manifest, &surfaces)?;
    let filter_note = if surfaces.len() == all_surfaces.len() {
        String::new()
    } else {
        format!(" [surface filter: {}]", surfaces.join(", "))
    };
    let unstable_note = if unstable_drift.is_empty() {
        String::new()
    } else {
        format!(
            " ({} filed unstable row(s) drifted without gating)",
            unstable_drift.len()
        )
    };
    println!(
        "corpus-diff: PASS in {elapsed_seconds}s - zero gating drift across {}/{} comparisons ({} projects x {} surfaces, {} files); scope proof matches {}-project manifest{filter_note}{unstable_note}",
        fresh_rows.len(),
        expected_fresh_comparisons,
        fresh["scope"]["projects_run"],
        fresh["scope"]["surfaces_per_project"],
        fresh["scope"]["total_file_count"],
        manifest["projects"].as_array().map_or(0, Vec::len)
    );
    Ok(())
}

fn parse_args(argv: Vec<String>) -> Result<Args, String> {
    let mut args = Args {
        shards: 4,
        vize_bin: None,
        baseline: None,
        write_fresh: None,
        timeout_ms: None,
        keep_raw: false,
        surfaces: Vec::new(),
        clean_fixtures: false,
        allow_dirty_fixtures: false,
    };
    let mut index = 0;
    while index < argv.len() {
        let arg = &argv[index];
        let value = |index: &mut usize| -> Result<String, String> {
            *index += 1;
            argv.get(*index)
                .cloned()
                .ok_or_else(|| format!("{arg} requires a value"))
        };
        match arg.as_str() {
            "--shards" => {
                args.shards = davinci_corpus::positive_integer(&value(&mut index)?, arg)? as usize
            }
            "--vize-bin" => args.vize_bin = Some(PathBuf::from(value(&mut index)?)),
            "--baseline" => args.baseline = Some(PathBuf::from(value(&mut index)?)),
            "--write-fresh" => args.write_fresh = Some(PathBuf::from(value(&mut index)?)),
            "--timeout-ms" => {
                args.timeout_ms = Some(davinci_corpus::positive_integer(&value(&mut index)?, arg)?)
            }
            "--keep-raw" => args.keep_raw = true,
            "--clean-fixtures" => args.clean_fixtures = true,
            "--allow-dirty-fixtures" => args.allow_dirty_fixtures = true,
            "--surface" => args
                .surfaces
                .extend(davinci_corpus::split_csv(&value(&mut index)?)),
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
        index += 1;
    }
    Ok(args)
}

fn print_help() {
    println!(
        "usage: rust-script tools/commands/davinci/corpus-diff.rs [--surface s[,s]] [--shards n] [--vize-bin path] [--baseline path] [--write-fresh path] [--timeout-ms n] [--keep-raw] [--clean-fixtures] [--allow-dirty-fixtures]"
    );
}

fn absolutize(root: &std::path::Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

fn prefix12(value: &str) -> &str {
    value.get(..12).unwrap_or(value)
}

fn process_slug() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("{}-{millis}", std::process::id())
}

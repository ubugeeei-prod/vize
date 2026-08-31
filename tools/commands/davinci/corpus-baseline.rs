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

use std::{
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
    out: Option<PathBuf>,
    timeout_ms: Option<u64>,
    keep_raw: bool,
    clean_fixtures: bool,
    allow_dirty_fixtures: bool,
}

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let root = common::repo_root()?;
    let args = parse_args(env::args().skip(1).collect())?;
    let manifest = davinci_corpus::load_manifest(&root)?;
    let vize_bin = davinci_corpus::resolve_vize_bin(&root, args.vize_bin)?;
    let out_path = args
        .out
        .unwrap_or_else(|| root.join(davinci_corpus::BASELINE_REL));
    let out_path = if out_path.is_absolute() {
        out_path
    } else {
        root.join(out_path)
    };
    let scratch_dir = davinci_corpus::scratch_root(&root, &format!("baseline-{}", process_slug()));

    if args.clean_fixtures {
        let removed = davinci_corpus::clean_fixtures(&root)?;
        println!("corpus-baseline: cleaned {removed} materialized node_modules from the fixtures");
    }
    davinci_corpus::assert_fixtures_pristine(&root, args.allow_dirty_fixtures)?;

    let surfaces = davinci_corpus::surface_filter(&Vec::new())?;
    println!(
        "corpus-baseline: {} projects x {} surfaces, {} parallel shards",
        manifest["projects"].as_array().map_or(0, Vec::len),
        surfaces.len(),
        args.shards
    );
    let started = Instant::now();
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
    let rows = davinci_corpus::reduce_shards(&root, &shard_dirs, &surfaces)?;
    let artifact = davinci_corpus::build_artifact(&rows, &manifest)?;
    let scope_failures =
        davinci_corpus::verify_scope(&artifact, &manifest, &surfaces, "generated artifact")?;
    if !scope_failures.is_empty() {
        println!(
            "raw shard reports kept for debugging: {}",
            common::relative_path(&root, &scratch_dir)
        );
        return Err(format!(
            "scope proof failed:\n  {}",
            scope_failures.join("\n  ")
        ));
    }
    common::write_json_pretty(&out_path, &artifact)?;
    if !args.keep_raw {
        davinci_corpus::cleanup_scratch(&scratch_dir)?;
    } else {
        println!(
            "raw shard reports kept: {}",
            common::relative_path(&root, &scratch_dir)
        );
    }

    let elapsed_seconds = started.elapsed().as_secs();
    let out_label = if out_path == root.join(davinci_corpus::BASELINE_REL) {
        davinci_corpus::BASELINE_REL.to_string()
    } else {
        common::relative_path(&root, &out_path)
    };
    let expected_comparisons = davinci_corpus::expected_comparison_count(&manifest, &surfaces)?;
    let scope = &artifact["scope"];
    println!(
        "wrote {out_label}: {}/{} comparisons ({} projects x {} surfaces, {} files) in {}s",
        scope["row_count"],
        expected_comparisons,
        scope["projects_run"],
        scope["surfaces_per_project"],
        scope["total_file_count"],
        elapsed_seconds
    );
    Ok(())
}

fn parse_args(argv: Vec<String>) -> Result<Args, String> {
    let mut args = Args {
        shards: 4,
        vize_bin: None,
        out: None,
        timeout_ms: None,
        keep_raw: false,
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
            "--out" => args.out = Some(PathBuf::from(value(&mut index)?)),
            "--timeout-ms" => {
                args.timeout_ms = Some(davinci_corpus::positive_integer(&value(&mut index)?, arg)?)
            }
            "--keep-raw" => args.keep_raw = true,
            "--clean-fixtures" => args.clean_fixtures = true,
            "--allow-dirty-fixtures" => args.allow_dirty_fixtures = true,
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
        "usage: rust-script tools/commands/davinci/corpus-baseline.rs [--shards n] [--vize-bin path] [--out path] [--timeout-ms n] [--keep-raw] [--clean-fixtures] [--allow-dirty-fixtures]"
    );
}

fn process_slug() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("{}-{millis}", std::process::id())
}

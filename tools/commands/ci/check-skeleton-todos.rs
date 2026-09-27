#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```
//!
//! Ratchet for skeleton `todo!()` bodies (the 2026-09-28 skeleton-first
//! decision, #6826). The workspace denies `clippy::todo`; a skeleton module
//! opts out with a module-level
//! `#![expect(clippy::todo, reason = "skeleton: #NNNN")]`. This check counts
//! those `expect(clippy::todo` occurrences (outside comments) per crate
//! under `crates/` and compares them with `tools/config/skeleton-todos.toml`:
//!
//! - a crate above its baseline (or missing from it) fails: counts only go down;
//! - a crate below its baseline fails too, so each PR that fills a skeleton
//!   lowers the committed number (`--write` does that for you);
//! - a baseline entry for a crate with no skeleton modules fails.
//!
//! `--write` only ever lowers or removes entries; raising one is a reviewed
//! manual edit of the baseline file.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

const BASELINE: &str = "tools/config/skeleton-todos.toml";
const MARKER: &str = "expect(clippy::todo";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check = args.iter().any(|arg| arg == "--check");
    let write = args.iter().any(|arg| arg == "--write");
    match run(check, write) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(check: bool, write: bool) -> Result<(), String> {
    let baseline_text = fs::read_to_string(BASELINE).map_err(|err| format!("{BASELINE}: {err}"))?;
    let baseline = parse_baseline(&baseline_text)?;
    let counts = count_crates(Path::new("crates"))?;

    let mut problems = Vec::new();
    let mut raised = false;
    for (krate, &count) in &counts {
        match baseline.get(krate) {
            None => {
                raised = true;
                problems.push(format!(
                    "{krate}: {count} skeleton todo module(s) but no entry in {BASELINE}"
                ));
            }
            Some(&limit) if count > limit => {
                raised = true;
                problems.push(format!(
                    "{krate}: {count} skeleton todo module(s), baseline {limit}; counts only go down"
                ));
            }
            Some(&limit) if count < limit => problems.push(format!(
                "{krate}: {count} skeleton todo module(s), baseline {limit}; lower the baseline (--write)"
            )),
            Some(_) => {}
        }
    }
    for krate in baseline.keys() {
        if !counts.contains_key(krate) {
            problems.push(format!(
                "{krate}: no skeleton todo modules left; remove its entry (--write)"
            ));
        }
    }

    let total: usize = counts.values().sum();
    println!(
        "skeleton todos: {total} module(s) in {} crate(s)",
        counts.len()
    );

    if write {
        if raised {
            return Err(problems.join("\n"));
        }
        fs::write(BASELINE, render_baseline(&baseline_text, &counts))
            .map_err(|err| format!("{BASELINE}: {err}"))?;
        println!("  wrote {BASELINE}");
        return Ok(());
    }
    if problems.is_empty() || !check {
        for problem in &problems {
            println!("  {problem}");
        }
        return Ok(());
    }
    Err(problems.join("\n"))
}

/// Reads `crate = count` lines; `#` comments and blank lines are ignored.
fn parse_baseline(text: &str) -> Result<BTreeMap<String, usize>, String> {
    let mut entries = BTreeMap::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        let parsed = line.split_once('=').and_then(|(key, value)| {
            let key = key.trim().trim_matches('"');
            let value = value.trim().parse::<usize>().ok()?;
            (!key.is_empty()).then(|| (key.to_owned(), value))
        });
        let Some((key, value)) = parsed else {
            return Err(format!("{BASELINE}:{}: expected `crate = count`", index + 1));
        };
        if entries.insert(key, value).is_some() {
            return Err(format!("{BASELINE}:{}: duplicate entry", index + 1));
        }
    }
    Ok(entries)
}

/// Keeps the header comment and rewrites the entries from `counts`.
fn render_baseline(previous: &str, counts: &BTreeMap<String, usize>) -> String {
    let mut out: String = previous
        .lines()
        .take_while(|line| line.trim().is_empty() || line.trim_start().starts_with('#'))
        .map(|line| format!("{line}\n"))
        .collect();
    for (krate, count) in counts {
        out.push_str(&format!("{krate} = {count}\n"));
    }
    out
}

/// Counts marker occurrences in every `.rs` file, grouped by `crates/<name>`.
fn count_crates(root: &Path) -> Result<BTreeMap<String, usize>, String> {
    let mut counts = BTreeMap::new();
    let entries = fs::read_dir(root).map_err(|err| format!("{}: {err}", root.display()))?;
    for entry in entries {
        let path = entry.map_err(|err| err.to_string())?.path();
        if !path.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        collect_rust_files(&path, &mut files)?;
        let mut count = 0usize;
        for file in files {
            let source =
                fs::read_to_string(&file).map_err(|err| format!("{}: {err}", file.display()))?;
            count += count_markers(&source);
        }
        if count > 0 {
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            counts.insert(name, count);
        }
    }
    Ok(counts)
}

/// Counts markers outside `//` comments, ignoring whitespace so a
/// rustfmt-wrapped `#[expect(\n    clippy::todo,` still counts.
fn count_markers(source: &str) -> usize {
    let code: String = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .flat_map(str::chars)
        .filter(|ch| !ch.is_whitespace())
        .collect();
    code.matches(MARKER).count()
}

fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|err| format!("{}: {err}", dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|err| err.to_string())?.path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if path.is_dir() {
            if name != "target" && name != "node_modules" {
                collect_rust_files(&path, files)?;
            }
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

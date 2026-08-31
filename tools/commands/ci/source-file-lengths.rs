#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{
    collections::HashMap,
    env, fs,
    process::{Command, ExitCode},
};

#[derive(Clone)]
struct SourceFile {
    path: String,
    lines: usize,
}

enum FailureReason {
    NewFileExceededLimit,
    CrossedLimit,
    OverLimitFileGrew,
}

struct CheckFailure {
    path: String,
    lines: usize,
    base_lines: Option<usize>,
    reason: FailureReason,
}

fn main() -> ExitCode {
    ExitCode::from(run() as u8)
}

fn run() -> i32 {
    let mut max_lines = 350;
    let mut limit = 50;
    let mut check = false;
    let mut base_ref = String::new();
    let args = env::args().skip(1).collect::<Vec<_>>();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--check" => check = true,
            "--max-lines" => match parse_value(&args, &mut i, "--max-lines") {
                Ok(value) => max_lines = value,
                Err(message) => return print_error(message),
            },
            "--limit" => match parse_value(&args, &mut i, "--limit") {
                Ok(value) => limit = value,
                Err(message) => return print_error(message),
            },
            "--base-ref" => match string_value(&args, &mut i, "--base-ref") {
                Ok(value) => base_ref = value,
                Err(message) => return print_error(message),
            },
            "--help" => {
                println!(
                    "Usage: rust-script tools/commands/ci/source-file-lengths.rs [--check --base-ref <ref>] [--max-lines 350] [--limit 50]"
                );
                return 0;
            }
            value => return print_error(format!("Unknown argument: {value}")),
        }
        i += 1;
    }

    let paths = match tracked_paths() {
        Ok(paths) => paths,
        Err(()) => return print_error("Failed to read tracked files with git ls-files"),
    };
    let files = collect_current_files(paths);
    print_inventory(&files, max_lines, limit);

    if !check {
        return 0;
    }
    if !git_ref_exists(&base_ref) {
        println!("\nNo comparable base ref found; inventory completed without enforcing growth.");
        return 0;
    }

    let base_paths = base_rename_paths(&base_ref);
    let failures = collect_failures(&files, max_lines, &base_ref, &base_paths);
    println!("\nCompared with {base_ref}.");
    if failures.is_empty() {
        println!("No new or grown files exceed {max_lines} lines.");
        return 0;
    }
    println!("Files requiring action:");
    print_failures(&failures);
    1
}

fn parse_value(args: &[String], i: &mut usize, name: &str) -> Result<i64, String> {
    let value = string_value(args, i, name)?;
    value
        .parse::<i64>()
        .map_err(|_| format!("Invalid {name} value: {value}"))
}

fn string_value(args: &[String], i: &mut usize, name: &str) -> Result<String, String> {
    let Some(value) = args.get(*i + 1) else {
        return Err(format!("Missing value for {name}"));
    };
    *i += 1;
    Ok(value.clone())
}

fn print_error(message: impl AsRef<str>) -> i32 {
    println!("{}", message.as_ref());
    1
}

fn line_count(content: &str) -> usize {
    content.lines().count()
}

fn is_source_path(path: &str) -> bool {
    [
        ".rs", ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".mts", ".cts", ".vue", ".css",
        ".scss", ".html", ".pkl", ".mbt", ".scm", ".lua", ".json", ".toml", ".yaml", ".yml", ".md",
    ]
    .iter()
    .any(|suffix| path.ends_with(suffix))
}

fn has_path_segment(path: &str, segment: &str) -> bool {
    path.starts_with(&format!("{segment}/")) || path.contains(&format!("/{segment}/"))
}

fn is_excluded_path(path: &str) -> bool {
    path.ends_with("Cargo.lock")
        || path.ends_with("pnpm-lock.yaml")
        || path.ends_with(".lock")
        || path.starts_with("tools/benchmarks/results/")
        || path.starts_with("tools/commands/")
        || path.starts_with("tools/support/")
        || path == "npm/cli/schemas/vize.config.schema.json"
        || path == "npm/cli/src/types/generated.ts"
        || path == "davinci-road/plan/croquis-consumption.md"
        || path == "davinci-road/plan/corpus-coverage.md"
        || path == "package.json"
        || path.ends_with("/package.json")
        || path == ".github/workflows/release.yml"
        || path.starts_with("docs/content/ja/")
        || path.starts_with("docs/content/zh-CN/")
        || path.starts_with("docs/content/pt-BR/")
        || path.starts_with("docs/content/fr/")
        || path.contains(".min.")
        || [
            "_fixtures",
            "__fixtures__",
            "fixtures",
            "__snapshots__",
            "snapshots",
            "dist",
        ]
        .iter()
        .any(|segment| has_path_segment(path, segment))
        || [
            "target",
            "node_modules",
            "vendor",
            "generated",
            "__generated__",
            "i18n",
        ]
        .iter()
        .any(|segment| has_path_segment(path, segment))
        || has_path_segment(path, "playwright-report")
        || has_path_segment(path, "coverage")
        || path == "npm/native/index.js"
        || path == "npm/native/index.d.ts"
        || path == "npm/fresco-native/index.js"
        || path == "npm/fresco-native/index.d.ts"
}

fn collect_current_files(paths: Vec<String>) -> Vec<SourceFile> {
    let mut files = paths
        .into_iter()
        .filter(|path| is_source_path(path) && !is_excluded_path(path))
        .filter(|path| is_regular_file(path))
        .filter_map(|path| {
            fs::read_to_string(&path).ok().map(|content| SourceFile {
                path,
                lines: line_count(&content),
            })
        })
        .collect::<Vec<_>>();
    files.sort_by(|a, b| b.lines.cmp(&a.lines).then_with(|| a.path.cmp(&b.path)));
    files
}

fn is_regular_file(path: &str) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_file())
        .unwrap_or(false)
}

fn tracked_paths() -> Result<Vec<String>, ()> {
    let output = Command::new("git")
        .arg("ls-files")
        .output()
        .map_err(|_| ())?;
    if !output.status.success() {
        return Err(());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect())
}

fn git_ref_exists(ref_name: &str) -> bool {
    !ref_name.is_empty()
        && Command::new("git")
            .args(["rev-parse", "--verify", &format!("{ref_name}^{{commit}}")])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
}

fn base_rename_paths(base_ref: &str) -> HashMap<String, String> {
    let output = match Command::new("git")
        .args(["diff", "--name-status", "--find-renames", base_ref, "--"])
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return HashMap::new(),
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let parts = line.split('\t').collect::<Vec<_>>();
            (parts.len() >= 3 && parts[0].starts_with('R'))
                .then(|| (parts[2].to_string(), parts[1].to_string()))
        })
        .collect()
}

fn base_file_lines(
    base_ref: &str,
    path: &str,
    base_paths: &HashMap<String, String>,
) -> Option<usize> {
    let base_path = base_paths.get(path).map_or(path, String::as_str);
    let output = Command::new("git")
        .args(["show", &format!("{base_ref}:{base_path}")])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| line_count(&String::from_utf8_lossy(&output.stdout)))
}

fn collect_failures(
    files: &[SourceFile],
    max_lines: i64,
    base_ref: &str,
    base_paths: &HashMap<String, String>,
) -> Vec<CheckFailure> {
    let mut failures = Vec::new();
    for file in files {
        if file.lines as i64 <= max_lines {
            continue;
        }
        let base_lines = base_file_lines(base_ref, &file.path, base_paths);
        let reason = match base_lines {
            None => Some(FailureReason::NewFileExceededLimit),
            Some(lines) if lines as i64 <= max_lines => Some(FailureReason::CrossedLimit),
            Some(lines) if file.lines > lines => Some(FailureReason::OverLimitFileGrew),
            Some(_) => None,
        };
        if let Some(reason) = reason {
            failures.push(CheckFailure {
                path: file.path.clone(),
                lines: file.lines,
                base_lines,
                reason,
            });
        }
    }
    failures.sort_by(|a, b| b.lines.cmp(&a.lines).then_with(|| a.path.cmp(&b.path)));
    failures
}

fn print_inventory(files: &[SourceFile], max_lines: i64, limit: i64) {
    let over_limit = files
        .iter()
        .filter(|file| file.lines as i64 > max_lines)
        .collect::<Vec<_>>();
    println!("Source files scanned: {}", files.len());
    println!("Files over {max_lines} lines: {}", over_limit.len());
    println!("\n| Lines | Path |\n| ---: | --- |");
    for file in over_limit.into_iter().take(limit.max(0) as usize) {
        println!("| {} | `{}` |", file.lines, file.path);
    }
}

fn print_failures(failures: &[CheckFailure]) {
    println!("| Lines | Base | Reason | Path |");
    println!("| ---: | ---: | --- | --- |");
    for failure in failures {
        let base = failure
            .base_lines
            .map(|lines| lines.to_string())
            .unwrap_or_else(|| "-".to_string());
        println!(
            "| {} | {} | {} | `{}` |",
            failure.lines,
            base,
            failure_reason_label(&failure.reason),
            failure.path
        );
    }
}

fn failure_reason_label(reason: &FailureReason) -> &'static str {
    match reason {
        FailureReason::NewFileExceededLimit => "new file exceeds limit",
        FailureReason::CrossedLimit => "crossed limit",
        FailureReason::OverLimitFileGrew => "over-limit file grew",
    }
}

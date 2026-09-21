//! TS-42 driver (P5-9). `tools/commands/davinci/incremental-equivalence.rs`
//! runs this with the committed edit scripts and the corpus roots:
//!
//! ```text
//! cargo run -p vize_resident --example incremental_equivalence -- \
//!   --scripts <dir> --root <label>=<dir> [--root ...]
//! ```
//!
//! Prints the report's `key=value` counts (and one `mismatch` line per
//! differing state). Exit status: 0 equivalent, 1 mismatches, 2 nothing
//! compared or a usage/input error.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use vize_resident::equivalence::{EditScript, EquivalenceReport, parse_script};
use vize_s0::String;

fn main() -> ExitCode {
    match run() {
        Ok(report) => {
            print!("{}", report.summary());
            match report.verdict() {
                Ok(()) => ExitCode::SUCCESS,
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(if report.mismatches.is_empty() { 2 } else { 1 })
                }
            }
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<EquivalenceReport, String> {
    let mut scripts_dir = None;
    let mut roots: Vec<(String, PathBuf)> = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args.next().ok_or_else(|| usage("missing value"))?;
        match arg.as_str() {
            "--scripts" => scripts_dir = Some(PathBuf::from(value)),
            "--root" => {
                let (label, dir) = value
                    .split_once('=')
                    .ok_or_else(|| usage("--root <label>=<dir>"))?;
                roots.push((String::from(label), PathBuf::from(dir)));
            }
            _ => return Err(usage("unknown argument")),
        }
    }
    let scripts = read_scripts(&scripts_dir.ok_or_else(|| usage("--scripts is required"))?)?;
    let mut report = EquivalenceReport::default();
    for (label, dir) in roots {
        let mut files = Vec::new();
        collect_vue_files(&dir, &mut files)?;
        files.sort();
        for path in files {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let relative = path.strip_prefix(&dir).unwrap_or(&path).to_string_lossy();
            let mut name = label.clone();
            name.push('/');
            name.push_str(&relative);
            report.check_file(&name, &text, &scripts);
        }
    }
    Ok(report)
}

fn usage(problem: &str) -> String {
    let mut message = String::from(problem);
    message.push_str(
        "\nusage: incremental_equivalence --scripts <dir> --root <label>=<dir> [--root ...]",
    );
    message
}

fn read_scripts(dir: &Path) -> Result<Vec<EditScript>, String> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|error| io_error(dir, &error))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "edits"))
        .collect();
    paths.sort();
    let mut scripts = Vec::new();
    for path in paths {
        let source = std::fs::read_to_string(&path).map_err(|error| io_error(&path, &error))?;
        let name = path.file_stem().map_or_else(String::default, |stem| {
            String::from(stem.to_string_lossy().as_ref())
        });
        scripts.push(parse_script(&name, &source)?);
    }
    if scripts.is_empty() {
        return Err(String::from("no *.edits scripts found"));
    }
    Ok(scripts)
}

fn collect_vue_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|error| io_error(dir, &error))? {
        let path = entry.map_err(|error| io_error(dir, &error))?.path();
        let name = path.file_name().map(|name| name.to_string_lossy());
        if name
            .as_deref()
            .is_some_and(|name| name.starts_with('.') || name == "node_modules")
        {
            continue;
        }
        if path.is_dir() {
            collect_vue_files(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "vue") {
            files.push(path);
        }
    }
    Ok(())
}

fn io_error(path: &Path, error: &std::io::Error) -> String {
    let mut message = String::from(path.to_string_lossy().as_ref());
    message.push_str(": ");
    message.push_str(&vize_s0::ToCompactString::to_compact_string(error));
    message
}

#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```
//!
//! Ratchet for the workspace lint contract (`[workspace.lints]` in the root
//! `Cargo.toml`): panic-free shipped code plus reasoned `#[expect]`
//! suppressions. Every workspace member must either opt in with
//! `[lints] workspace = true` or be listed in `PENDING`. The list only shrinks:
//! a listed member that already opted in fails the check too, so each
//! migration PR removes its own line.

use std::{fs, process::ExitCode};

/// Members that have not joined the contract yet. Remove a line in the same
/// PR that adds `[lints] workspace = true` to that member.
const PENDING: &[&str] = &[
    "crates/vize",
    "crates/vize_armature",
    "crates/vize_atelier_core",
    "crates/vize_atelier_dom",
    "crates/vize_atelier_jsx",
    "crates/vize_atelier_sfc",
    "crates/vize_atelier_ssr",
    "crates/vize_atelier_vapor",
    "crates/vize_canon",
    "crates/vize_carton",
    "crates/vize_croquis",
    "crates/vize_croquis_cf",
    "crates/vize_curator",
    "crates/vize_davinci",
    "crates/vize_davinci_derive",
    "crates/vize_dialect_moonbit",
    "crates/vize_doctor",
    "crates/vize_extension_host",
    "crates/vize_extension_sdk",
    "crates/vize_fresco",
    "crates/vize_glyph",
    "crates/vize_impeto",
    "crates/vize_maestro",
    "crates/vize_marquette",
    "crates/vize_musea",
    "crates/vize_patina",
    "crates/vize_relief",
    "crates/vize_s1",
    "crates/vize_s1_to_s2",
    "crates/vize_s2",
    "crates/vize_vitrine",
    "tests/davinci_test_support",
    "tests/vize_test_runner",
    "tools/benchmarks/crates/davinci_harness",
    "tools/benchmarks/crates/vize",
];

fn main() -> ExitCode {
    let check = std::env::args().skip(1).any(|arg| arg == "--check");
    match run(check) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(check: bool) -> Result<(), String> {
    let root = fs::read_to_string("Cargo.toml").map_err(|err| format!("Cargo.toml: {err}"))?;
    let members = workspace_members(&root);
    if members.is_empty() {
        return Err("no workspace members found in Cargo.toml".into());
    }

    let mut problems = Vec::new();
    let mut joined = 0usize;
    for member in &members {
        let manifest_path = format!("{member}/Cargo.toml");
        let manifest =
            fs::read_to_string(&manifest_path).map_err(|err| format!("{manifest_path}: {err}"))?;
        let opted_in = opts_into_workspace_lints(&manifest);
        let pending = PENDING.contains(&member.as_str());
        match (opted_in, pending) {
            (true, true) => problems.push(format!(
                "{member} opted into the workspace lints; remove it from PENDING"
            )),
            (false, false) => problems.push(format!(
                "{member} must add `[lints] workspace = true` (or be listed in PENDING)"
            )),
            (true, false) => joined += 1,
            (false, true) => {}
        }
    }
    for pending in PENDING {
        if !members.iter().any(|member| member == pending) {
            problems.push(format!("{pending} is in PENDING but is not a workspace member"));
        }
    }

    println!(
        "workspace lints: {joined}/{} members joined, {} pending",
        members.len(),
        members.len() - joined
    );
    if problems.is_empty() || !check {
        for problem in &problems {
            println!("  {problem}");
        }
        return Ok(());
    }
    Err(problems.join("\n"))
}

/// Collects the quoted entries of the root `[workspace] members = [...]` array.
fn workspace_members(root: &str) -> Vec<String> {
    let Some(start) = root.find("members = [") else {
        return Vec::new();
    };
    let rest = root.get(start + "members = [".len()..).unwrap_or_default();
    let body = rest.split(']').next().unwrap_or_default();
    body.split(',')
        .filter_map(|entry| {
            let entry = entry.trim();
            entry.strip_prefix('"')?.strip_suffix('"').map(str::to_owned)
        })
        .collect()
}

/// True when the manifest has a `[lints]` table whose body is `workspace = true`.
fn opts_into_workspace_lints(manifest: &str) -> bool {
    let mut in_lints = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_lints = line == "[lints]";
            continue;
        }
        if in_lints && line.replace(' ', "") == "workspace=true" {
            return true;
        }
    }
    false
}

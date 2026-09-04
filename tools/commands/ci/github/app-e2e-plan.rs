#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```

use serde::Serialize;
use std::{env, process::ExitCode};

const BLACKSMITH_RUNNER: &str = "blacksmith-32vcpu-ubuntu-2404";
const FULL_SUITES: &[&str] = &["dev", "vrt", "preview", "check", "lint", "build", "all"];
const CHECK_FIXTURES: &[&str] = &[
    "ant-design-vue",
    "directus",
    "element-plus",
    "elk",
    "frontend-phpcon-do-website",
    "hoppscotch",
    "misskey",
    "naive-ui",
    "npmx.dev",
    "nuxt-ui",
    "primevue",
    "reka-ui",
    "voicevox",
    "vue-vben-admin",
    "vuefes-2025",
    "vuetify",
];
const READINESS_FIXTURES: &[&str] = &["elk", "misskey", "npmx.dev", "nuxt-ui", "reka-ui"];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Row {
    profile: &'static str,
    suite: &'static str,
    shard: &'static str,
    task: &'static str,
    fixtures: Vec<String>,
    needs_playwright: bool,
    timeout: &'static str,
    runner: &'static str,
    cache_key: String,
    worktree_id: String,
    artifact_stem: String,
}

#[derive(Debug)]
struct Args {
    profile: String,
    suite: String,
    shard: Option<String>,
    field: String,
    target_sha: Option<String>,
    run_head_sha: Option<String>,
}

#[derive(Serialize)]
struct Matrix {
    include: Vec<Row>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Evidence {
    schema: &'static str,
    version: u8,
    profile: String,
    suite: String,
    target_sha: String,
    source_head_sha: Option<String>,
    row_count: usize,
    rows: Vec<Row>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let args = parse_args(env::args().skip(1))?;
    if let Some(target_sha) = args.target_sha.as_deref() {
        let run_head_sha = args
            .run_head_sha
            .as_deref()
            .ok_or_else(|| "--run-head-sha is required with --validate-target".to_string())?;
        validate_target(&args.suite, target_sha, run_head_sha)?;
        return Ok(());
    }

    if args.shard.is_none() {
        let rows = plan_rows(&args.profile, &args.suite)?;
        match args.field.as_str() {
            "matrix" => println!(
                "{}",
                serde_json::to_string(&Matrix { include: rows }).unwrap()
            ),
            "count" => println!("{}", rows.len()),
            "evidence" => {
                let target_sha = env::var("E2E_TARGET_SHA").unwrap_or_default();
                let source_head_sha = env::var("E2E_SOURCE_HEAD_SHA")
                    .ok()
                    .filter(|value| !value.is_empty());
                println!(
                    "{}",
                    serde_json::to_string_pretty(&create_evidence(
                        &args.profile,
                        &args.suite,
                        target_sha,
                        source_head_sha,
                    )?)
                    .unwrap()
                );
            }
            _ => return Err("Matrix planning only supports matrix or count fields".to_string()),
        }
        return Ok(());
    }

    let row = find_row(&args.profile, &args.suite, args.shard.as_deref().unwrap())?;
    let value = match args.field.as_str() {
        "fixtures" => row.fixtures.join("\n"),
        "task" => row.task.to_string(),
        "timeout" => row.timeout.to_string(),
        "needs-playwright" => row.needs_playwright.to_string(),
        "cache-key" => row.cache_key,
        "worktree-id" => row.worktree_id,
        "artifact-stem" => row.artifact_stem,
        field => return Err(format!("Unknown row field: {field}")),
    };
    println!("{value}");
    Ok(())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut parsed = Args {
        profile: String::new(),
        suite: "all".to_string(),
        shard: None,
        field: "matrix".to_string(),
        target_sha: None,
        run_head_sha: None,
    };
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} requires a value"));
        match arg.as_str() {
            "--profile" => parsed.profile = value()?,
            "--suite" => parsed.suite = value()?,
            "--shard" => parsed.shard = Some(value()?),
            "--field" => parsed.field = value()?,
            "--validate-target" => parsed.target_sha = Some(value()?),
            "--run-head-sha" => parsed.run_head_sha = Some(value()?),
            _ => return Err(format!("Unknown argument: {arg}")),
        }
    }
    if parsed.profile.is_empty() {
        return Err("--profile is required".to_string());
    }
    Ok(parsed)
}

fn fixture(id: &str) -> String {
    format!("tests/_fixtures/_git/{id}")
}

fn row(
    profile: &'static str,
    suite: &'static str,
    shard: &'static str,
    task: &'static str,
    fixtures: &[&str],
    needs_playwright: bool,
    timeout: &'static str,
) -> Row {
    let prefix = format!("{profile}-{suite}-{shard}");
    Row {
        profile,
        suite,
        shard,
        task,
        fixtures: fixtures.iter().map(|id| fixture(id)).collect(),
        needs_playwright,
        timeout,
        runner: BLACKSMITH_RUNNER,
        cache_key: format!("app-e2e-{prefix}"),
        worktree_id: format!("ci-app-e2e-{prefix}"),
        artifact_stem: prefix,
    }
}

#[rustfmt::skip]
fn full_rows() -> Vec<Row> {
    let lint = CHECK_FIXTURES
        .iter()
        .copied()
        .filter(|id| *id != "frontend-phpcon-do-website")
        .collect::<Vec<_>>();
    vec![
        row("full", "dev", "elk", "test:dev:elk", &["elk"], true, "12m"),
        row("full", "dev", "misskey", "test:dev:misskey", &["misskey"], true, "12m"),
        row("full", "dev", "npmx", "test:dev:npmx", &["npmx.dev"], true, "12m"),
        row("full", "dev", "nuxt-ui", "test:dev:nuxt-ui", &["nuxt-ui"], true, "15m"),
        row("full", "dev", "vuefes", "test:dev:vuefes", &["vuefes-2025"], true, "12m"),
        row("full", "vrt", "elk", "test:vrt:elk", &["elk"], true, "15m"),
        row("full", "vrt", "frontend-phpcon", "test:vrt:frontend-phpcon", &["frontend-phpcon-do-website"], true, "15m"),
        row("full", "vrt", "misskey", "test:vrt:misskey", &["misskey"], true, "20m"),
        row("full", "vrt", "npmx", "test:vrt:npmx", &["npmx.dev"], true, "15m"),
        row("full", "vrt", "vuefes", "test:vrt:vuefes", &["vuefes-2025"], true, "15m"),
        row("full", "preview", "elk", "test:preview:elk", &["elk"], false, "10m"),
        row("full", "preview", "misskey", "test:preview:misskey", &["misskey"], false, "10m"),
        row("full", "preview", "npmx", "test:preview:npmx", &["npmx.dev"], false, "10m"),
        row("full", "preview", "vuefes", "test:preview:vuefes", &["vuefes-2025"], false, "10m"),
        row("full", "build", "all", "test:build", &["elk", "misskey", "npmx.dev", "vuefes-2025"], false, "10m"),
        row("full", "check", "all", "test:check", CHECK_FIXTURES, false, "75m"),
        row("full", "lint", "all", "test:lint", &lint, false, "10m"),
    ]
}

/// The shared local-package prelude now runs before the row (see the
/// "Build local Vize packages" step), so a row pays only for its own
/// fixture setup and work. `check` and `lint` still carry misskey's
/// workspace builds inside them, which a release commit lands on a cold
/// cache; both are sized for that, the rest for the warm path they
/// already meet.
#[rustfmt::skip]
fn readiness_rows() -> Vec<Row> {
    vec![
        row("readiness", "readiness", "check", "test:readiness:check", READINESS_FIXTURES, false, "25m"),
        row("readiness", "readiness", "check-vuefes", "test:readiness:check:vuefes", &["vuefes-2025"], false, "2m"),
        row("readiness", "readiness", "lint", "test:readiness:lint", READINESS_FIXTURES, false, "20m"),
        row("readiness", "readiness", "build", "test:readiness:build", &["elk"], false, "3m"),
        row("readiness", "readiness", "dev-misskey", "test:readiness:dev:misskey", &["misskey"], true, "8m"),
        row("readiness", "readiness", "dev-nuxt-ui", "test:readiness:dev:nuxt-ui", &["nuxt-ui"], true, "20m"),
    ]
}

fn plan_rows(profile: &str, suite: &str) -> Result<Vec<Row>, String> {
    match profile {
        "readiness" => {
            if suite != "all" && suite != "readiness" {
                return Err(format!("Unknown readiness suite: {suite}"));
            }
            Ok(readiness_rows())
        }
        "full" => {
            if !FULL_SUITES.contains(&suite) {
                return Err(format!("Unknown full App E2E suite: {suite}"));
            }
            let rows = full_rows();
            let selected = if suite == "all" {
                rows
            } else {
                rows.into_iter().filter(|row| row.suite == suite).collect()
            };
            if selected.is_empty() {
                Err(format!("App E2E suite selected no rows: {suite}"))
            } else {
                Ok(selected)
            }
        }
        _ => Err(format!("Unknown App E2E profile: {profile}")),
    }
}

fn find_row(profile: &str, suite: &str, shard: &str) -> Result<Row, String> {
    let lookup_suite = if profile == "readiness" { "all" } else { suite };
    let matches = plan_rows(profile, lookup_suite)?
        .into_iter()
        .filter(|row| row.suite == suite && row.shard == shard)
        .collect::<Vec<_>>();
    if matches.len() == 1 {
        Ok(matches.into_iter().next().unwrap())
    } else {
        Err(format!("Unknown App E2E row: {profile}:{suite}:{shard}"))
    }
}

fn validate_target(suite: &str, target_sha: &str, run_head_sha: &str) -> Result<(), String> {
    if target_sha.is_empty() {
        if suite == "all" {
            return Err("target_sha is required when suite=all".to_string());
        }
        return Ok(());
    }
    if target_sha.len() != 40
        || !target_sha
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase())
    {
        return Err("target_sha must be a full lowercase 40-character commit SHA".to_string());
    }
    if run_head_sha != target_sha {
        return Err(format!(
            "dispatch ref must resolve to target_sha {target_sha}; got {run_head_sha}"
        ));
    }
    Ok(())
}

fn create_evidence(
    profile: &str,
    suite: &str,
    target_sha: String,
    source_head_sha: Option<String>,
) -> Result<Evidence, String> {
    if target_sha.len() != 40
        || !target_sha
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase())
    {
        return Err("Plan evidence requires an exact target SHA".to_string());
    }
    if source_head_sha.as_deref().is_some_and(|sha| {
        sha.len() != 40
            || !sha
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase())
    }) {
        return Err("Plan evidence source head must be an exact SHA".to_string());
    }
    let rows = plan_rows(profile, suite)?;
    Ok(Evidence {
        schema: "vize.appE2ePlanEvidence",
        version: 1,
        profile: profile.to_string(),
        suite: suite.to_string(),
        target_sha,
        source_head_sha,
        row_count: rows.len(),
        rows,
    })
}

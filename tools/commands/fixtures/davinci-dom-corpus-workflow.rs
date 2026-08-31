#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! regex = "1"
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../rust/common.rs"]
mod common;

use regex::Regex;
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    env, fs,
    io::{Read, Write},
    path::Path,
    process::{Command, ExitCode, Stdio},
    sync::{Arc, Mutex},
    thread,
};

const ARTIFACT_DIR: &str = "real-project-davinci-dom-corpus";
const CORPUS_ROOT: &str = "tests/_fixtures/_git";
const EXPECTED_GITLINKS: usize = 146;
const EXPECTED_DOM_OUTPUT_COMPARISONS: usize = 144;
const EXPECTED_OLD_ERROR_SKIPS: usize = 16;

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<u8, String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [command] if command == "hydrate" => hydrate_corpus(ARTIFACT_DIR),
        [command] if command == "run" => run_corpus(),
        [command] if command == "finalize" => finalize_corpus(ARTIFACT_DIR),
        _ => Err(
            "usage: rust-script tools/commands/fixtures/davinci-dom-corpus-workflow.rs hydrate|run|finalize"
                .to_string(),
        ),
    }
}

fn hydrate_corpus(artifact: &str) -> Result<u8, String> {
    fs::create_dir_all(artifact).map_err(|error| format!("cannot create {artifact}: {error}"))?;
    let fixture_paths = selected_gitlinks()?;
    if fixture_paths.len() != EXPECTED_GITLINKS {
        eprintln!(
            "::error title=Unexpected fixture gitlinks::expected {EXPECTED_GITLINKS}, got {}",
            fixture_paths.len()
        );
        return Ok(1);
    }
    common::write_text(
        Path::new(artifact).join("selected-gitlinks.txt"),
        &format!("{}\n", fixture_paths.join("\n")),
    )?;

    let mut bulk_args = vec![
        "submodule",
        "update",
        "--init",
        "--checkout",
        "--depth",
        "1",
        "--jobs",
        "8",
        "--",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();
    bulk_args.extend(fixture_paths.iter().cloned());
    if run_git(&bulk_args).is_err() {
        eprintln!("::warning title=Davinci corpus hydrate serial fallback::bulk shallow failed");
        for fixture_path in &fixture_paths {
            hydrate_fixture_serially(fixture_path)?;
        }
    }

    let status = run_git(&["submodule", "status", "--", CORPUS_ROOT].map(str::to_string))?;
    common::write_text(Path::new(artifact).join("submodule-status.txt"), &status)?;
    Ok(0)
}

fn hydrate_fixture_serially(fixture_path: &str) -> Result<(), String> {
    let shallow = [
        "submodule",
        "update",
        "--init",
        "--checkout",
        "--depth",
        "1",
        "--jobs",
        "1",
        "--",
        fixture_path,
    ]
    .map(str::to_string);
    if run_git(&shallow).is_ok() {
        return Ok(());
    }
    eprintln!(
        "::warning title=Davinci corpus hydrate full fallback::{fixture_path}: shallow failed"
    );
    run_git(
        &[
            "submodule",
            "update",
            "--init",
            "--checkout",
            "--force",
            "--",
            fixture_path,
        ]
        .map(str::to_string),
    )?;
    Ok(())
}

fn run_corpus() -> Result<u8, String> {
    fs::create_dir_all(ARTIFACT_DIR)
        .map_err(|error| format!("cannot create {ARTIFACT_DIR}: {error}"))?;
    let log_path = Path::new(ARTIFACT_DIR).join("dom-corpus.log");
    let log = fs::File::create(&log_path)
        .map_err(|error| format!("cannot create {}: {error}", log_path.display()))?;
    let log = Arc::new(Mutex::new(log));
    let mut child = Command::new("cargo")
        .args([
            "test",
            "-p",
            "vize_s1_to_s2",
            "--features",
            "davinci-differential",
            "--test",
            "davinci_dom_corpus",
            "--",
            "--nocapture",
        ])
        .env("VIZE_DAVINCI_DIFFERENTIAL_CORPUS", CORPUS_ROOT)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to run davinci DOM corpus: {error}"))?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let out_log = Arc::clone(&log);
    let err_log = Arc::clone(&log);
    let out_thread = thread::spawn(move || copy_stream(stdout, std::io::stdout(), out_log));
    let err_thread = thread::spawn(move || copy_stream(stderr, std::io::stderr(), err_log));
    let status = child
        .wait()
        .map_err(|error| format!("failed to wait for davinci DOM corpus: {error}"))?;
    out_thread
        .join()
        .map_err(|_| "failed to join stdout thread".to_string())??;
    err_thread
        .join()
        .map_err(|_| "failed to join stderr thread".to_string())??;
    Ok(status.code().unwrap_or(1) as u8)
}

fn finalize_corpus(artifact: &str) -> Result<u8, String> {
    fs::create_dir_all(artifact).map_err(|error| format!("cannot create {artifact}: {error}"))?;
    let mode = env::var("VIZE_DAVINCI_DOM_CORPUS_MODE").unwrap_or_else(|_| "enforce".to_string());
    let outcome =
        env::var("VIZE_DAVINCI_DOM_CORPUS_OUTCOME").unwrap_or_else(|_| "failure".to_string());
    let validation = validate_corpus_evidence(artifact);
    let mut verdict = verdict_for(&outcome, &mode);
    if outcome == "success" && !validation.failures.is_empty() {
        verdict = "failure".to_string();
    }
    common::write_json_compact(
        Path::new(artifact).join("summary.json"),
        &json!({
            "mode": mode,
            "outcome": outcome,
            "verdict": verdict,
            "manifestDomOutputComparisons": validation.manifest_dom_output_comparisons,
            "selectedGitlinks": validation.selected_gitlinks,
            "submoduleStatusRows": validation.submodule_status_rows,
            "evidence": validation.evidence,
            "failures": validation.failures,
        }),
    )?;
    append_corpus_summary(artifact, &mode, &outcome, &verdict)?;
    if verdict != "success" {
        for failure in &validation.failures {
            eprintln!("::error title=Invalid Davinci S2 DOM corpus evidence::{failure}");
        }
        eprintln!("::error title=Davinci S2 DOM corpus failed::mode={mode} verdict={verdict}");
        return Ok(1);
    }
    Ok(0)
}

fn selected_gitlinks() -> Result<Vec<String>, String> {
    let index_output = run_git(&["ls-files", "--stage", "--", CORPUS_ROOT].map(str::to_string))?;
    parse_fixture_gitlinks(&index_output)
}

fn parse_fixture_gitlinks(index_output: &str) -> Result<Vec<String>, String> {
    let pattern = Regex::new(r"^160000 [0-9a-f]{40} 0\t(.+)$").unwrap();
    let mut links = index_output
        .lines()
        .filter_map(|line| {
            pattern
                .captures(line)
                .map(|captures| captures[1].to_string())
        })
        .collect::<Vec<_>>();
    links.sort();
    Ok(links)
}

fn verdict_for(outcome: &str, mode: &str) -> String {
    if mode == "record-only" && outcome == "failure" {
        "success".to_string()
    } else {
        outcome.to_string()
    }
}

fn validate_corpus_evidence(artifact: &str) -> Validation {
    let selected = read_optional_lines(Path::new(artifact).join("selected-gitlinks.txt"));
    let status = read_optional_lines(Path::new(artifact).join("submodule-status.txt"));
    let evidence = parse_corpus_evidence(&common::read_optional_text(
        Path::new(artifact).join("dom-corpus.log"),
    ));
    let mut failures = Vec::new();
    if EXPECTED_DOM_OUTPUT_COMPARISONS != 144 {
        failures.push(format!(
            "manifest DOM-output comparisons {EXPECTED_DOM_OUTPUT_COMPARISONS} != 144"
        ));
    }
    if selected.len() != EXPECTED_GITLINKS {
        failures.push(format!(
            "selected gitlinks {} != {EXPECTED_GITLINKS}",
            selected.len()
        ));
    }
    if status.len() != EXPECTED_GITLINKS {
        failures.push(format!(
            "submodule status rows {} != {EXPECTED_GITLINKS}",
            status.len()
        ));
    }
    if !evidence.canonical_scope || !evidence.closure_evidence {
        failures.push("corpus log is missing canonical closure evidence".to_string());
    }
    if evidence.submodules != EXPECTED_GITLINKS {
        failures.push(format!(
            "corpus log submodules {} != {EXPECTED_GITLINKS}",
            evidence.submodules
        ));
    }
    if evidence.files == 0 || evidence.templates == 0 || evidence.compared == 0 {
        failures.push("corpus log proves no DOM-output comparisons".to_string());
    }
    if evidence.unreadable != 0 {
        failures.push(format!(
            "corpus log unreadable inputs: unreadable={}",
            evidence.unreadable
        ));
    }
    let expected_old_error_reasons = expected_old_error_reasons();
    if evidence.old_error_skips != EXPECTED_OLD_ERROR_SKIPS
        || !same_reason_counts(&evidence.old_error_reasons, &expected_old_error_reasons)
    {
        let reasons = format_reason_counts(&evidence.old_error_reasons);
        failures.push(format!(
            "corpus old-lane skip allowlist drift: old_error_skips={}/{} reasons={} expected_reasons={}",
            evidence.old_error_skips,
            EXPECTED_OLD_ERROR_SKIPS,
            if reasons.is_empty() { "none".to_string() } else { reasons },
            format_reason_counts(&expected_old_error_reasons)
        ));
    }
    if evidence.s2_refusals != 0 || evidence.divergences != 0 {
        failures.push(format!(
            "corpus log is not clean: s2_refusals={} divergences={}",
            evidence.s2_refusals, evidence.divergences
        ));
    }
    Validation {
        manifest_dom_output_comparisons: EXPECTED_DOM_OUTPUT_COMPARISONS,
        selected_gitlinks: selected.len(),
        submodule_status_rows: status.len(),
        evidence,
        failures,
    }
}

#[derive(Debug)]
struct Validation {
    manifest_dom_output_comparisons: usize,
    selected_gitlinks: usize,
    submodule_status_rows: usize,
    evidence: CorpusEvidence,
    failures: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CorpusEvidence {
    canonical_scope: bool,
    closure_evidence: bool,
    submodules: usize,
    files: usize,
    unreadable: usize,
    parsed: usize,
    templates: usize,
    compared: usize,
    old_error_skips: usize,
    old_error_reasons: BTreeMap<String, usize>,
    s2_refusals: usize,
    divergences: usize,
}

fn parse_corpus_evidence(log_text: &str) -> CorpusEvidence {
    let mut evidence = CorpusEvidence {
        canonical_scope: false,
        closure_evidence: false,
        submodules: 0,
        files: 0,
        unreadable: 0,
        parsed: 0,
        templates: 0,
        compared: 0,
        old_error_skips: 0,
        old_error_reasons: BTreeMap::new(),
        s2_refusals: 0,
        divergences: 0,
    };
    let scope =
        Regex::new(r"scope=canonical closure_evidence=(true|false) submodules=(\d+)").unwrap();
    let sweep = Regex::new(r"files=(\d+) unreadable=(\d+) parsed=(\d+) templates=(\d+) compared=(\d+) old_error_skips=(\d+) s2_refusals=(\d+) divergences=(\d+)").unwrap();
    let reasons = Regex::new(r"old-lane error reasons: (\{.*\})").unwrap();
    for line in corpus_evidence_lines(log_text) {
        let line = strip_ansi(&line);
        if let Some(captures) = scope.captures(&line) {
            evidence.canonical_scope = true;
            evidence.closure_evidence = &captures[1] == "true";
            evidence.submodules = captures[2].parse().unwrap_or(0);
            continue;
        }
        if let Some(captures) = sweep.captures(&line) {
            evidence.files = captures[1].parse().unwrap_or(0);
            evidence.unreadable = captures[2].parse().unwrap_or(0);
            evidence.parsed = captures[3].parse().unwrap_or(0);
            evidence.templates = captures[4].parse().unwrap_or(0);
            evidence.compared = captures[5].parse().unwrap_or(0);
            evidence.old_error_skips = captures[6].parse().unwrap_or(0);
            evidence.s2_refusals = captures[7].parse().unwrap_or(0);
            evidence.divergences = captures[8].parse().unwrap_or(0);
            continue;
        }
        if let Some(captures) = reasons.captures(&line) {
            evidence.old_error_reasons = sort_reason_counts(
                serde_json::from_str::<BTreeMap<String, Value>>(&captures[1]).unwrap_or_default(),
            );
        }
    }
    if evidence.old_error_reasons.is_empty() {
        evidence.old_error_reasons = parse_old_error_reasons(log_text);
    }
    evidence
}

fn corpus_evidence_lines(log_text: &str) -> Vec<String> {
    log_text
        .lines()
        .filter(|line| {
            let line = strip_ansi(line);
            line.contains("davinci-differential corpus scope")
                || line.contains("davinci DOM corpus sweep")
                || line.contains("davinci DOM corpus old-lane error reasons")
        })
        .map(str::to_string)
        .collect()
}

fn parse_old_error_reasons(log_text: &str) -> BTreeMap<String, usize> {
    let stripped = strip_ansi(log_text);
    let explicit = Regex::new(r"old-lane error reasons: (\{.*\})").unwrap();
    for captures in explicit.captures_iter(&stripped) {
        return sort_reason_counts(
            serde_json::from_str::<BTreeMap<String, Value>>(&captures[1]).unwrap_or_default(),
        );
    }
    let block = Regex::new(
        r"(?s)corpus old-lane error skips \(\d+\)(?: by reason \{.*\})?:\n(.*?)\n\ncorpus S2 refusals",
    )
    .unwrap();
    let Some(captures) = block.captures(&stripped) else {
        return BTreeMap::new();
    };
    let code = Regex::new(r"code: ([A-Za-z0-9_]+)").unwrap();
    let mut reasons = BTreeMap::new();
    for capture in code.captures_iter(&captures[1]) {
        *reasons.entry(capture[1].to_string()).or_default() += 1;
    }
    reasons
}

fn sort_reason_counts(reasons: BTreeMap<String, Value>) -> BTreeMap<String, usize> {
    reasons
        .into_iter()
        .filter_map(|(reason, count)| {
            let count = count.as_u64()? as usize;
            (count > 0).then_some((reason, count))
        })
        .collect()
}

fn format_reason_counts(reasons: &BTreeMap<String, usize>) -> String {
    reasons
        .iter()
        .map(|(reason, count)| format!("{reason}={count}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn expected_old_error_reasons() -> BTreeMap<String, usize> {
    [
        ("ExtendPoint", 1),
        ("InvalidEndTag", 20),
        ("MissingEndTag", 10),
        ("MissingWhitespaceBetweenAttributes", 4),
        ("VElseNoAdjacentIf", 1),
        ("VIfSameKey", 4),
        ("VSlotDuplicateSlotNames", 1),
    ]
    .into_iter()
    .map(|(reason, count)| (reason.to_string(), count))
    .collect()
}

fn same_reason_counts(
    left: &BTreeMap<String, usize>,
    right: &BTreeMap<String, usize>,
) -> bool {
    left == right
}

fn append_corpus_summary(
    artifact: &str,
    mode: &str,
    outcome: &str,
    verdict: &str,
) -> Result<(), String> {
    let Some(summary_path) = env::var_os("GITHUB_STEP_SUMMARY") else {
        return Ok(());
    };
    let validation = validate_corpus_evidence(artifact);
    let evidence_lines = corpus_evidence_lines(&common::read_optional_text(
        Path::new(artifact).join("dom-corpus.log"),
    ));
    common::append_text(
        summary_path,
        &format!(
            "## Davinci S2 DOM Corpus\n\n- mode: `{mode}`\n- outcome: `{outcome}`\n- verdict: `{verdict}`\n- manifest DOM-output comparisons: `{}`\n- gitlinks: `{}`\n- submodule status rows: `{}`\n- compared templates: `{}`\n- old-lane error reasons: `{}`\n\n{}\n",
            validation.manifest_dom_output_comparisons,
            validation.selected_gitlinks,
            validation.submodule_status_rows,
            validation.evidence.compared,
            {
                let reasons = format_reason_counts(&validation.evidence.old_error_reasons);
                if reasons.is_empty() {
                    "none".to_string()
                } else {
                    reasons
                }
            },
            evidence_lines.join("\n")
        ),
    )
}

fn read_optional_lines(path: impl AsRef<Path>) -> Vec<String> {
    common::read_optional_text(path)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn strip_ansi(value: &str) -> String {
    Regex::new(r"\x1b\[[0-?]*[ -/]*[@-~]")
        .unwrap()
        .replace_all(value, "")
        .into_owned()
}

fn run_git(args: &[String]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed with exit code {}",
            args.join(" "),
            output.status.code().unwrap_or(1)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn copy_stream(
    mut input: impl Read,
    mut output: impl Write,
    log: Arc<Mutex<fs::File>>,
) -> Result<(), String> {
    let mut buffer = [0u8; 8192];
    loop {
        let count = input
            .read(&mut buffer)
            .map_err(|error| format!("cannot read child output: {error}"))?;
        if count == 0 {
            return Ok(());
        }
        output
            .write_all(&buffer[..count])
            .map_err(|error| format!("cannot write child output: {error}"))?;
        log.lock()
            .map_err(|_| "cannot lock corpus log".to_string())?
            .write_all(&buffer[..count])
            .map_err(|error| format!("cannot write corpus log: {error}"))?;
    }
}

//! `davinci-opt`'s P2-13 pipeline-mode extras, pinned end to end: the
//! `--folio-dir` per-pass dump, its `--folio-after-change` hash gate, and the
//! `--timing-json` export validated against the committed P0-11 schema
//! through the strict shared validator (TS-15).

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const USAGE: &str = "usage: davinci-opt --roundtrip <file> [--stage croquis]\n       davinci-opt --pipeline \"<syntax>\" [--stage <stage>] [--folio-dir <dir> [--folio-after-change]] [--timing-json <path>] < folio\n";

/// A canonical `[budget-observer]` page - the smallest committed-format
/// artifact a pipeline can run over.
const BUDGET: &str =
    "[budget-observer]\nwalks=2\npasses=3\nanalyses=0\npipelines=1\nfailures=0\n\n";

fn run(args: &[&str], stdin: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_davinci-opt"));
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("davinci-opt spawns");
    if let Some(text) = stdin {
        child
            .stdin
            .as_mut()
            .expect("stdin is piped")
            .write_all(text.as_bytes())
            .expect("stdin accepts the folio");
    }
    drop(child.stdin.take());
    child.wait_with_output().expect("davinci-opt exits")
}

fn stdout_of(output: &Output) -> &str {
    core::str::from_utf8(&output.stdout).expect("stdout is UTF-8")
}

fn stderr_of(output: &Output) -> &str {
    core::str::from_utf8(&output.stderr).expect("stderr is UTF-8")
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn sorted_entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .expect("dump directory exists")
        .map(|entry| {
            entry
                .expect("readable dump entry")
                .file_name()
                .into_string()
                .expect("UTF-8 page name")
        })
        .collect();
    names.sort();
    names
}

#[test]
fn folio_dir_writes_one_page_per_pass() {
    let dir = temp_dir("p2-13-folio-dir");
    let output = run(
        &[
            "--pipeline",
            "s2(alpha,beta)",
            "--stage",
            "budget-observer",
            "--folio-dir",
            dir.to_str().expect("UTF-8 dir"),
        ],
        Some(BUDGET),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_of(&output), BUDGET);
    assert_eq!(
        stderr_of(&output),
        format!(
            "davinci-opt: pipeline s2(alpha,beta): walks=1 passes=2\n\
             davinci-opt: folio-dir {}: 2 page(s)\n",
            dir.display()
        )
    );
    // Pages plus the directory's Spolvero feed (P2-18); the feed's content
    // is pinned by `tests/spolvero_feed.rs` (TS-52).
    assert_eq!(
        sorted_entries(&dir),
        ["000-s2.alpha.folio", "001-s2.beta.folio", "spolvero.json"]
    );
    for name in ["000-s2.alpha.folio", "001-s2.beta.folio"] {
        let text = std::fs::read_to_string(dir.join(name)).expect("page reads");
        assert_eq!(text, BUDGET, "{name}");
    }
}

#[test]
fn folio_after_change_gates_noop_passes_to_an_empty_directory() {
    let dir = temp_dir("p2-13-after-change");
    let output = run(
        &[
            "--pipeline",
            "s2(alpha,beta)",
            "--stage",
            "budget-observer",
            "--folio-dir",
            dir.to_str().expect("UTF-8 dir"),
            "--folio-after-change",
        ],
        Some(BUDGET),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_of(&output), BUDGET);
    assert_eq!(
        stderr_of(&output),
        format!(
            "davinci-opt: pipeline s2(alpha,beta): walks=1 passes=2\n\
             davinci-opt: folio-dir {}: 0 page(s)\n",
            dir.display()
        )
    );
    // The directory exists and holds no pages - only the Spolvero feed,
    // whose zero-page content says "the gate emitted nothing" explicitly
    // (pinned by `tests/spolvero_feed.rs`), so the empty run stays
    // observable, not indistinguishable from an ignored flag.
    assert_eq!(sorted_entries(&dir), ["spolvero.json"]);
}

#[test]
fn the_dump_and_timing_flags_are_pipeline_mode_only() {
    for (args, message) in [
        (
            &["--roundtrip", "x.folio", "--folio-dir", "d"][..],
            "--folio-dir requires --pipeline",
        ),
        (
            &["--roundtrip", "x.folio", "--timing-json", "t.json"][..],
            "--timing-json requires --pipeline",
        ),
        (
            &["--pipeline", "s2()", "--folio-after-change"][..],
            "--folio-after-change requires --folio-dir",
        ),
    ] {
        let output = run(args, Some(""));
        assert_eq!(output.status.code(), Some(2), "{message}: exit code");
        assert_eq!(stdout_of(&output), "", "{message}: stdout");
        assert_eq!(
            stderr_of(&output),
            format!("davinci-opt: {message}\n{USAGE}"),
            "{message}: stderr"
        );
    }
}

#[test]
fn timing_json_satisfies_the_p0_11_schema() {
    let dir = temp_dir("p2-13-timing");
    std::fs::create_dir_all(&dir).expect("timing dir creates");
    let path = dir.join("timing.json");
    let output = run(
        &[
            "--pipeline",
            "s2(alpha,beta)",
            "--stage",
            "budget-observer",
            "--timing-json",
            path.to_str().expect("UTF-8 path"),
        ],
        Some(BUDGET),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_of(&output), BUDGET);
    assert_eq!(
        stderr_of(&output),
        "davinci-opt: pipeline s2(alpha,beta): walks=1 passes=2\n"
    );

    let text = std::fs::read_to_string(&path).expect("timing export reads");
    let json: serde_json::Value = serde_json::from_str(&text).expect("export is valid JSON");
    let schema = load_schema();
    assert_eq!(schema_check::validate(&schema, &json, "$"), Ok(()));

    // The deterministic structure: one fused walk, recorded under the
    // davinci key with the stage and the group's lead pass; no allocator
    // installed, nothing truncated, no counters.
    assert_eq!(json["schema_version"], serde_json::json!(1));
    assert_eq!(json["command"], serde_json::json!("davinci-opt"));
    assert_eq!(json["allocation"], serde_json::Value::Null);
    assert_eq!(json["counters"], serde_json::json!([]));
    assert_eq!(
        json["truncation"],
        serde_json::json!({ "dropped_spans": 0, "dropped_counters": 0 })
    );
    let spans = json["spans"].as_array().expect("spans is an array");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0]["key"], serde_json::json!("davinci.pass.walk"));
    assert_eq!(spans[0]["count"], serde_json::json!(1));
    assert_eq!(spans[0]["alloc"], serde_json::Value::Null);
    assert_eq!(
        spans[0]["attribution"],
        serde_json::json!({ "stage": "s2", "pass": "alpha" })
    );
}

/// Load the committed schema relative to this crate's manifest.
fn load_schema() -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("davinci-road")
        .join("plan")
        .join("profile-export.schema.json");
    let text = std::fs::read_to_string(&path).expect("committed schema reads");
    serde_json::from_str(&text).expect("committed schema is valid JSON")
}
use davinci_test_support::schema as schema_check;

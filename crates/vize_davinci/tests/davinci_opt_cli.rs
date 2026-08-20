//! The extended `davinci-opt` binary, pinned end to end (P2-4).
//!
//! Two jobs:
//!
//! 1. **Regression pin:** the 14 committed P0-10 croquis fixtures still
//!    round-trip byte-identically through the extended binary, with the
//!    roundtrip mode's stdout, stderr and exit codes unchanged.
//! 2. The new `--pipeline` alternative: folio on stdin, full normalized
//!    folio on stdout, plan summary on stderr, and the documented pipeline
//!    rejections surfacing as usage errors - every message asserted
//!    exactly (assurance §4).

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use vize_davinci::folio::{Folio, FolioMode, croquis::CroquisFolio};

const USAGE: &str = "usage: davinci-opt --roundtrip <file> [--stage croquis]\n       davinci-opt --pipeline \"<syntax>\" [--stage <stage>] < folio\n";

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

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("croquis")
}

fn committed_folios() -> Vec<PathBuf> {
    let mut folios: Vec<PathBuf> = std::fs::read_dir(fixture_dir())
        .expect("fixture directory exists")
        .map(|entry| entry.expect("readable fixture entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "folio"))
        .collect();
    folios.sort();
    folios
}

fn temp_file(name: &str, content: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&path, content).expect("temp file writes");
    path
}

/// The regression pin: every committed croquis fixture is round-trip
/// identity through the extended binary, under the default stage and under
/// an explicit `--stage croquis`, with the historical output byte-exact.
#[test]
fn the_fourteen_croquis_fixtures_still_roundtrip_byte_identically() {
    let folios = committed_folios();
    assert_eq!(folios.len(), 14, "the P0-10 fixture set is 14 folios");
    for folio in folios {
        let text = std::fs::read_to_string(&folio).expect("committed folio reads");
        let path = folio.display();
        let expected_stdout = format!("roundtrip OK: {path} ({} bytes)\n", text.len());
        for args in [
            vec!["--roundtrip", folio.to_str().expect("UTF-8 path")],
            vec![
                "--roundtrip",
                folio.to_str().expect("UTF-8 path"),
                "--stage",
                "croquis",
            ],
        ] {
            let output = run(&args, None);
            assert_eq!(output.status.code(), Some(0), "{path}: exit code");
            assert_eq!(stdout_of(&output), expected_stdout, "{path}: stdout");
            assert_eq!(stderr_of(&output), "", "{path}: stderr");
        }
    }
}

#[test]
fn help_prints_the_usage_and_exits_zero() {
    let output = run(&["--help"], None);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_of(&output), USAGE);
    assert_eq!(stderr_of(&output), "");
}

#[test]
fn selecting_no_mode_is_a_usage_error() {
    let output = run(&[], None);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout_of(&output), "");
    assert_eq!(
        stderr_of(&output),
        format!("davinci-opt: --roundtrip <file> or --pipeline \"<syntax>\" is required\n{USAGE}")
    );
}

#[test]
fn selecting_both_modes_is_a_usage_error() {
    let output = run(&["--roundtrip", "x.folio", "--pipeline", "s2()"], None);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        stderr_of(&output),
        format!(
            "davinci-opt: --roundtrip and --pipeline are alternatives, give exactly one\n{USAGE}"
        )
    );
}

#[test]
fn an_unknown_stage_reports_the_available_stages() {
    let folio = &committed_folios()[0];
    let output = run(
        &[
            "--roundtrip",
            folio.to_str().expect("UTF-8 path"),
            "--stage",
            "bogus",
        ],
        None,
    );
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        stderr_of(&output),
        "davinci-opt: unknown stage: bogus (available: budget-observer, croquis)\n"
    );
}

#[test]
fn non_canonical_input_reports_the_first_divergent_line() {
    let input = "[vir]\nscript_setup=false\nscopes=0\nbindings=1\n\n[bindings]\nc:b,a\n\n";
    let printed = CroquisFolio::parse(input)
        .expect("non-canonical text parses")
        .print_to_string(FolioMode::Full);
    let file = temp_file("p2-4-non-canonical.folio", input);
    let output = run(&["--roundtrip", file.to_str().expect("UTF-8 path")], None);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout_of(&output), "");
    assert_eq!(
        stderr_of(&output),
        format!(
            "davinci-opt: {}: round-trip mismatch starting at line 7 (input {} bytes, \
             printed {} bytes); the input is not canonical - canonical text is what \
             print(parse(input)) emits\n",
            file.display(),
            input.len(),
            printed.len(),
        )
    );
}

#[test]
fn a_parse_failure_exits_one_with_the_folio_error() {
    let file = temp_file("p2-4-unparseable.folio", "x\n");
    let output = run(&["--roundtrip", file.to_str().expect("UTF-8 path")], None);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stderr_of(&output),
        format!(
            "davinci-opt: {}: folio parse error at line 1: content before the [vir] header\n",
            file.display()
        )
    );
}

#[test]
fn a_pipeline_streams_the_normalized_folio_and_reports_the_plan() {
    let folio = fixture_dir().join("props-destructure.folio");
    let text = std::fs::read_to_string(&folio).expect("committed folio reads");
    // Committed folios are canonical and the no-op passes change nothing,
    // so stdout is the input byte-for-byte.
    let output = run(&["--pipeline", "s2(alpha,beta)"], Some(&text));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_of(&output), text);
    assert_eq!(
        stderr_of(&output),
        "davinci-opt: pipeline s2(alpha,beta): walks=1 passes=2\n"
    );
    // Segments run as separate plans: one walk each here (no-op passes are
    // fusable, so each segment's passes share one walk).
    let output = run(
        &["--pipeline", "s2(alpha),s2-to-s3(beta,gamma)"],
        Some(&text),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_of(&output), text);
    assert_eq!(
        stderr_of(&output),
        "davinci-opt: pipeline s2(alpha),s2-to-s3(beta,gamma): walks=2 passes=3\n"
    );
    // An empty pass list is legal (`s2()` runs nothing) - the bisection
    // property the pipeline grammar documents.
    let output = run(&["--pipeline", "s2()"], Some(&text));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_of(&output), text);
    assert_eq!(
        stderr_of(&output),
        "davinci-opt: pipeline s2(): walks=0 passes=0\n"
    );
}

#[test]
fn a_pipeline_parses_the_selected_stage() {
    let budget = "[budget-observer]\nwalks=2\npasses=3\nanalyses=0\npipelines=1\nfailures=0\n\n";
    let output = run(
        &["--pipeline", "s2(alpha)", "--stage", "budget-observer"],
        Some(budget),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_of(&output), budget);
    assert_eq!(
        stderr_of(&output),
        "davinci-opt: pipeline s2(alpha): walks=1 passes=1\n"
    );
}

#[test]
fn malformed_pipeline_strings_are_usage_errors_with_the_documented_messages() {
    for (syntax, message) in [
        ("", "empty pipeline string at offset 0"),
        ("s2", "expected `(` after stage name at offset 2"),
        ("s2(a", "unterminated pass list, expected `)` at offset 4"),
        ("S2(a)", "unexpected character `S` at offset 0"),
        ("s2-(a)", "identifier must not end with `-` at offset 2"),
    ] {
        let output = run(&["--pipeline", syntax], Some(""));
        assert_eq!(output.status.code(), Some(2), "{syntax}: exit code");
        assert_eq!(stdout_of(&output), "", "{syntax}: stdout");
        assert_eq!(
            stderr_of(&output),
            format!("davinci-opt: --pipeline: {message}\n"),
            "{syntax}: stderr"
        );
    }
}

#[test]
fn unparseable_stdin_exits_one_before_any_plan_runs() {
    let output = run(&["--pipeline", "s2(alpha)"], Some("x\n"));
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout_of(&output), "");
    assert_eq!(
        stderr_of(&output),
        "davinci-opt: <stdin>: folio parse error at line 1: content before the [vir] header\n"
    );
}

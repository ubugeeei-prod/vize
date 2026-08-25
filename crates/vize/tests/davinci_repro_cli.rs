//! TS-23 - crash-repro replay (P2-13, charter #30), pinned end to end.
//!
//! An injected panic in one file of a batch build must: fail exactly that
//! file, write its `repro.folio`, and leave every other file's output in
//! place (asserted as an **exact file set**); `vize repro` must replay the
//! repro to the **same** failure, compared by exact equality on
//! (stage, pass, reason) - the comparison lives inside the tool and exit 0
//! is its verdict. The negative verdicts (diverged, did-not-reproduce,
//! malformed) are pinned with exact output too.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use vize_davinci::folio::repro::ReproFolio;
use vize_davinci::folio::{Folio, FolioMode};
use vize_s0::String as CartonString;

const INJECTED_FAILURE: &str = "template.transform: injected davinci panic in pass `transform`";

fn temp_project_dir(test_name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vize-davinci-repro-{}-{}-{}",
        std::process::id(),
        test_name,
        nonce
    ))
}

fn write_batch(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    for name in ["a", "b", "c"] {
        fs::write(
            root.join("src").join(format!("{name}.vue")),
            format!("<template><div>{name}</div></template>\n"),
        )
        .unwrap();
    }
}

fn vize(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

fn sorted_entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("directory exists")
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

fn stderr_lines(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .map(str::to_owned)
        .collect()
}

fn has_line(lines: &[String], expected: &str) -> bool {
    lines.iter().any(|line| line == expected)
}

/// Run the injected-panic batch build and return the project root.
fn injected_build(test_name: &str) -> (PathBuf, Output) {
    let root = temp_project_dir(test_name);
    write_batch(&root);
    let output = vize(
        &root,
        &[
            "build",
            "src",
            "--output",
            "dist",
            "--davinci-inject-panic",
            "b:transform",
        ],
    );
    (root, output)
}

#[test]
fn an_injected_panic_fails_one_file_writes_its_repro_and_emits_the_rest() {
    let (root, output) = injected_build("batch");
    assert_eq!(output.status.code(), Some(1));

    // The file-scoped property as an exact file set: the panicking file
    // emitted its repro and nothing else; every other file emitted normally.
    assert_eq!(
        sorted_entries(&root.join("dist")),
        ["a.js", "b.repro.folio", "c.js"]
    );

    // The build names the failure and the repro, through the error report.
    let lines = stderr_lines(&output);
    let ice_header = "  \u{1b}[31mInternal compiler errors (1):\u{1b}[0m";
    let failure_line = format!("      internal compiler error: {INJECTED_FAILURE}");
    let repro_line = format!(
        "      repro: dist{}b.repro.folio",
        std::path::MAIN_SEPARATOR
    );
    assert!(
        has_line(&lines, ice_header),
        "missing {ice_header:?} in {lines:#?}"
    );
    assert!(
        has_line(&lines, &failure_line),
        "missing {failure_line:?} in {lines:#?}"
    );
    assert!(
        has_line(&lines, &repro_line),
        "missing {repro_line:?} in {lines:#?}"
    );

    // The repro is exactly what the contract says it is: pipeline string,
    // config, recorded failure, last-good stage dump (the source).
    let text = fs::read_to_string(root.join("dist/b.repro.folio")).unwrap();
    let folio = ReproFolio::parse(&text).expect("the written repro parses");
    assert_eq!(folio.pipeline.as_str(), "template(transform,codegen)");
    assert_eq!(folio.failed_stage.as_str(), "template");
    assert_eq!(folio.failed_pass.as_str(), "transform");
    assert_eq!(
        folio.reason.as_str(),
        "injected davinci panic in pass `transform`"
    );
    assert_eq!(folio.artifact_stage.as_str(), "source");
    assert_eq!(folio.config.len(), 2);
    assert_eq!(
        folio.config.get("inject-panic").map(CartonString::as_str),
        Some("transform")
    );
    assert_eq!(
        folio.config.get("mode").map(CartonString::as_str),
        Some("dom")
    );
    assert_eq!(
        folio.artifact.as_str(),
        "<template><div>b</div></template>\n"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn vize_repro_replays_to_the_same_failure() {
    let (root, _build) = injected_build("replay");
    let output = vize(&root, &["repro", "dist/b.repro.folio"]);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("repro: reproduced: {INJECTED_FAILURE}\n")
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    assert_eq!(output.status.code(), Some(0));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_tampered_recorded_failure_is_reported_as_divergence() {
    let (root, _build) = injected_build("diverged");
    let text = fs::read_to_string(root.join("dist/b.repro.folio")).unwrap();
    let mut folio = ReproFolio::parse(&text).unwrap();
    folio.reason = CartonString::from("tampered reason");
    fs::write(
        root.join("dist/tampered.repro.folio"),
        folio.print_to_string(FolioMode::Full).as_str(),
    )
    .unwrap();

    let output = vize(&root, &["repro", "dist/tampered.repro.folio"]);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        format!(
            "repro: diverged: replayed {INJECTED_FAILURE} \
             (recorded template.transform: tampered reason)\n"
        )
    );
    assert_eq!(output.status.code(), Some(1));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_repro_whose_replay_completes_reports_did_not_reproduce() {
    let (root, _build) = injected_build("completed");
    let text = fs::read_to_string(root.join("dist/b.repro.folio")).unwrap();
    let mut folio = ReproFolio::parse(&text).unwrap();
    // Without the injection the replay is a real compile of the embedded
    // source, which succeeds - the recorded failure does not come back.
    folio.config.remove("inject-panic");
    fs::write(
        root.join("dist/completed.repro.folio"),
        folio.print_to_string(FolioMode::Full).as_str(),
    )
    .unwrap();

    let output = vize(&root, &["repro", "dist/completed.repro.folio"]);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        format!("repro: did not reproduce: the pipeline completed (recorded {INJECTED_FAILURE})\n")
    );
    assert_eq!(output.status.code(), Some(1));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_malformed_repro_file_is_a_usage_error() {
    let root = temp_project_dir("malformed");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("bad.repro.folio"), "x\n").unwrap();
    let output = vize(&root, &["repro", "bad.repro.folio"]);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "repro: bad.repro.folio: folio parse error at line 1: content before the [repro] header\n"
    );
    assert_eq!(output.status.code(), Some(2));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn the_build_folio_dir_is_created_and_stays_empty_until_p2_12b() {
    // The compile path has no folio-printable stage artifact yet, so the
    // pinned behavior of --folio-dir on `vize build` is: the directory
    // exists and holds zero pages. This test is the "vacuity is measured,
    // not decorative" witness; davinci-opt's twin dumps real pages.
    let root = temp_project_dir("folio-dir");
    write_batch(&root);
    let output = vize(
        &root,
        &["build", "src", "--output", "dist", "--folio-dir", "folios"],
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(sorted_entries(&root.join("dist")), ["a.js", "b.js", "c.js"]);
    assert_eq!(sorted_entries(&root.join("folios")).len(), 0);
    let _ = fs::remove_dir_all(root);
}

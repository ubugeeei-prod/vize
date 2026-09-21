//! TS-42 on the committed plane (P5-9): the committed edit scripts
//! (`tools/commands/davinci/incremental-equivalence/scripts/`) run through
//! one long-lived database per file over every committed project fixture,
//! and every served artifact equals the clean path after every step.
//!
//! The scope is pinned exactly — a run that silently compared less fails.
//! Under the `seeded-stale-cache` feature (the firewall's equality weakened
//! to block lengths) the same run must report stale artifacts, which proves
//! the harness can see the bug class it exists for. The corpus shard runs
//! the same harness in CI (`davinci-incremental.yml`).

use std::path::{Path, PathBuf};

use vize_resident::equivalence::{EditScript, EquivalenceReport, parse_script};
use vize_s0::String;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scripts() -> Vec<EditScript> {
    let dir = repo().join("tools/commands/davinci/incremental-equivalence/scripts");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("committed edit scripts")
        .map(|entry| entry.expect("script entry").path())
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| {
            let name = path.file_stem().expect("stem").to_string_lossy();
            let source = std::fs::read_to_string(path).expect("script text");
            parse_script(&name, &source).expect("committed scripts parse")
        })
        .collect()
}

fn vue_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("fixture dir") {
        let path = entry.expect("fixture entry").path();
        if path.is_dir() {
            vue_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "vue") {
            out.push(path);
        }
    }
}

fn run_plane() -> EquivalenceReport {
    let root = repo().join("tests/_fixtures/_projects");
    let mut files = Vec::new();
    vue_files(&root, &mut files);
    files.sort();
    let scripts = scripts();
    let mut report = EquivalenceReport::default();
    for path in files {
        let text = std::fs::read_to_string(&path).expect("fixture text");
        let name = path
            .strip_prefix(&root)
            .expect("under root")
            .to_string_lossy();
        report.check_file(&name, &text, &scripts);
    }
    report
}

#[cfg(not(feature = "seeded-stale-cache"))]
#[test]
fn incremental_equals_clean_on_every_committed_project_fixture() {
    let report = run_plane();
    println!("{}", report.summary());
    assert_eq!(report.mismatches, []);
    assert_eq!(report.verdict(), Ok(()));
    assert_eq!(
        (
            report.files,
            report.script_runs,
            report.steps_applied,
            report.ops_skipped,
            report.comparisons,
            report.blocks_compared,
        ),
        (67, 402, 2020, 258, 2489, 5390)
    );
}

#[cfg(feature = "seeded-stale-cache")]
#[test]
fn the_seeded_stale_cache_is_caught() {
    let report = run_plane();
    println!("{}", report.summary());
    // Only length-preserving edits slip past the weakened equality; each one
    // leaves the stale block's S0 key (and every artifact behind it) served.
    assert_eq!(report.mismatches.len(), 347);
    assert_eq!(
        report.mismatches[0],
        vize_resident::equivalence::Mismatch {
            file: String::from("class-component/src/App.vue"),
            script: String::from("05-length-preserving"),
            step: 1,
            detail: String::from("template[0] s0-key"),
        }
    );
    let error = report.verdict().expect_err("the seeded bug must be caught");
    assert_eq!(
        error.lines().next(),
        Some("TS-42: incremental artifacts differ from clean ones")
    );
}

#[test]
fn a_run_that_compares_nothing_fails() {
    let empty = EquivalenceReport::default();
    assert_eq!(
        empty.verdict(),
        Err(String::from(
            "TS-42 compared nothing: a zero-file, zero-step or zero-block run proves no equivalence"
        ))
    );
    // Opening a file without any script compares its initial state but
    // applies no edit: still no evidence of incremental equivalence.
    let mut unedited = EquivalenceReport::default();
    unedited.check_file("A.vue", "<template><div/></template>\n", &[]);
    assert_eq!((unedited.comparisons, unedited.steps_applied), (1, 0));
    assert_eq!(unedited.verdict(), empty.verdict());
}

#[test]
fn edit_scripts_reject_malformed_lines_with_their_line_number() {
    let cases = [
        ("bogus template", "s:1: unknown operation"),
        ("# note\n\ntype nowhere \"x\"", "s:3: unknown target"),
        ("prepend x", "s:1: expected a quoted string"),
        (
            "insert-end template \"a\\q\"",
            "s:1: expected a quoted string",
        ),
        ("configure vue1", "s:1: expected vue2 or vue3"),
        ("revert now", "s:1: unknown operation"),
    ];
    for (source, expected) in cases {
        assert_eq!(
            parse_script("s", source),
            Err(String::from(expected)),
            "{source}"
        );
    }
}

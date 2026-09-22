//! TS-34 — the Croquis fact groups against their declarative specs.
//!
//! Every artifact is analyzed by production (`analyze_sfc_descriptor`, the
//! `full` options every lint/check consumer uses) with the `UndefinedRefs`
//! input trace armed, then each group's table is compared by exact equality
//! with the naive evaluator of its spec (`vize_croquis::facts::spec`).
//!
//! Planes, each with its scope proof:
//!
//! - the committed battery ([`battery`]) and the P0-2 fixture ladder, with
//!   pinned counts — a move is a rule, extraction or production change;
//! - the committed P2-15 construct matrix (`tests/fixtures/davinci-matrix`,
//!   90 template-only stubs: `Bindings` compares empty tables, and the
//!   `UndefinedRefs` half covers every checked expression);
//! - a corpus shard: `VIZE_DAVINCI_FACT_CORPUS=<dir>[,<dir>…]` sweeps every
//!   `.vue` file below (the per-PR gate is
//!   `tests/tooling/davinci-fact-spec-corpus.test.ts`).
//!
//! The `UndefinedRefs` trace is recorded in debug builds only; a release
//! run reports that half as not compared instead of passing it.

mod battery;
mod runner;

use std::path::{Path, PathBuf};

use runner::{Planes, collect_vue_files, run_source};

fn matrix_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/davinci-matrix")
}

#[test]
fn the_committed_planes_agree_with_the_specs() {
    let mut battery = Planes::default();
    for (name, source) in battery::SOURCES {
        run_source(name, source, &mut battery);
    }
    battery.assert_verdicts("battery");
    eprintln!("{}", battery.scope_lines("battery"));
    assert_eq!(
        battery.census(),
        ((9, 9, 85), (9, 9, 11)),
        "battery census moved: re-pin deliberately (see the P4-3a record)"
    );

    let mut ladder = Planes::default();
    for fixture in &davinci_harness::fixtures::LADDER {
        run_source(fixture.name, fixture.source, &mut ladder);
    }
    ladder.assert_verdicts("ladder");
    eprintln!("{}", ladder.scope_lines("ladder"));
    assert_eq!(
        ladder.census(),
        ((6, 5, 80), (6, 6, 4)),
        "ladder census moved: re-pin deliberately (see the P4-3a record)"
    );

    let mut files = Vec::new();
    collect_vue_files(&matrix_dir(), &mut files);
    assert_eq!(files.len(), 90, "the committed P2-15 plane moved");
    let mut matrix = Planes::default();
    for file in &files {
        let source = std::fs::read_to_string(file).expect("matrix stub");
        run_source(
            &vize_carton::cstr!("{}", file.display()),
            &source,
            &mut matrix,
        );
    }
    eprintln!("{}", matrix.scope_lines("matrix plane"));
    assert_eq!(
        matrix.census(),
        ((90, 90, 0), (90, 90, 0)),
        "matrix census moved: re-pin deliberately (see the P4-3a record)"
    );
    assert!(matrix.bindings.divergences.is_empty() && matrix.undefined.divergences.is_empty());
}

#[test]
fn the_corpus_shard_agrees_with_the_specs() {
    let Some(roots) = std::env::var_os("VIZE_DAVINCI_FACT_CORPUS") else {
        eprintln!("VIZE_DAVINCI_FACT_CORPUS unset: committed planes only");
        return;
    };
    let mut files = Vec::new();
    for root in roots.to_string_lossy().split(',') {
        let root = PathBuf::from(root);
        let root = if root.is_relative() && !root.is_dir() {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(&root)
        } else {
            root
        };
        assert!(
            root.is_dir(),
            "VIZE_DAVINCI_FACT_CORPUS must name directories: {}",
            root.display()
        );
        collect_vue_files(&root, &mut files);
    }
    assert!(!files.is_empty(), "corpus shard found no .vue files");
    let mut shard = Planes::default();
    for file in &files {
        let Ok(source) = std::fs::read_to_string(file) else {
            shard.bindings.skip("unreadable");
            shard.undefined.skip("unreadable");
            continue;
        };
        run_source(
            &vize_carton::cstr!("{}", file.display()),
            &source,
            &mut shard,
        );
    }
    eprintln!("{}", shard.scope_lines("corpus shard"));
    shard.assert_verdicts("corpus shard");
}

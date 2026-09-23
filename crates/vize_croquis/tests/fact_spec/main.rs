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
use vize_croquis::facts::spec::reactivity;

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
    assert_census(&battery, (9, 9, 87), (9, 9, 11), "battery");

    let mut ladder = Planes::default();
    for fixture in &davinci_harness::fixtures::LADDER {
        run_source(fixture.name, fixture.source, &mut ladder);
    }
    ladder.assert_verdicts("ladder");
    eprintln!("{}", ladder.scope_lines("ladder"));
    assert_census(&ladder, (6, 5, 80), (6, 6, 4), "ladder");

    let mut files = Vec::new();
    collect_vue_files(&matrix_dir(), &mut files).expect("read the P2-15 matrix");
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
    assert_census(&matrix, (90, 90, 0), (90, 90, 0), "matrix");
    assert!(matrix.bindings.divergences.is_empty() && matrix.undefined.divergences.is_empty());
    for (label, plane) in [
        ("battery", &battery.reactivity),
        ("ladder", &ladder.reactivity),
        ("matrix", &matrix.reactivity),
    ] {
        assert!(
            plane.divergences.is_empty(),
            "{label} reactivity diverged: {:?}",
            plane.divergences
        );
    }
    for (label, plane) in [
        ("battery", &battery.provide),
        ("ladder", &ladder.provide),
        ("matrix", &matrix.provide),
        ("battery-race", &battery.race),
        ("ladder-race", &ladder.race),
        ("matrix-race", &matrix.race),
    ] {
        assert!(
            plane.divergences.is_empty(),
            "{label} reactivity diverged: {:?}",
            plane.divergences
        );
    }
}

#[test]
fn the_lattice_join_matches_the_spec() {
    let matrix = reactivity::join_matrix();
    eprintln!("{}", matrix.scope_line("reactivity", "lattice matrix"));
    matrix
        .verdict("reactivity", "lattice matrix")
        .unwrap_or_else(|message| panic!("{message}"));
    reactivity::kind_table_agrees().unwrap_or_else(|message| panic!("{message}"));
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
        collect_vue_files(&root, &mut files)
            .unwrap_or_else(|error| panic!("corpus walk {}: {error}", root.display()));
    }
    assert!(!files.is_empty(), "corpus shard found no .vue files");
    let mut shard = Planes::default();
    for file in &files {
        let Ok(source) = std::fs::read_to_string(file) else {
            shard.bindings.skip("unreadable");
            shard.undefined.skip("unreadable");
            shard.reactivity.skip("unreadable");
            shard.provide.skip("unreadable");
            shard.race.skip("unreadable");
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
    shard
        .reactivity
        .verdict("reactivity", "corpus shard")
        .unwrap_or_else(|message| panic!("{message}"));
    assert!(
        shard.provide.divergences.is_empty(),
        "provide-inject corpus diverged: {:?}",
        shard.provide.divergences
    );
    // The committed shard's direct `provide()` / `inject()` calls sit outside
    // the tracker's top-level script-setup shape, so that half compares no
    // rows. `the_provide_inject_spec_agrees_on_a_drawn_sfc` is the comparison.
    shard
        .race
        .verdict("race-conditions", "corpus shard")
        .unwrap_or_else(|message| panic!("{message}"));
}

#[test]
fn the_provide_inject_spec_agrees_on_a_drawn_sfc() {
    let source = "\
<script setup>
const theme = ref('dark')
provide('theme', theme)
const color = inject('theme')
</script>
<template><p>{{ color }}</p></template>
";
    let mut planes = Planes::default();
    run_source("provide.vue", source, &mut planes);
    eprintln!(
        "{}",
        planes.provide.scope_line("provide-inject", "drawn sfc")
    );
    planes
        .provide
        .verdict("provide-inject", "drawn sfc")
        .unwrap_or_else(|message| panic!("{message}"));
}

/// Bindings census is build-independent. `UndefinedRefs` is compared only
/// when the debug trace exists; a release run still counts the artifacts
/// and compares nothing.
fn assert_census(
    planes: &Planes,
    bindings: (u64, u64, u64),
    undefined_debug: (u64, u64, u64),
    what: &str,
) {
    let undefined = if cfg!(debug_assertions) {
        undefined_debug
    } else {
        (undefined_debug.0, 0, 0)
    };
    assert_eq!(
        planes.census(),
        (bindings, undefined),
        "{what} census moved: re-pin deliberately (see the P4-3a record)"
    );
}

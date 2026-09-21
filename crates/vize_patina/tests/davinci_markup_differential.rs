//! Davinci P4-7a corpus-runnable entry — the TS-25 markup lane.
//!
//! Compiled only with the `davinci-differential` feature (`[[test]]
//! required-features` in Cargo.toml). Runs the committed battery with its
//! exact-pinned census (the same numbers the plain suite pins in
//! `markup::tests::differential_tests`, so a feature-wiring regression fails
//! loudly in both lanes), then, with `VIZE_DAVINCI_DIFFERENTIAL_CORPUS=<dir>`,
//! sweeps every `.vue` file under `<dir>`: each SFC's `<template>` block is
//! projected through Relief and through S1→S2 and the facade's full hook
//! traces must be identical. Every file compares or has no template; none may
//! diverge. The canonical fixture root fails closed unless its submodule
//! inventory reconciles; other roots sweep in smoke scope with
//! `closure_evidence=false` (see `davinci_test_support::corpus`).
//!
//! Run:
//!
//! ```text
//! VIZE_DAVINCI_DIFFERENTIAL_CORPUS=tests/_fixtures/_git \
//!     cargo test -p vize_patina --features davinci-differential \
//!     --test davinci_markup_differential -- --nocapture
//! ```

use std::fs;

use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_patina::markup::differential::{self, compare_template};

#[test]
fn markup_facade_observes_one_document() {
    // -- committed battery, exact-pinned census ------------------------
    let census = differential::run_battery();
    assert_eq!(
        census,
        differential::PINNED_BATTERY_CENSUS,
        "battery census moved: re-pin in both lanes deliberately"
    );

    // -- optional corpus sweep -----------------------------------------
    let Some(sweep) = davinci_test_support::corpus::resolve_env_sweep() else {
        eprintln!("VIZE_DAVINCI_DIFFERENTIAL_CORPUS unset: committed battery only");
        return;
    };
    let files = &sweep.files;
    assert!(
        !files.is_empty(),
        "corpus sweep found no .vue files under {}",
        sweep.root.display()
    );
    let mut compared = 0u64;
    let mut lines = 0u64;
    let mut without_template = 0u64;
    let mut unreadable = 0u64;
    for file in files {
        let Ok(source) = fs::read_to_string(file) else {
            unreadable += 1;
            continue;
        };
        let Ok(descriptor) = parse_sfc(&source, SfcParseOptions::default()) else {
            without_template += 1;
            continue;
        };
        let Some(template) = descriptor.template.as_ref() else {
            without_template += 1;
            continue;
        };
        match compare_template(&template.content) {
            Ok(count) => lines += count as u64,
            Err(divergence) => panic!(
                "{}: markup facade diverged at trace line {}\n  relief: {:?}\n  s2:     {:?}",
                file.display(),
                divergence.line,
                divergence.relief,
                divergence.s2
            ),
        }
        compared += 1;
    }
    assert!(compared > 0, "corpus sweep compared no template");
    eprintln!(
        "davinci markup differential corpus sweep: scope={} closure_evidence={} files={} \
         compared={} trace_lines={} without_template={} unreadable={}",
        sweep.scope_label(),
        sweep.closure_evidence(),
        files.len(),
        compared,
        lines,
        without_template,
        unreadable
    );
}

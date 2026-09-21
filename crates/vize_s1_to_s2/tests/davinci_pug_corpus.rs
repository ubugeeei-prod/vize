#![allow(clippy::disallowed_types)] // Fixture files arrive through `std::fs` as std strings.
//! Davinci P4-12c corpus-runnable entry for the pug dialect — the P2-8
//! lane shape (`davinci_lowering_corpus.rs`) for `<template lang="pug">`.
//!
//! Compiled only with the `davinci-differential` feature. Pins the
//! committed fixture census, then, with
//! `VIZE_DAVINCI_DIFFERENTIAL_CORPUS=<dir>`, sweeps every `.vue` file
//! under `<dir>` whose template is pug: the template body as the SFC
//! parser hands it over goes through the pug S1 parser (TS-19: the render
//! is the authored bytes) and through `lower_pug` (TS-20: total, the Vue
//! lowering's soundness oracle holds on the derived template, every
//! diagnostic inside the authored pug). The pinned-`pug` output oracle
//! over the same corpus is `tests/tooling/davinci-pug-oracle.test.ts`.
//!
//! ```text
//! VIZE_DAVINCI_DIFFERENTIAL_CORPUS=tests/_fixtures/_git \
//!     cargo test -p vize_s1_to_s2 --features davinci-differential \
//!     --test davinci_pug_corpus -- --nocapture
//! ```

mod support;

use std::path::PathBuf;

use support::assert_sound;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_s0::Allocator;
use vize_s1::pug::{check_pug_fidelity, parse_pug};
use vize_s1_to_s2::lower::pug::lower_pug;

fn fixture_count(dir: &str) -> usize {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/davinci-pug")
        .join(dir);
    std::fs::read_dir(root)
        .expect("fixture directory")
        .filter(|entry| {
            entry
                .as_ref()
                .is_ok_and(|entry| entry.path().extension().is_some_and(|ext| ext == "pug"))
        })
        .count()
}

/// TS-19 + TS-20 on one pug template body; `true` when it derives
/// without refusal.
fn check(content: &str, context: &str) -> bool {
    let allocator = Allocator::new();
    let (tree, errors) = parse_pug(&allocator, content);
    assert_eq!(
        check_pug_fidelity(&tree),
        Ok(()),
        "TS-19 fidelity: {context}"
    );
    let lowered = lower_pug(&allocator, &tree, &errors);
    for diagnostic in &lowered.diagnostics {
        assert!(
            diagnostic.span.end as usize <= content.len(),
            "{context}: {diagnostic:?}"
        );
    }
    assert_sound(lowered.html, context);
    !lowered.template.has_errors()
}

#[test]
fn pug_corpus_is_faithful_and_total() {
    assert_eq!(
        (fixture_count("matrix"), fixture_count("refused")),
        (26, 14)
    );
    let Some(sweep) = davinci_test_support::corpus::resolve_env_sweep() else {
        eprintln!("VIZE_DAVINCI_DIFFERENTIAL_CORPUS unset: committed fixtures only");
        return;
    };
    let (mut pug, mut derived) = (0u64, 0u64);
    for file in &sweep.files {
        let Ok(source) = std::fs::read_to_string(file) else {
            continue;
        };
        let Ok(descriptor) = parse_sfc(&source, SfcParseOptions::default()) else {
            continue;
        };
        let Some(template) = descriptor.template.as_ref() else {
            continue;
        };
        if template.src.is_some() || template.lang.as_deref() != Some("pug") {
            continue;
        }
        pug += 1;
        if check(&template.content, file.to_string_lossy().as_ref()) {
            derived += 1;
        }
    }
    eprintln!(
        "davinci pug corpus sweep: files={} pug={pug} derived={derived} refused={}",
        sweep.files.len(),
        pug - derived
    );
}

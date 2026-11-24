//! Retained-heap budget for type-aware preparation.
//!
//! Preparation keeps checker inputs until the project snapshot is published.
//! Marker-free modules must not keep a second copy of their source for that wait.
//!
//! Baseline before the retention change, one run, mimalloc counting allocator:
//! `script_len=537790 script_retained=546870 sfc_len=149880 sfc_retained=5942278`.

use davinci_harness::alloc::{CountingAllocator, stats};
use vize_patina::{LintPreset, Linter};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator::mimalloc();

fn plain_module(statements: usize) -> String {
    let mut source = String::from("export {}\n");
    for index in 0..statements {
        source.push_str("export const n");
        source.push_str(&index.to_string());
        source.push_str(" = ");
        source.push_str(&index.to_string());
        source.push_str("\n");
    }
    source
}

fn large_sfc(statements: usize) -> String {
    let mut source = String::from("<script setup lang=\"ts\">\nconst value: any = 1\n");
    for index in 0..statements {
        source.push_str("const n");
        source.push_str(&index.to_string());
        source.push_str(" = ");
        source.push_str(&index.to_string());
        source.push_str("\n");
    }
    source.push_str("</script>\n<template><div>{{ value }}</div></template>\n");
    source
}

#[test]
fn type_aware_prepare_retention() {
    let linter = Linter::with_preset(LintPreset::Opinionated).with_type_aware_lint(true);
    let script = plain_module(20_000);
    let sfc = large_sfc(8_000);
    let before_script = stats().live_bytes;
    let prepared_script = linter.prepare_script(&script, "mod.ts");
    let script_retained = stats().live_bytes.saturating_sub(before_script);
    let before_sfc = stats().live_bytes;
    let prepared_sfc = linter.prepare_sfc(&sfc, "Widget.vue");
    let sfc_retained = stats().live_bytes.saturating_sub(before_sfc);
    eprintln!(
        "budget script_len={script_len} script_retained={script_retained} sfc_len={sfc_len} sfc_retained={sfc_retained}",
        script_len = script.len(),
        sfc_len = sfc.len(),
    );
    // Marker-free script preparation has nothing to keep but the checker plan.
    // A second copy of the source would exceed the source itself.
    assert!(
        script_retained < script.len() as u64 / 4,
        "plain script retained {script_retained} bytes for a {} byte source",
        script.len()
    );
    // The virtual TypeScript module stays. The authored SFC text must not.
    // 5_942_278 was the retained size while that text was copied; the copy is
    // `sfc_len` bytes, so staying under 5_860_000 requires it to be gone.
    assert!(
        sfc_retained < 5_860_000,
        "sfc retained {sfc_retained} bytes for a {} byte source",
        sfc.len()
    );
    drop((prepared_script, prepared_sfc, script, sfc));
}

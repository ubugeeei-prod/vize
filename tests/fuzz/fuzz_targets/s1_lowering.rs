#![no_main]

// S1 → S2 lowering fuzz target (Davinci P2-8, TS-20).
//
// Drives the full Davinci template front half with arbitrary UTF-8:
// `vize_s1::parse` (the lossless S1 surface tree, total over
// malformed input by the typed-hole policy) followed by
// `vize_s1_to_s2::lower` (the total S1→S2 lowering — S2 ops or
// diagnostics, never a panic and never a partial-then-abandoned state).
//
// The target asserts more than no-crash, so a logic break surfaces as a
// fuzz finding: the page-order id accounting law (`op_count` equals the
// folio's op count), canonical S2 verifier invariants, lowering side-table id
// resolution, and the S2 folio round-trip law (the canonical `Full`-mode print
// re-parses to the same document) must hold on every input.
//
// The corpus is seeded from the `<template>` blocks of repository .vue
// fixtures by `tools/commands/ci/fuzz/seed_corpus.rs`.
use libfuzzer_sys::fuzz_target;
use vize_davinci::folio::{Folio, FolioMode};
use vize_davinci::side_table::SideTable;
use vize_s0::Allocator;
use vize_s1::parse;
use vize_s1_to_s2::lower;
use vize_s2::folio::DisegnoFolio;
use vize_s2::verify::{Rigor, verify, verify_table};

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    if u32::try_from(source.len()).is_err() {
        // S1 sources are u32-addressed by contract; libFuzzer's max_len
        // keeps real runs far below this.
        return;
    }
    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, source);
    let lowered = lower(&allocator, &tree, &errors);
    let folio = DisegnoFolio::of(&lowered.root.ops);
    assert_eq!(u64::from(lowered.op_count), folio.op_count());
    assert_eq!(verify(&folio, Rigor::Canonical), vec![]);
    assert_eq!(verify_table(&folio, &lowered.scopes), vec![]);
    assert_eq!(verify_table(&folio, &lowered.texts), vec![]);
    assert_eq!(verify_table(&folio, &lowered.wrappers), vec![]);
    assert_eq!(verify_table(&folio, &lowered.for_wrappers), vec![]);
    let provenance_ids: SideTable<()> = lowered
        .provenance
        .iter()
        .filter_map(|record| record.node.map(|node| (node, ())))
        .collect();
    assert_eq!(verify_table(&folio, &provenance_ids), vec![]);
    let printed = folio.print_to_string(FolioMode::Full);
    let reparsed = DisegnoFolio::parse(printed.as_str()).expect("canonical print must re-parse");
    assert_eq!(reparsed, folio);
});

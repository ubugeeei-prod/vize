#![no_main]

// Dump-parser fuzz target (Levels P2-8, TS-20).
//
// Drives the hand-written Levels dump parsers with arbitrary UTF-8
// under the invariant that *no input must panic*: parsers return
// `Result<_, DumpError>` for malformed pages, so a panic here is always
// a bug. Four parsers share the input — the L2 L2 page
// (`vize_l2`, path `levels/vize_l2`), the L3 L3 page, the croquis page,
// and the repro page (`vize_curator`).
//
// When an input does parse, the mode-explicit round-trip law is asserted
// on it: the canonical `Full`-mode print must re-parse to a document
// that prints identically (normalization by the first print).
//
// The corpus is seeded from the committed .dump fixtures by
// `tools/commands/ci/fuzz/seed-corpus.ts`.
use libfuzzer_sys::fuzz_target;
use vize_croquis::dump::Page as CroquisPage;
use vize_curator::repro::Page as ReproPage;
use vize_l0::dump::{Dump, Mode as DumpMode};
use vize_l2::dump::Page as L2Page;
use vize_l3::dump::Page as L3Page;

fn round_trip<F: Dump + PartialEq + core::fmt::Debug>(source: &str) {
    let Ok(parsed) = F::parse(source) else {
        return;
    };
    let printed = parsed.print_to_string(DumpMode::Full);
    let reparsed = F::parse(printed.as_str()).expect("canonical print must re-parse");
    assert_eq!(reparsed.print_to_string(DumpMode::Full), printed);
}

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    round_trip::<L2Page>(source);
    round_trip::<L3Page>(source);
    round_trip::<CroquisPage>(source);
    round_trip::<ReproPage>(source);
});

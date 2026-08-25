#![no_main]

// Folio-parser fuzz target (Davinci P2-8, TS-20).
//
// Drives the hand-written Davinci folio parsers with arbitrary UTF-8
// under the invariant that *no input must panic*: parsers return
// `Result<_, FolioError>` for malformed pages, so a panic here is always
// a bug. Three parsers share the input — the S2 Disegno page
// (`vize_s2`, package `vize_disegno`), the croquis page, and the repro page
// (`vize_davinci`).
//
// When an input does parse, the mode-explicit round-trip law is asserted
// on it: the canonical `Full`-mode print must re-parse to a document
// that prints identically (normalization by the first print).
//
// The corpus is seeded from the committed .folio fixtures by
// `tools/fuzz/seed_corpus.mjs`.
use libfuzzer_sys::fuzz_target;
use vize_davinci::folio::croquis::CroquisFolio;
use vize_davinci::folio::repro::ReproFolio;
use vize_davinci::folio::{Folio, FolioMode};
use vize_s2::folio::DisegnoFolio;

fn round_trip<F: Folio + PartialEq + core::fmt::Debug>(source: &str) {
    let Ok(parsed) = F::parse(source) else {
        return;
    };
    let printed = parsed.print_to_string(FolioMode::Full);
    let reparsed = F::parse(printed.as_str()).expect("canonical print must re-parse");
    assert_eq!(reparsed.print_to_string(FolioMode::Full), printed);
}

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    round_trip::<DisegnoFolio>(source);
    round_trip::<CroquisFolio>(source);
    round_trip::<ReproFolio>(source);
});

#![no_main]

// SFC parser fuzz target.
//
// Drives `vize_atelier_sfc::parse_sfc` with arbitrary UTF-8 input under the
// invariant that *no input must panic*. parse_sfc returns Result<_, SfcError>
// for malformed input, so a panic here is always a bug — either a missing
// bounds check, an arithmetic overflow, or an unwrap that assumed a shape the
// parser does not actually guarantee.
//
// The corpus is seeded from `tests/fixtures/**/*.vue` and `playground/**/*.vue`
// by `tools/fuzz/seed_corpus.mjs` so libFuzzer starts with a coverage map that
// reflects realistic SFC shapes (template + script + style + custom blocks).
use libfuzzer_sys::fuzz_target;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    let _ = parse_sfc(source, SfcParseOptions::default());
});

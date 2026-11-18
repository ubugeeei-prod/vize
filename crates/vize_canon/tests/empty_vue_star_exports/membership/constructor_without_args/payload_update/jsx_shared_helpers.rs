//! Invert the #13149 shared JSX declarations before older source custody.
use super::support;
use serde_json::json;
use std::fs;

#[derive(Clone, Copy)]
struct Identity {
    bytes: usize,
    sha256: &'static str,
}

const GLOBALS_PATH: &str = "npm/cli/virtual-ts-globals.d.ts";
const GLOBALS_BEFORE: Identity = Identity {
    bytes: 18_023,
    sha256: "3603093403379904949ed9c7c7ffd221595b2e1d8b3f7f10820ca748d7ed533d",
};
const GLOBALS_AFTER: Identity = Identity {
    bytes: 21_217,
    sha256: "d2cc4d1e40b40dac7e6362a3532ba92e9cefc4d7272a345f73f096919aba390e",
};
const GLOBALS_START: &str =
    "  // Keep this ABI block line-aligned with the canonical Rust declaration assets.\n";
const GLOBALS_END: &str = "  type __RuntimePropValue";

fn has_identity(text: &str, expected: Identity) -> bool {
    text.len() == expected.bytes && support::hash(text.as_bytes()) == expected.sha256
}

fn authenticate(text: &str, expected: Identity) {
    assert_eq!(json!(text.len()), expected.bytes);
    assert_eq!(support::hash(text.as_bytes()), expected.sha256);
}

fn restore_source(path: &str, current: &str) -> String {
    if path != GLOBALS_PATH || !has_identity(current, GLOBALS_AFTER) {
        return current.to_owned();
    }
    assert_eq!(current.matches(GLOBALS_START).count(), 1);
    assert_eq!(current.matches(GLOBALS_END).count(), 1);
    let start = current.find(GLOBALS_START).unwrap();
    let end = current.find(GLOBALS_END).unwrap();
    let previous = format!("{}{}", &current[..start], &current[end..]);
    authenticate(&previous, GLOBALS_BEFORE);
    previous
}

pub(super) fn source_before(path: &str, current: String) -> String {
    restore_source(path, &current)
}

pub(super) fn preamble_before(current: &str) -> String {
    current.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_sources_and_preambles_restore_the_exact_predecessors() {
        let current = fs::read_to_string(support::repository().join(GLOBALS_PATH)).unwrap();
        let previous = source_before(GLOBALS_PATH, current);
        authenticate(&previous, GLOBALS_BEFORE);
        assert_eq!(source_before(GLOBALS_PATH, previous.clone()), previous);
        for current in [
            vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
            vize_canon::virtual_ts::SHARED_PREAMBLE_STRICT_DTS,
        ] {
            let previous = preamble_before(current);
            assert_eq!(previous, current);
            assert_eq!(preamble_before(&previous), previous);
        }
    }
}

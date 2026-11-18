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
const PREAMBLE_PATH: &str = "crates/vize_canon/src/virtual_ts/helpers/preamble.rs";
const GLOBALS_BEFORE: Identity = Identity {
    bytes: 18_023,
    sha256: "3603093403379904949ed9c7c7ffd221595b2e1d8b3f7f10820ca748d7ed533d",
};
const GLOBALS_AFTER: Identity = Identity {
    bytes: 21_217,
    sha256: "d2cc4d1e40b40dac7e6362a3532ba92e9cefc4d7272a345f73f096919aba390e",
};
const PREAMBLE_BEFORE: Identity = Identity {
    bytes: 6_130,
    sha256: "6594462c0c1e163fde882056a484c0d5edc9e65d45d5f65494c2319975e2e62d",
};
const PREAMBLE_AFTER: Identity = Identity {
    bytes: 6_259,
    sha256: "8b6e4b50a2e8b0294d6ef1e5d7699daef0dcdd9915df72f6632fdb54e90af4a6",
};
const LOOSE_BEFORE: Identity = Identity {
    bytes: 58_391,
    sha256: "3c64c8f55d8c6bbeb5e39c4fae676c36a8546245b5895ad8df09af9123b412f4",
};
const LOOSE_AFTER: Identity = Identity {
    bytes: 61_195,
    sha256: "5812d3aeca8fa8427d8305d704b62d00699c2f076898a715fb97821bad2c6821",
};
const STRICT_BEFORE: Identity = Identity {
    bytes: 58_182,
    sha256: "bb0d1e6d191f1acbbbd6ed4503569e5f622628f4588e57aa9f11ccf7e6058ef5",
};
const STRICT_AFTER: Identity = Identity {
    bytes: 60_986,
    sha256: "d9c1311bdc0fd8ed4cc1f6a002e0946199e241b19c8f8beba3247701260330f9",
};
const GLOBALS_START: &str =
    "  // Keep this ABI block line-aligned with the canonical Rust declaration assets.\n";
const GLOBALS_END: &str = "  type __RuntimePropValue";
const PREAMBLE_INCLUDES: &str = "            include_str!(\"declarations/jsx-context.d.ts.txt\"),\n            include_str!(\"declarations/jsx-components.d.ts.txt\"),\n";
const CONTEXT: &str = "type Ctx<Emits = {}, Slots = {}> = { emit: __EmitFn<Emits>; slots: Slots; attrs: Record<string, unknown>; };\n";

fn has_identity(text: &str, expected: Identity) -> bool {
    text.len() == expected.bytes && support::hash(text.as_bytes()) == expected.sha256
}

fn authenticate(text: &str, expected: Identity) {
    assert_eq!(json!(text.len()), expected.bytes);
    assert_eq!(support::hash(text.as_bytes()), expected.sha256);
}

fn restore_source(path: &str, current: &str) -> String {
    let (before, after) = match path {
        GLOBALS_PATH => (GLOBALS_BEFORE, GLOBALS_AFTER),
        PREAMBLE_PATH => (PREAMBLE_BEFORE, PREAMBLE_AFTER),
        _ => return current.to_owned(),
    };
    if !has_identity(current, after) {
        return current.to_owned();
    }
    let previous = if path == GLOBALS_PATH {
        assert_eq!(current.matches(GLOBALS_START).count(), 1);
        assert_eq!(current.matches(GLOBALS_END).count(), 1);
        let start = current.find(GLOBALS_START).unwrap();
        let end = current.find(GLOBALS_END).unwrap();
        format!("{}{}", &current[..start], &current[end..])
    } else {
        assert_eq!(current.matches(PREAMBLE_INCLUDES).count(), 1);
        current.replacen(PREAMBLE_INCLUDES, "", 1)
    };
    authenticate(&previous, before);
    previous
}

pub(super) fn source_before(path: &str, current: String) -> String {
    restore_source(path, &current)
}

pub(super) fn preamble_before(current: &str) -> String {
    let (before, after) = if has_identity(current, LOOSE_AFTER) {
        (LOOSE_BEFORE, LOOSE_AFTER)
    } else if has_identity(current, STRICT_AFTER) {
        (STRICT_BEFORE, STRICT_AFTER)
    } else {
        return current.to_owned();
    };
    authenticate(current, after);
    let block = format!("{CONTEXT}{}", vize_canon::virtual_ts::JSX_COMPONENT_HELPER);
    assert_eq!(current.matches(block.as_str()).count(), 1);
    let previous = current.replacen(block.as_str(), "", 1);
    authenticate(&previous, before);
    previous
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_sources_and_preambles_restore_the_exact_predecessors() {
        for (path, expected) in [
            (GLOBALS_PATH, GLOBALS_BEFORE),
            (PREAMBLE_PATH, PREAMBLE_BEFORE),
        ] {
            let current = fs::read_to_string(support::repository().join(path)).unwrap();
            let previous = source_before(path, current);
            authenticate(&previous, expected);
            assert_eq!(source_before(path, previous.clone()), previous);
        }
        for (current, expected) in [
            (vize_canon::virtual_ts::SHARED_PREAMBLE_DTS, LOOSE_BEFORE),
            (
                vize_canon::virtual_ts::SHARED_PREAMBLE_STRICT_DTS,
                STRICT_BEFORE,
            ),
        ] {
            let previous = preamble_before(current);
            authenticate(&previous, expected);
            assert_eq!(preamble_before(&previous), previous);
        }
    }
}

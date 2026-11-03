//! Invert the untyped constructor emit-props input before older public layers.
use super::support;
use serde_json::{Value, json};

const BEFORE: &str = ") => <A>(props: $EmitInput<P, $EmitFallthrough<F>, O, A>) => __EmitProps<E>;";
const AFTER: &str = ") => (props: unknown) => __EmitProps<E>;";
const DELTA: usize = 36;
/// (path, bytes before, sha256 before, sha256 after)
const FILES: [(&str, usize, &str, &str); 5] = [
    (
        "npm/cli/virtual-ts.d.ts",
        15334,
        "d5e5eee803afd4fdb74912647d453322d36ce476daef487012eb1eca25f750df",
        "302fa9861b60876fc4e5f1b72fbdfd3b167ea05f55aadc8b292c766559ba062a",
    ),
    (
        "npm/cli/virtual-ts.d.mts",
        15334,
        "d5e5eee803afd4fdb74912647d453322d36ce476daef487012eb1eca25f750df",
        "302fa9861b60876fc4e5f1b72fbdfd3b167ea05f55aadc8b292c766559ba062a",
    ),
    (
        "npm/cli/virtual-ts.d.cts",
        15334,
        "d5e5eee803afd4fdb74912647d453322d36ce476daef487012eb1eca25f750df",
        "302fa9861b60876fc4e5f1b72fbdfd3b167ea05f55aadc8b292c766559ba062a",
    ),
    (
        "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-emit-inference.d.ts.txt",
        1994,
        "6b9d378db61cd26288884255b4cb0ac24812a9213ed9102562e04f677b9764f4",
        "211a32eba730c03816340ceffdfb1c5522ce64307f3634fadc16f9a76ad882f8",
    ),
    (
        "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-inference.d.ts.txt",
        1931,
        "009ed9b8b67e500f7b7c0186dc2f2c7afecd1dc5dec1a08aca8da964b642880b",
        "0871df10b82628de68011721d106330bf64fab2840480113fa6637bf69cea9be",
    ),
];
const PREAMBLE_BEFORE_BYTES: usize = 44_244;
const PREAMBLE_BEFORE_SHA: &str =
    "298f0dd19a3114ee2fc55f67b598a5a3b1649c356b4f453be52312b854e65816";
const PREAMBLE_AFTER_SHA: &str = "183be97e44465daea8aee94edafc9c97e131b76a52c21ff0c0f9d143aaaf483b";

fn previous(current: &str, before_bytes: usize, before_sha: &str, after_sha: &str) -> String {
    assert_eq!(current.len() + DELTA, before_bytes);
    assert_eq!(support::hash(current.as_bytes()), after_sha);
    assert_eq!(current.matches(AFTER).count(), 1);
    assert_eq!(current.matches(BEFORE).count(), 0);
    let previous = current.replacen(AFTER, BEFORE, 1);
    assert_eq!(previous.len(), before_bytes);
    assert_eq!(support::hash(previous.as_bytes()), before_sha);
    previous
}

pub(super) fn source_before(path: &str, current: String) -> String {
    match FILES.iter().find(|(file, ..)| *file == path) {
        Some((_, bytes, before, after)) => previous(&current, *bytes, before, after),
        None => current,
    }
}

pub(super) fn preamble_before(current: &str) -> String {
    previous(
        current,
        PREAMBLE_BEFORE_BYTES,
        PREAMBLE_BEFORE_SHA,
        PREAMBLE_AFTER_SHA,
    )
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["currentUntypedEmitPropsInputUpdate"] = json!({
        "kind": "exact-current-untyped-emit-props-input-inverse",
        "nativeAcceptanceCredit": false,
        "deltaBytes": DELTA,
        "files": FILES.iter().map(|(path, bytes, before, after)| json!({
            "path": path, "beforeBytes": bytes, "afterBytes": bytes - DELTA,
            "beforeSha256": before, "afterSha256": after,
        })).collect::<Vec<_>>(),
        "compiledPreamble": { "path": "vize_canon::virtual_ts::SHARED_PREAMBLE_DTS",
            "beforeBytes": PREAMBLE_BEFORE_BYTES, "afterBytes": PREAMBLE_BEFORE_BYTES - DELTA,
            "beforeSha256": PREAMBLE_BEFORE_SHA, "afterSha256": PREAMBLE_AFTER_SHA }
    });
}

#[cfg(test)]
mod tests {
    use super::{AFTER, BEFORE, FILES, preamble_before, source_before};
    use std::fs;

    #[test]
    fn current_sources_and_preamble_reconstruct_only_the_typed_input() {
        for (path, ..) in FILES {
            let current = fs::read_to_string(super::support::repository().join(path)).unwrap();
            let prior = source_before(path, current.clone());
            assert_eq!(prior.replacen(BEFORE, AFTER, 1), current);
            for mutated in [
                current.replacen(AFTER, "", 1),
                current.replacen(AFTER, BEFORE, 1),
                format!("{current}\n"),
            ] {
                assert!(std::panic::catch_unwind(|| source_before(path, mutated.clone())).is_err());
            }
        }
        let preamble = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
        assert_eq!(
            preamble_before(preamble).replacen(BEFORE, AFTER, 1),
            preamble
        );
        let other = "npm/cli/virtual-ts-globals.d.ts";
        assert_eq!(source_before(other, String::from("x")), "x");
    }
}

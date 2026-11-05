//! Invert only the all-optional `$EmitRequiredFields` short-circuit before the emit-props twin keys.
use super::support;
use serde_json::{Value, json};

const FIELDS_BEFORE: &str =
    "type $EmitRequiredFields<T> = T extends unknown ? Pick<T, $EmitRequiredKeys<T>> : never;";
const FIELDS_AFTER: &str = "type $EmitRequiredFields<T> = T extends unknown ? {} extends T ? Pick<T, never> : Pick<T, $EmitRequiredKeys<T>> : never;";
const PUBLIC_FIELDS_BEFORE: &str =
    "type $EmitRequiredFields<T> = T extends unknown ? Pick<T, $EmitRequiredKeys<T>> : never;\n";
const PUBLIC_FIELDS_AFTER: &str = "type $EmitRequiredFields<T> = T extends unknown\n  ? {} extends T\n    ? Pick<T, never>\n    : Pick<T, $EmitRequiredKeys<T>>\n  : never;\n";
const PUBLIC_EDITS: &[(&str, &str)] = &[(PUBLIC_FIELDS_BEFORE, PUBLIC_FIELDS_AFTER)];
const PUBLIC_BEFORE_SHA: &str = "96262fd9a40041ca76881234ec4705777474d6e6b319369d0205191312845b7f";
const PUBLIC_AFTER_SHA: &str = "363aa2912d43f63c940a6af5f0eae6bc75dabac8cd97752fda1031b84c409f5e";
const PREAMBLE_EDITS: &[(&str, &str)] = &[(FIELDS_BEFORE, FIELDS_AFTER)];
const PREAMBLE_BEFORE_BYTES: usize = 44_431;
const PREAMBLE_BEFORE_SHA: &str =
    "85e25abaee2c56205b506b1c87dd9e5305848d8840f0f13342ff8635517aae02";
const PREAMBLE_AFTER_SHA: &str = "a7ed593d2476dd4240c79e4101d499d094a81c57ade7d812eb1628b135528b15";

struct File {
    path: &'static str,
    edits: &'static [(&'static str, &'static str)],
    before_bytes: usize,
    before_sha: &'static str,
    after_sha: &'static str,
}

const FILES: [File; 5] = [
    File {
        path: "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-inference.d.ts.txt",
        edits: &[(FIELDS_BEFORE, FIELDS_AFTER)],
        before_bytes: 1931,
        before_sha: "009ed9b8b67e500f7b7c0186dc2f2c7afecd1dc5dec1a08aca8da964b642880b",
        after_sha: "7d767ce98c77aeb1cac0f500fce09498df493479bce7626c0b2d92cc7fc8bebe",
    },
    File {
        path: "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-emit-inference.d.ts.txt",
        edits: &[(FIELDS_BEFORE, FIELDS_AFTER)],
        before_bytes: 1994,
        before_sha: "6b9d378db61cd26288884255b4cb0ac24812a9213ed9102562e04f677b9764f4",
        after_sha: "72d2601a671bc22668ceb1d14470a1b4be16864d5d9f5d637fbab3f0132c8a5c",
    },
    File {
        path: "npm/cli/virtual-ts.d.ts",
        edits: PUBLIC_EDITS,
        before_bytes: 15_505,
        before_sha: PUBLIC_BEFORE_SHA,
        after_sha: PUBLIC_AFTER_SHA,
    },
    File {
        path: "npm/cli/virtual-ts.d.mts",
        edits: PUBLIC_EDITS,
        before_bytes: 15_505,
        before_sha: PUBLIC_BEFORE_SHA,
        after_sha: PUBLIC_AFTER_SHA,
    },
    File {
        path: "npm/cli/virtual-ts.d.cts",
        edits: PUBLIC_EDITS,
        before_bytes: 15_505,
        before_sha: PUBLIC_BEFORE_SHA,
        after_sha: PUBLIC_AFTER_SHA,
    },
];

fn delta(edits: &[(&str, &str)]) -> usize {
    edits
        .iter()
        .map(|(before, after)| after.len() - before.len())
        .sum()
}

fn invert(
    current: &str,
    edits: &[(&str, &str)],
    before_bytes: usize,
    before_sha: &str,
    after_sha: &str,
) -> String {
    assert_eq!(current.len(), before_bytes + delta(edits));
    assert_eq!(support::hash(current.as_bytes()), after_sha);
    let mut previous = current.to_owned();
    for (before, after) in edits {
        assert_eq!(previous.matches(after).count(), 1);
        assert_eq!(previous.matches(before).count(), 0);
        previous = previous.replacen(after, before, 1);
    }
    assert_eq!(previous.len(), before_bytes);
    assert_eq!(support::hash(previous.as_bytes()), before_sha);
    previous
}

pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::extra_keys_policy::source_before(path, current);
    match FILES.iter().find(|file| file.path == path) {
        Some(file) => invert(
            &current,
            file.edits,
            file.before_bytes,
            file.before_sha,
            file.after_sha,
        ),
        None => current,
    }
}

pub(super) fn preamble_before(current: &str) -> String {
    invert(
        &super::extra_keys_policy::preamble_before(current),
        PREAMBLE_EDITS,
        PREAMBLE_BEFORE_BYTES,
        PREAMBLE_BEFORE_SHA,
        PREAMBLE_AFTER_SHA,
    )
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    let files = FILES
        .iter()
        .map(|file| {
            json!({ "path": file.path, "beforeBytes": file.before_bytes,
                "afterBytes": file.before_bytes + delta(file.edits),
                "beforeSha256": file.before_sha, "afterSha256": file.after_sha })
        })
        .collect::<Vec<_>>();
    manifest["provenance"]["currentEmitRequiredFieldsUpdate"] = json!({
        "kind": "exact-current-all-optional-emit-required-fields-inverse",
        "nativeAcceptanceCredit": false,
        "files": files,
        "compiledPreamble": { "path": "vize_canon::virtual_ts::SHARED_PREAMBLE_DTS",
            "beforeBytes": PREAMBLE_BEFORE_BYTES,
            "afterBytes": PREAMBLE_BEFORE_BYTES + delta(PREAMBLE_EDITS),
            "beforeSha256": PREAMBLE_BEFORE_SHA, "afterSha256": PREAMBLE_AFTER_SHA }
    });
}

#[cfg(test)]
mod tests {
    use super::{FILES, PREAMBLE_EDITS, preamble_before, retain_provenance, source_before};
    use serde_json::json;

    fn current(path: &str) -> String {
        std::fs::read_to_string(super::support::repository().join(path)).unwrap()
    }

    fn redo(mut previous: String, edits: &[(&str, &str)]) -> String {
        for (before, after) in edits.iter().rev() {
            previous = previous.replacen(before, after, 1);
        }
        previous
    }

    #[test]
    fn current_assets_preamble_and_sdk_reconstruct_only_the_required_fields_layer() {
        for file in &FILES {
            let body = current(file.path);
            assert_eq!(
                redo(source_before(file.path, body.clone()), file.edits),
                super::super::extra_keys_policy::source_before(file.path, body)
            );
        }
        let preamble = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
        assert_eq!(
            redo(preamble_before(preamble), PREAMBLE_EDITS),
            super::super::extra_keys_policy::preamble_before(preamble)
        );
        let other = "crates/vize_canon/src/virtual_ts/helpers/declarations/vue3-types.d.ts.txt";
        assert_eq!(source_before(other, current(other)), current(other));
        let mut manifest = json!({ "provenance": { "existing": "unchanged" } });
        retain_provenance(&mut manifest);
        assert_eq!(manifest["provenance"]["existing"], "unchanged");
    }

    #[test]
    fn missing_duplicate_stale_and_extra_bytes_refuse() {
        for file in &FILES {
            let body = current(file.path);
            for (before, after) in file.edits {
                for mutated in [
                    body.replacen(after, "", 1),
                    format!("{body}{after}"),
                    body.replacen(after, before, 1),
                    format!("{body}\n"),
                ] {
                    assert!(
                        std::panic::catch_unwind(|| source_before(file.path, mutated.clone()))
                            .is_err()
                    );
                }
            }
        }
    }
}

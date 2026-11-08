//! Invert the #11497 removal of the one-prop boolean-attribute exception.
use super::support;
use serde_json::{Value, json};

const CANONICAL_BEFORE: &str = "type $ExtraInputKeys<T, Declared, Authored, Allowed, K extends keyof Authored = Exclude<keyof Authored, keyof Allowed>> = K extends unknown ? $RelaxedDynamicUnionExtra<T, Declared, Authored, K> extends true ? never : K : never;";
const CANONICAL_AFTER: &str = "type $ExtraInputKeys<_T, _Declared, Authored, Allowed> = Exclude<keyof Authored, keyof Allowed>;";
const GENERATED_BEFORE: &str = "type $ExtraInputKeys<\n  T,\n  Declared,\n  Authored,\n  Allowed,\n  K extends keyof Authored = Exclude<keyof Authored, keyof Allowed>,\n> = K extends unknown\n  ? $RelaxedDynamicUnionExtra<T, Declared, Authored, K> extends true\n    ? never\n    : K\n  : never;";
const GENERATED_AFTER: &str = "type $ExtraInputKeys<_T, _Declared, Authored, Allowed> = Exclude<keyof Authored, keyof Allowed>;";
const GLOBALS_BEFORE: &str = "  type $ExtraInputKeys<\n    T,\n    Declared,\n    Authored,\n    Allowed,\n    K extends keyof Authored = Exclude<keyof Authored, keyof Allowed>,\n  > = K extends unknown\n    ? $RelaxedDynamicUnionExtra<T, Declared, Authored, K> extends true\n      ? never\n      : K\n    : never;";
const GLOBALS_AFTER: &str = "  type $ExtraInputKeys<_T, _Declared, Authored, Allowed> = Exclude<keyof Authored, keyof Allowed>;";

struct File {
    path: &'static str,
    before: &'static str,
    after: &'static str,
    before_bytes: usize,
    before_sha: &'static str,
    after_sha: &'static str,
}

const FILES: [File; 3] = [
    File {
        path: "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt",
        before: CANONICAL_BEFORE,
        after: CANONICAL_AFTER,
        before_bytes: 5_870,
        before_sha: "3a3c9fb284feba5cdd7fc639ae1b99d19c31e5cb344339968acd553734adff53",
        after_sha: "3eca9ffe508f656bf244960c7c9a2da73993dc073e6a5262778fbc4e99c10c6d",
    },
    File {
        path: "npm/cli/generated/content-mapper-types.d.ts",
        before: GENERATED_BEFORE,
        after: GENERATED_AFTER,
        before_bytes: 12_116,
        before_sha: "c62f373226b2967df18826a699d7116605020149093885a4bb0bcfed41cf725c",
        after_sha: "0e6d163880471a51191587140ab5df0467247a253c6a951e65906737368a90aa",
    },
    File {
        path: "npm/cli/virtual-ts-globals.d.ts",
        before: GLOBALS_BEFORE,
        after: GLOBALS_AFTER,
        before_bytes: 17_176,
        before_sha: "a92231c9b924778d38ba10c25d7908d1f3a70e1516aad96f8a653fe46e11977b",
        after_sha: "f9e13dfcbbb3e719055570a6b13128720b01a29c2ffe4044a8d16613acc3b4d2",
    },
];

const PREAMBLE_BEFORE_BYTES: usize = 47_268;
const PREAMBLE_BEFORE_SHA: &str =
    "be971e511d48ce689262c5c6f422ff2685085bf166a0c93f9c0a9c93b9848ba3";
const PREAMBLE_AFTER_SHA: &str = "15b8c986ecc0b3eba8bba2ea7797793564cce0600b4f1e0e575e5572e0f133d5";

fn invert(
    current: &str,
    before: &str,
    after: &str,
    before_bytes: usize,
    before_sha: &str,
    after_sha: &str,
) -> String {
    assert_eq!(
        current.len() as isize,
        before_bytes as isize + after.len() as isize - before.len() as isize
    );
    assert_eq!(support::hash(current.as_bytes()), after_sha);
    assert_eq!(current.matches(after).count(), 1);
    assert_eq!(current.matches(before).count(), 0);
    let previous = current.replacen(after, before, 1);
    assert_eq!(previous.len(), before_bytes);
    assert_eq!(support::hash(previous.as_bytes()), before_sha);
    previous
}

pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::canon_leftovers::source_before(path, current);
    let Some(file) = FILES.iter().find(|file| file.path == path) else {
        return current;
    };
    invert(
        &current,
        file.before,
        file.after,
        file.before_bytes,
        file.before_sha,
        file.after_sha,
    )
}

pub(super) fn preamble_before(current: &str) -> String {
    let current = super::canon_leftovers::preamble_before(current);
    invert(
        &current,
        CANONICAL_BEFORE,
        CANONICAL_AFTER,
        PREAMBLE_BEFORE_BYTES,
        PREAMBLE_BEFORE_SHA,
        PREAMBLE_AFTER_SHA,
    )
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["strictExtraKeysParity11497"] = json!({
        "kind": "exact-current-extra-key-special-case-inverse",
        "issue": 11497,
        "nativeAcceptanceCredit": false,
        "files": FILES.iter().map(|file| json!({
            "path": file.path,
            "beforeBytes": file.before_bytes,
            "afterBytes": file.before_bytes as isize + file.after.len() as isize - file.before.len() as isize,
            "beforeSha256": file.before_sha,
            "afterSha256": file.after_sha,
        })).collect::<Vec<_>>(),
        "compiledPreamble": {
            "path": "vize_canon::virtual_ts::SHARED_PREAMBLE_DTS",
            "beforeBytes": PREAMBLE_BEFORE_BYTES,
            "afterBytes": PREAMBLE_BEFORE_BYTES as isize + CANONICAL_AFTER.len() as isize - CANONICAL_BEFORE.len() as isize,
            "beforeSha256": PREAMBLE_BEFORE_SHA,
            "afterSha256": PREAMBLE_AFTER_SHA,
        },
    });
}

#[cfg(test)]
mod tests {
    use super::{FILES, preamble_before, retain_provenance, source_before};
    use serde_json::json;

    fn current(path: &str) -> String {
        let current = std::fs::read_to_string(super::support::repository().join(path)).unwrap();
        super::super::canon_leftovers::source_before(path, current)
    }

    #[test]
    fn current_assets_and_preamble_reconstruct_only_the_strict_extra_key_layer() {
        for file in &FILES {
            let body = current(file.path);
            let previous = source_before(file.path, body.clone());
            assert_eq!(previous.replacen(file.before, file.after, 1), body);
        }
        let preamble = super::super::canon_leftovers::preamble_before(
            vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
        );
        assert_eq!(
            preamble_before(&preamble).replacen(super::CANONICAL_BEFORE, super::CANONICAL_AFTER, 1),
            preamble
        );
        let other = "crates/vize_canon/src/virtual_ts/helpers/declarations/vue3-types.d.ts.txt";
        assert_eq!(source_before(other, current(other)), current(other));
        let mut manifest = json!({ "provenance": { "existing": "unchanged" } });
        retain_provenance(&mut manifest);
        assert_eq!(manifest["provenance"]["existing"], "unchanged");
    }

    #[test]
    fn missing_duplicate_stale_and_extra_bytes_are_refused() {
        for file in &FILES {
            let body = current(file.path);
            for changed in [
                body.replacen(file.after, "", 1),
                format!("{body}{}", file.after),
                body.replacen(file.after, file.before, 1),
                format!("{body}\n"),
            ] {
                assert!(
                    std::panic::catch_unwind(|| source_before(file.path, changed.clone())).is_err()
                );
            }
        }
    }
}

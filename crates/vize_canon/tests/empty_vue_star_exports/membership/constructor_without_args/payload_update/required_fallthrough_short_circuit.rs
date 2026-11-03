//! Invert only the all-optional `$RequiredFallthroughProps` key default before the `$EmitRequiredFields` layer.
use super::support;
use serde_json::{Value, json};

const CANONICAL_BEFORE: &str =
    "$RequiredFallthroughProps<C, __F = $FallthroughProps<C>, __K = $RequiredKeys<__F>> = [";
const CANONICAL_AFTER: &str = "$RequiredFallthroughProps<C, __F = $FallthroughProps<C>, __K = [__F extends unknown ? ({} extends __F ? never : __F) : never] extends [never] ? never : $RequiredKeys<__F>> = [";
const PUBLIC_BEFORE: &str = "export type $RequiredFallthroughProps<C, __F = $FallthroughProps<C>, __K = $RequiredKeys<__F>> = [\n  __K,\n] extends [never]\n  ? {}\n  : { [K in keyof __F as K extends __K ? K : never]: __F[K] };\n";
const PUBLIC_AFTER: &str = "export type $RequiredFallthroughProps<\n  C,\n  __F = $FallthroughProps<C>,\n  __K = [__F extends unknown ? ({} extends __F ? never : __F) : never] extends [never]\n    ? never\n    : $RequiredKeys<__F>,\n> = [__K] extends [never] ? {} : { [K in keyof __F as K extends __K ? K : never]: __F[K] };\n";
const PUBLIC_BEFORE_SHA: &str = "363aa2912d43f63c940a6af5f0eae6bc75dabac8cd97752fda1031b84c409f5e";
const PUBLIC_AFTER_SHA: &str = "1e7fdf1189dccffc190bccffe24eadf080687c08732c06c342ea8e109e3fb026";
const PREAMBLE_BEFORE_BYTES: usize = 44_463;
const PREAMBLE_BEFORE_SHA: &str =
    "a7ed593d2476dd4240c79e4101d499d094a81c57ade7d812eb1628b135528b15";
const PREAMBLE_AFTER_SHA: &str = "ce459f5905f40d8b20df142316540cd89ae2df5f5c263c91b055f5f6227ab5e4";

struct File {
    path: &'static str,
    before: &'static str,
    after: &'static str,
    before_bytes: usize,
    before_sha: &'static str,
    after_sha: &'static str,
}

const FILES: [File; 4] = [
    File {
        path: "crates/vize_canon/src/virtual_ts/helpers/declarations/component-checks.d.ts.txt",
        before: CANONICAL_BEFORE,
        after: CANONICAL_AFTER,
        before_bytes: 8324,
        before_sha: "8b78a238856c7599da7d38bd9f1f354d9394481282230845c5c52ba9056cb133",
        after_sha: "62de3916267cc0b3c49156313a774b28c5d50bcc455b120a4ba8f4a423b9cadb",
    },
    File {
        path: "npm/cli/virtual-ts.d.ts",
        before: PUBLIC_BEFORE,
        after: PUBLIC_AFTER,
        before_bytes: 15_549,
        before_sha: PUBLIC_BEFORE_SHA,
        after_sha: PUBLIC_AFTER_SHA,
    },
    File {
        path: "npm/cli/virtual-ts.d.mts",
        before: PUBLIC_BEFORE,
        after: PUBLIC_AFTER,
        before_bytes: 15_549,
        before_sha: PUBLIC_BEFORE_SHA,
        after_sha: PUBLIC_AFTER_SHA,
    },
    File {
        path: "npm/cli/virtual-ts.d.cts",
        before: PUBLIC_BEFORE,
        after: PUBLIC_AFTER,
        before_bytes: 15_549,
        before_sha: PUBLIC_BEFORE_SHA,
        after_sha: PUBLIC_AFTER_SHA,
    },
];

fn invert(
    current: &str,
    (before, after): (&str, &str),
    before_bytes: usize,
    before_sha: &str,
    after_sha: &str,
) -> String {
    assert_eq!(current.len(), before_bytes + after.len() - before.len());
    assert_eq!(support::hash(current.as_bytes()), after_sha);
    assert_eq!(current.matches(after).count(), 1);
    assert_eq!(current.matches(before).count(), 0);
    let previous = current.replacen(after, before, 1);
    assert_eq!(previous.len(), before_bytes);
    assert_eq!(support::hash(previous.as_bytes()), before_sha);
    previous
}

pub(super) fn source_before(path: &str, current: String) -> String {
    match FILES.iter().find(|file| file.path == path) {
        Some(file) => invert(
            &current,
            (file.before, file.after),
            file.before_bytes,
            file.before_sha,
            file.after_sha,
        ),
        None => current,
    }
}

pub(super) fn preamble_before(current: &str) -> String {
    invert(
        current,
        (CANONICAL_BEFORE, CANONICAL_AFTER),
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
                "afterBytes": file.before_bytes + file.after.len() - file.before.len(),
                "beforeSha256": file.before_sha, "afterSha256": file.after_sha })
        })
        .collect::<Vec<_>>();
    manifest["provenance"]["currentRequiredFallthroughKeysUpdate"] = json!({
        "kind": "exact-current-all-optional-required-fallthrough-keys-inverse",
        "nativeAcceptanceCredit": false,
        "files": files,
        "compiledPreamble": { "path": "vize_canon::virtual_ts::SHARED_PREAMBLE_DTS",
            "beforeBytes": PREAMBLE_BEFORE_BYTES,
            "afterBytes": PREAMBLE_BEFORE_BYTES + CANONICAL_AFTER.len() - CANONICAL_BEFORE.len(),
            "beforeSha256": PREAMBLE_BEFORE_SHA, "afterSha256": PREAMBLE_AFTER_SHA }
    });
}

#[cfg(test)]
mod tests {
    use super::{FILES, preamble_before, retain_provenance, source_before};
    use serde_json::json;

    fn current(path: &str) -> String {
        std::fs::read_to_string(super::support::repository().join(path)).unwrap()
    }

    #[test]
    fn current_assets_preamble_and_sdk_reconstruct_only_the_required_fallthrough_layer() {
        for file in &FILES {
            let body = current(file.path);
            let prior = source_before(file.path, body.clone());
            assert_eq!(prior.replacen(file.before, file.after, 1), body);
        }
        let preamble = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
        assert_eq!(
            preamble_before(preamble).replacen(FILES[0].before, FILES[0].after, 1),
            preamble
        );
        let other = "crates/vize_canon/src/virtual_ts/helpers/declarations/core-types.d.ts.txt";
        assert_eq!(source_before(other, current(other)), current(other));
        let mut manifest = json!({ "provenance": { "existing": "unchanged" } });
        retain_provenance(&mut manifest);
        assert_eq!(manifest["provenance"]["existing"], "unchanged");
    }

    #[test]
    fn missing_duplicate_stale_and_extra_bytes_refuse() {
        for file in &FILES {
            let body = current(file.path);
            for mutated in [
                body.replacen(file.after, "", 1),
                format!("{body}{}", file.after),
                body.replacen(file.after, file.before, 1),
                format!("{body}\n"),
            ] {
                assert!(
                    std::panic::catch_unwind(|| source_before(file.path, mutated.clone())).is_err()
                );
            }
        }
    }
}

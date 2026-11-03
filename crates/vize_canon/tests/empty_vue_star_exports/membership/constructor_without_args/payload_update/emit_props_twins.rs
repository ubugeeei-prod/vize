//! Invert only the #11453 camel/kebab `__EmitProps` key guard before #11445.
use super::support;
use serde_json::{Value, json};

const CANONICAL_BEFORE: &str = "as $HandlerKey<K> | `on${Capitalize<K>}`]";
const CANONICAL_AFTER: &str = "as ($Camelize<K> extends keyof __EmitOptions<T> ? string extends keyof __EmitOptions<T> ? $HandlerKey<K> : never : $HandlerKey<K>) | `on${Capitalize<K>}`]";
const PUBLIC_BEFORE: &str =
    "    K in keyof __EmitOptions<T> & string as $HandlerKey<K> | `on${Capitalize<K>}`\n";
const PUBLIC_AFTER: &str = "    K in keyof __EmitOptions<T> & string as\n      | ($Camelize<K> extends keyof __EmitOptions<T>\n          ? string extends keyof __EmitOptions<T>\n            ? $HandlerKey<K>\n            : never\n          : $HandlerKey<K>)\n      | `on${Capitalize<K>}`\n";
const PUBLIC_BEFORE_SHA: &str = "d5e5eee803afd4fdb74912647d453322d36ce476daef487012eb1eca25f750df";
const PUBLIC_AFTER_SHA: &str = "96262fd9a40041ca76881234ec4705777474d6e6b319369d0205191312845b7f";
const PREAMBLE_BEFORE_BYTES: usize = 44_318;
const PREAMBLE_BEFORE_SHA: &str =
    "b2158cd05d330ec5da8876b5128d621a9130fdf799cbae6a7f79357999374eb2";
const PREAMBLE_AFTER_SHA: &str = "85e25abaee2c56205b506b1c87dd9e5305848d8840f0f13342ff8635517aae02";

struct File {
    path: &'static str,
    before: &'static str,
    after: &'static str,
    before_bytes: usize,
    before_sha: &'static str,
    after_sha: &'static str,
}

const FILES: [File; 5] = [
    File {
        path: "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-props.d.ts.txt",
        before: CANONICAL_BEFORE,
        after: CANONICAL_AFTER,
        before_bytes: 129,
        before_sha: "f61641504217b85ffd4d80f25da2d3598f58694a9dfded9f229d94cf88e84215",
        after_sha: "ede8a64664b4f16e139b909b77ce10fd911fa11164ed5eff53ad07b4bb1d4d30",
    },
    File {
        path: "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-emit-props.d.ts.txt",
        before: CANONICAL_BEFORE,
        after: CANONICAL_AFTER,
        before_bytes: 175,
        before_sha: "4c33222d29ef6da43b219f38876e96d1cc42ec15221b48e9a934c5582653ac36",
        after_sha: "0be7660bf70badeb69a5f854649c752845f62153b6d58e5d286912119bbb6e4d",
    },
    File {
        path: "npm/cli/virtual-ts.d.ts",
        before: PUBLIC_BEFORE,
        after: PUBLIC_AFTER,
        before_bytes: 15_334,
        before_sha: PUBLIC_BEFORE_SHA,
        after_sha: PUBLIC_AFTER_SHA,
    },
    File {
        path: "npm/cli/virtual-ts.d.mts",
        before: PUBLIC_BEFORE,
        after: PUBLIC_AFTER,
        before_bytes: 15_334,
        before_sha: PUBLIC_BEFORE_SHA,
        after_sha: PUBLIC_AFTER_SHA,
    },
    File {
        path: "npm/cli/virtual-ts.d.cts",
        before: PUBLIC_BEFORE,
        after: PUBLIC_AFTER,
        before_bytes: 15_334,
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
    let current = super::required_keys_short_circuit::source_before(path, current);
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
    let current = &super::required_keys_short_circuit::preamble_before(current);
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
    manifest["provenance"]["currentEmitPropsTwinsUpdate"] = json!({
        "kind": "exact-current-emit-props-twin-key-inverse",
        "issue": 11453,
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
    fn current_assets_preamble_and_sdk_reconstruct_only_the_emit_props_layer() {
        for file in &FILES {
            let body = current(file.path);
            let newer =
                super::super::required_keys_short_circuit::source_before(file.path, body.clone());
            let prior = source_before(file.path, body);
            assert_eq!(prior.replacen(file.before, file.after, 1), newer);
        }
        let preamble = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
        assert_eq!(
            preamble_before(preamble).replacen(FILES[1].before, FILES[1].after, 1),
            super::super::required_keys_short_circuit::preamble_before(preamble)
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

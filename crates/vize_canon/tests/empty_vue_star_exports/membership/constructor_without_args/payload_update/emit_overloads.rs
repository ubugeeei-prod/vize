//! Invert only the current tail-accumulator declaration before frozen history.
use super::support;
use serde_json::{Value, json};

const PATH: &str = "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt";
const BEFORE: &str = r#"type $OverloadUnionRecursive<TOverload, TPartialOverload = unknown, TDepth extends unknown[] = []> = TDepth['length'] extends 32 ? never : TOverload extends (...args: infer TArgs) => infer TReturn ? TPartialOverload extends TOverload ? never : $OverloadUnionRecursive<TPartialOverload & TOverload, TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload>, [...TDepth, unknown]> | ((...args: TArgs) => TReturn) : never;
"#;
const AFTER: &str = r#"type $OverloadUnionRecursive<TOverload, TPartialOverload = unknown, TDepth extends unknown[] = [], TAccumulated = never> = TDepth['length'] extends 900 ? TAccumulated : TOverload extends (...args: infer TArgs) => infer TReturn ? TPartialOverload extends TOverload ? TAccumulated : $OverloadUnionRecursive<TPartialOverload & TOverload, TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload>, [...TDepth, unknown], TAccumulated | ((...args: TArgs) => TReturn)> : TAccumulated;
"#;
const ASSET_BEFORE_SHA: &str = "23e4f39a94627fe0005c538b49f2a683f317c803dcdbd496fce1e7eef97e9321";
const ASSET_AFTER_SHA: &str = "084a0e36a868685e7a42532eb1c7d961f05725db32daa88c6be47b98de103540";
const PREAMBLE_BEFORE_SHA: &str =
    "1c403d85d0afecf6ec0c8fa303f5cc270857675f527133dcca2ce75ff80e9ff0";
const PREAMBLE_AFTER_SHA: &str = "0e5c9ee5c5ee3278da0a5ded6609f7202d031167e6306a6577082413455d26e4";

fn previous(current: &str, before_bytes: usize, before_sha: &str, after_sha: &str) -> String {
    assert_eq!(current.len(), before_bytes + 58);
    assert_eq!(support::hash(current.as_bytes()), after_sha);
    assert_eq!(current.matches(AFTER).count(), 1);
    assert!(!current.contains(BEFORE));
    let previous = current.replacen(AFTER, BEFORE, 1);
    assert_eq!(previous.len(), before_bytes);
    assert_eq!(support::hash(previous.as_bytes()), before_sha);
    previous
}

pub(super) fn source_before(path: &str, current: String) -> String {
    if path == PATH {
        previous(&current, 1606, ASSET_BEFORE_SHA, ASSET_AFTER_SHA)
    } else {
        current
    }
}

pub(super) fn preamble_before(current: &str) -> String {
    previous(current, 43_967, PREAMBLE_BEFORE_SHA, PREAMBLE_AFTER_SHA)
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["currentEmitOverloadTailUpdate"] = json!({
        "kind": "exact-current-emit-overload-tail-declaration-inverse",
        "issue": 11390,
        "nativeAcceptanceCredit": false,
        "deltaBytes": 58,
        "source": { "path": PATH, "beforeBytes": 1606, "afterBytes": 1664,
            "beforeSha256": ASSET_BEFORE_SHA, "afterSha256": ASSET_AFTER_SHA },
        "compiledPreamble": { "path": "vize_canon::virtual_ts::SHARED_PREAMBLE_DTS",
            "beforeBytes": 43_967, "afterBytes": 44_025,
            "beforeSha256": PREAMBLE_BEFORE_SHA, "afterSha256": PREAMBLE_AFTER_SHA }
    });
}

#[cfg(test)]
mod tests {
    use super::{
        AFTER, ASSET_AFTER_SHA, ASSET_BEFORE_SHA, BEFORE, PATH, PREAMBLE_AFTER_SHA,
        PREAMBLE_BEFORE_SHA, preamble_before, previous, retain_provenance, source_before, support,
    };
    use serde_json::json;

    const CURRENT_ASSET: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt"
    ));

    #[test]
    fn exact_current_asset_and_compiled_preamble_reconstruct_only_prior_declaration() {
        for (current, before_bytes, before_sha, after_sha) in [
            (CURRENT_ASSET, 1606, ASSET_BEFORE_SHA, ASSET_AFTER_SHA),
            (
                vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
                43_967,
                PREAMBLE_BEFORE_SHA,
                PREAMBLE_AFTER_SHA,
            ),
        ] {
            let prior = previous(current, before_bytes, before_sha, after_sha);
            assert_eq!(prior.matches(BEFORE).count(), 1);
            assert!(!prior.contains(AFTER));
            assert_eq!(prior.replacen(BEFORE, AFTER, 1), current);
        }
        assert_eq!(
            source_before(PATH, CURRENT_ASSET.to_owned()),
            previous(CURRENT_ASSET, 1606, ASSET_BEFORE_SHA, ASSET_AFTER_SHA)
        );
        assert_eq!(
            support::hash(preamble_before(vize_canon::virtual_ts::SHARED_PREAMBLE_DTS).as_bytes()),
            PREAMBLE_BEFORE_SHA
        );
        let other_path =
            "crates/vize_canon/src/virtual_ts/helpers/declarations/core-types.d.ts.txt";
        assert_eq!(
            source_before(other_path, CURRENT_ASSET.to_owned()),
            CURRENT_ASSET
        );
        let mut manifest = json!({ "provenance": { "existing": "unchanged" } });
        retain_provenance(&mut manifest);
        assert_eq!(manifest["provenance"]["existing"], "unchanged");
        assert_eq!(
            manifest["provenance"]["currentEmitOverloadTailUpdate"]["nativeAcceptanceCredit"],
            false
        );
    }

    #[test]
    fn missing_duplicate_stale_foreign_and_extra_bytes_refuse_for_both_whole_bodies() {
        for (current, before_bytes, before_sha, after_sha) in [
            (CURRENT_ASSET, 1606, ASSET_BEFORE_SHA, ASSET_AFTER_SHA),
            (
                vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
                43_967,
                PREAMBLE_BEFORE_SHA,
                PREAMBLE_AFTER_SHA,
            ),
        ] {
            for mutated in [
                current.replacen(AFTER, "", 1),
                format!("{current}{AFTER}"),
                current.replacen(AFTER, BEFORE, 1),
                current.replacen("type ", "xxxx ", 1),
                format!("{current}\n"),
            ] {
                assert!(
                    std::panic::catch_unwind(|| previous(
                        &mutated,
                        before_bytes,
                        before_sha,
                        after_sha
                    ))
                    .is_err()
                );
            }
            assert!(
                std::panic::catch_unwind(|| previous(
                    current,
                    before_bytes + 1,
                    before_sha,
                    after_sha
                ))
                .is_err()
            );
            assert!(
                std::panic::catch_unwind(|| previous(current, before_bytes, "unknown", after_sha))
                    .is_err()
            );
            assert!(
                std::panic::catch_unwind(|| previous(current, before_bytes, before_sha, "unknown"))
                    .is_err()
            );
        }
    }
}

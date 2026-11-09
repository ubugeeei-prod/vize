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

const PUBLIC_PATH: &str = "npm/cli/virtual-ts-globals.d.ts";
const PUBLIC_BEFORE: &str = r#"  type $OverloadUnionRecursive<
    TOverload,
    TPartialOverload = unknown,
    TDepth extends unknown[] = [],
  > = TDepth["length"] extends 32
    ? never
    : TOverload extends (...args: infer TArgs) => infer TReturn
      ? TPartialOverload extends TOverload
        ? never
        :
            | $OverloadUnionRecursive<
                TPartialOverload & TOverload,
                TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload>,
                [...TDepth, unknown]
              >
            | ((...args: TArgs) => TReturn)
      : never;
"#;
const PUBLIC_AFTER: &str = r#"  type $OverloadUnionRecursive<
    TOverload,
    TPartialOverload = unknown,
    TDepth extends unknown[] = [],
    TAccumulated = never,
  > = TDepth["length"] extends 900
    ? TAccumulated
    : TOverload extends (...args: infer TArgs) => infer TReturn
      ? TPartialOverload extends TOverload
        ? TAccumulated
        : $OverloadUnionRecursive<
            TPartialOverload & TOverload,
            TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload>,
            [...TDepth, unknown],
            TAccumulated | ((...args: TArgs) => TReturn)
          >
      : TAccumulated;
"#;
const PUBLIC_BEFORE_SHA: &str = "d647737ea035f166c94c131013af3265a3a33783dbad307b2eb3c6d558fc476a";
const PUBLIC_AFTER_SHA: &str = "14f9a36105a2d72aa6ddcbf25cef6c24839eb792cfa09f28450e3e22bf6fc234";

fn previous(current: &str, before_bytes: usize, before_sha: &str, after_sha: &str) -> String {
    assert_eq!(current.len(), before_bytes + 58);
    assert_eq!(support::hash(current.as_bytes()), after_sha);
    assert_eq!(current.matches(AFTER).count(), 1);
    assert_eq!(current.matches(BEFORE).count(), 0);
    let previous = current.replacen(AFTER, BEFORE, 1);
    assert_eq!(previous.len(), before_bytes);
    assert_eq!(support::hash(previous.as_bytes()), before_sha);
    previous
}

pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::canon_leftovers::source_before(path, current);
    let current = super::emit_overload_stall::source_before(path, current);
    if path == PATH {
        previous(&current, 1606, ASSET_BEFORE_SHA, ASSET_AFTER_SHA)
    } else if path == PUBLIC_PATH {
        previous_public(&current)
    } else {
        current
    }
}

fn previous_public(current: &str) -> String {
    assert_eq!(current.len(), 18_998);
    assert_eq!(support::hash(current.as_bytes()), PUBLIC_AFTER_SHA);
    assert_eq!(current.matches(PUBLIC_AFTER).count(), 1);
    assert_eq!(current.matches(PUBLIC_BEFORE).count(), 0);
    let previous = current.replacen(PUBLIC_AFTER, PUBLIC_BEFORE, 1);
    assert_eq!(previous.len(), 18_966);
    assert_eq!(support::hash(previous.as_bytes()), PUBLIC_BEFORE_SHA);
    previous
}

pub(super) fn preamble_before(current: &str) -> String {
    let current = &super::canon_leftovers::preamble_before(current);
    let current = &super::emit_overload_stall::preamble_before(current);
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
            "beforeSha256": PREAMBLE_BEFORE_SHA, "afterSha256": PREAMBLE_AFTER_SHA },
        "publishedSdk": { "path": PUBLIC_PATH, "deltaBytes": 32,
            "physicalRole": "actual-current-published-ambient-body",
            "historicalRole": "exact-previous-body-for-immutable-package-comparisons-only",
            "beforeBytes": 18_966, "afterBytes": 18_998,
            "beforeSha256": PUBLIC_BEFORE_SHA, "afterSha256": PUBLIC_AFTER_SHA }
    });
}

#[cfg(test)]
mod tests {
    use super::{
        AFTER, ASSET_AFTER_SHA, ASSET_BEFORE_SHA, BEFORE, PATH, PREAMBLE_AFTER_SHA,
        PREAMBLE_BEFORE_SHA, preamble_before, previous, retain_provenance, source_before, support,
    };
    use super::{
        PUBLIC_AFTER, PUBLIC_AFTER_SHA, PUBLIC_BEFORE, PUBLIC_BEFORE_SHA, PUBLIC_PATH,
        previous_public,
    };
    use serde_json::json;

    const CURRENT_STALL_ASSET: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt"
    ));

    fn tail_layer(path: &str, current: &str) -> String {
        super::super::emit_overload_stall::source_before(path, current.to_owned())
    }

    fn tail_preamble() -> String {
        super::super::emit_overload_stall::preamble_before(
            vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
        )
    }

    #[test]
    fn exact_current_asset_and_compiled_preamble_reconstruct_only_prior_declaration() {
        let current_asset = tail_layer(PATH, CURRENT_STALL_ASSET);
        let current_preamble = tail_preamble();
        for (current, before_bytes, before_sha, after_sha) in [
            (
                current_asset.as_str(),
                1606,
                ASSET_BEFORE_SHA,
                ASSET_AFTER_SHA,
            ),
            (
                current_preamble.as_str(),
                43_967,
                PREAMBLE_BEFORE_SHA,
                PREAMBLE_AFTER_SHA,
            ),
        ] {
            let prior = previous(current, before_bytes, before_sha, after_sha);
            assert_eq!(prior.matches(BEFORE).count(), 1);
            assert_eq!(prior.matches(AFTER).count(), 0);
            assert_eq!(prior.replacen(BEFORE, AFTER, 1), current);
        }
        assert_eq!(
            source_before(PATH, CURRENT_STALL_ASSET.to_owned()),
            previous(&current_asset, 1606, ASSET_BEFORE_SHA, ASSET_AFTER_SHA)
        );
        assert_eq!(
            support::hash(preamble_before(vize_canon::virtual_ts::SHARED_PREAMBLE_DTS).as_bytes()),
            PREAMBLE_BEFORE_SHA
        );
        let other_path =
            "crates/vize_canon/src/virtual_ts/helpers/declarations/vue3-types.d.ts.txt";
        assert_eq!(
            source_before(other_path, CURRENT_STALL_ASSET.to_owned()),
            CURRENT_STALL_ASSET
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
        let current_asset = tail_layer(PATH, CURRENT_STALL_ASSET);
        let current_preamble = tail_preamble();
        for (current, before_bytes, before_sha, after_sha) in [
            (
                current_asset.as_str(),
                1606,
                ASSET_BEFORE_SHA,
                ASSET_AFTER_SHA,
            ),
            (
                current_preamble.as_str(),
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
    #[test]
    fn current_published_globals_reconstruct_complete_historical_body_and_refuse_mutants() {
        let stall_current = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../npm/cli/virtual-ts-globals.d.ts"
        ));
        let current = &tail_layer(PUBLIC_PATH, stall_current);
        assert_eq!(support::hash(current.as_bytes()), PUBLIC_AFTER_SHA);
        let prior = previous_public(current);
        assert_eq!(prior.len(), 18_966);
        assert_eq!(support::hash(prior.as_bytes()), PUBLIC_BEFORE_SHA);
        assert_eq!(prior.matches(PUBLIC_BEFORE).count(), 1);
        assert_eq!(prior.matches(PUBLIC_AFTER).count(), 0);
        assert_eq!(&prior.replacen(PUBLIC_BEFORE, PUBLIC_AFTER, 1), current);
        assert_eq!(source_before(PUBLIC_PATH, stall_current.to_owned()), prior);
        for mutated in [
            current.replacen(PUBLIC_AFTER, "", 1),
            format!("{current}{PUBLIC_AFTER}"),
            current.replacen(PUBLIC_AFTER, PUBLIC_BEFORE, 1),
            current.replacen("type ", "xxxx ", 1),
            format!("{current}\n"),
        ] {
            assert!(std::panic::catch_unwind(|| previous_public(&mutated)).is_err());
        }
    }
}

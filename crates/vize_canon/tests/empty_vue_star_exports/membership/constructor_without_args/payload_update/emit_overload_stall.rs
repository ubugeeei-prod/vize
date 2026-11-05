//! Invert only the generic-overload stall guard before the tail-accumulator layer.
use super::support;
use serde_json::{Value, json};

const PATH: &str = "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt";
const BEFORE: &str = r#"type $OverloadUnionRecursive<TOverload, TPartialOverload = unknown, TDepth extends unknown[] = [], TAccumulated = never> = TDepth['length'] extends 900 ? TAccumulated : TOverload extends (...args: infer TArgs) => infer TReturn ? TPartialOverload extends TOverload ? TAccumulated : $OverloadUnionRecursive<TPartialOverload & TOverload, TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload>, [...TDepth, unknown], TAccumulated | ((...args: TArgs) => TReturn)> : TAccumulated;
"#;
const AFTER: &str = r#"type $OverloadUnionRecursive<TOverload, TPartialOverload = unknown, TDepth extends unknown[] = [], TAccumulated = never, TLast = never> = TDepth['length'] extends 900 ? TAccumulated : TOverload extends (...args: infer TArgs) => infer TReturn ? TPartialOverload extends TOverload ? TAccumulated : (<G>() => G extends (...args: TArgs) => TReturn ? 1 : 2) extends <G>() => G extends TLast ? 1 : 2 ? TAccumulated : $OverloadUnionRecursive<TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload> & TOverload, TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload>, [...TDepth, unknown], TAccumulated | ((...args: TArgs) => TReturn), (...args: TArgs) => TReturn> : TAccumulated;
"#;
const DELTA: usize = 219;
const ASSET_BEFORE_SHA: &str = "084a0e36a868685e7a42532eb1c7d961f05725db32daa88c6be47b98de103540";
const ASSET_AFTER_SHA: &str = "ef2a99a2e675a810472b7a3cc6c9a4c9a905f12369aa1366dc9605af94b9c6f5";
const PREAMBLE_BEFORE_SHA: &str =
    "0e5c9ee5c5ee3278da0a5ded6609f7202d031167e6306a6577082413455d26e4";
const PREAMBLE_AFTER_SHA: &str = "298f0dd19a3114ee2fc55f67b598a5a3b1649c356b4f453be52312b854e65816";

const PUBLIC_PATH: &str = "npm/cli/virtual-ts-globals.d.ts";
const PUBLIC_BEFORE: &str = r#"  type $OverloadUnionRecursive<
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
const PUBLIC_AFTER: &str = r#"  // One inference per step; repeating one means a generic overload is stuck.
  type $OverloadUnionRecursive<
    TOverload,
    TPartialOverload = unknown,
    TDepth extends unknown[] = [],
    TAccumulated = never,
    TLast = never,
  > = TDepth["length"] extends 900
    ? TAccumulated
    : TOverload extends (...args: infer TArgs) => infer TReturn
      ? TPartialOverload extends TOverload
        ? TAccumulated
        : (<G>() => G extends (...args: TArgs) => TReturn ? 1 : 2) extends <G>() => G extends TLast
              ? 1
              : 2
          ? TAccumulated
          : $OverloadUnionRecursive<
              TPartialOverload &
                ((...args: TArgs) => TReturn) &
                $OverloadProps<TOverload> &
                TOverload,
              TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload>,
              [...TDepth, unknown],
              TAccumulated | ((...args: TArgs) => TReturn),
              (...args: TArgs) => TReturn
            >
      : TAccumulated;
"#;
const PUBLIC_AFTER_BYTES: usize = 19_419;
const PUBLIC_BEFORE_SHA: &str = "14f9a36105a2d72aa6ddcbf25cef6c24839eb792cfa09f28450e3e22bf6fc234";
const PUBLIC_AFTER_SHA: &str = "0ad1f8a8892956b693732431fc6a1e277ee1c1b929d750cadf846248ae1a981e";

fn previous(current: &str, before_bytes: usize, before_sha: &str, after_sha: &str) -> String {
    assert_eq!(current.len(), before_bytes + DELTA);
    assert_eq!(support::hash(current.as_bytes()), after_sha);
    assert_eq!(current.matches(AFTER).count(), 1);
    assert_eq!(current.matches(BEFORE).count(), 0);
    let previous = current.replacen(AFTER, BEFORE, 1);
    assert_eq!(previous.len(), before_bytes);
    assert_eq!(support::hash(previous.as_bytes()), before_sha);
    previous
}

fn previous_public(current: &str) -> String {
    assert_eq!(current.len(), PUBLIC_AFTER_BYTES);
    assert_eq!(support::hash(current.as_bytes()), PUBLIC_AFTER_SHA);
    assert_eq!(current.matches(PUBLIC_AFTER).count(), 1);
    assert_eq!(current.matches(PUBLIC_BEFORE).count(), 0);
    let previous = current.replacen(PUBLIC_AFTER, PUBLIC_BEFORE, 1);
    assert_eq!(previous.len(), 18_998);
    assert_eq!(support::hash(previous.as_bytes()), PUBLIC_BEFORE_SHA);
    previous
}

pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::public_base_keys::source_before(path, current);
    if path == PATH {
        previous(&current, 1664, ASSET_BEFORE_SHA, ASSET_AFTER_SHA)
    } else if path == PUBLIC_PATH {
        previous_public(&current)
    } else {
        current
    }
}

pub(super) fn preamble_before(current: &str) -> String {
    let current = &super::public_base_keys::preamble_before(current);
    previous(current, 44_025, PREAMBLE_BEFORE_SHA, PREAMBLE_AFTER_SHA)
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["currentEmitOverloadStallUpdate"] = json!({
        "kind": "exact-current-emit-overload-stall-declaration-inverse",
        "nativeAcceptanceCredit": false,
        "deltaBytes": DELTA,
        "source": { "path": PATH, "beforeBytes": 1664, "afterBytes": 1664 + DELTA,
            "beforeSha256": ASSET_BEFORE_SHA, "afterSha256": ASSET_AFTER_SHA },
        "compiledPreamble": { "path": "vize_canon::virtual_ts::SHARED_PREAMBLE_DTS",
            "beforeBytes": 44_025, "afterBytes": 44_025 + DELTA,
            "beforeSha256": PREAMBLE_BEFORE_SHA, "afterSha256": PREAMBLE_AFTER_SHA },
        "publishedSdk": { "path": PUBLIC_PATH,
            "beforeBytes": 18_998, "afterBytes": PUBLIC_AFTER_BYTES,
            "beforeSha256": PUBLIC_BEFORE_SHA, "afterSha256": PUBLIC_AFTER_SHA }
    });
}

#[cfg(test)]
mod tests {
    use super::{
        AFTER, BEFORE, PATH, PUBLIC_AFTER, PUBLIC_BEFORE, PUBLIC_PATH, preamble_before, previous,
        previous_public, retain_provenance, source_before,
    };
    use serde_json::json;

    const CURRENT_ASSET: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt"
    ));
    const CURRENT_PUBLIC: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../npm/cli/virtual-ts-globals.d.ts"
    ));

    #[test]
    fn current_asset_preamble_and_sdk_reconstruct_only_the_tail_accumulator_layer() {
        let asset = source_before(PATH, CURRENT_ASSET.to_owned());
        assert_eq!(
            asset.replacen(BEFORE, AFTER, 1),
            super::super::public_base_keys::source_before(PATH, CURRENT_ASSET.to_owned())
        );
        let newer = super::super::public_base_keys::preamble_before(
            vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
        );
        let preamble = preamble_before(vize_canon::virtual_ts::SHARED_PREAMBLE_DTS);
        assert_eq!(preamble.replacen(BEFORE, AFTER, 1), newer);
        let newer =
            super::super::public_base_keys::source_before(PUBLIC_PATH, CURRENT_PUBLIC.to_owned());
        let public = source_before(PUBLIC_PATH, CURRENT_PUBLIC.to_owned());
        assert_eq!(public.replacen(PUBLIC_BEFORE, PUBLIC_AFTER, 1), newer);
        let other = "crates/vize_canon/src/virtual_ts/helpers/declarations/vue3-types.d.ts.txt";
        assert_eq!(
            source_before(other, CURRENT_ASSET.to_owned()),
            CURRENT_ASSET
        );
        let mut manifest = json!({ "provenance": { "existing": "unchanged" } });
        retain_provenance(&mut manifest);
        assert_eq!(manifest["provenance"]["existing"], "unchanged");
    }

    #[test]
    fn missing_duplicate_stale_and_extra_bytes_refuse() {
        let current = super::super::public_base_keys::source_before(PATH, CURRENT_ASSET.to_owned());
        for mutated in [
            current.replacen(AFTER, "", 1),
            format!("{current}{AFTER}"),
            current.replacen(AFTER, BEFORE, 1),
            format!("{current}\n"),
        ] {
            assert!(
                std::panic::catch_unwind(|| previous(
                    &mutated,
                    1664,
                    super::ASSET_BEFORE_SHA,
                    super::ASSET_AFTER_SHA,
                ))
                .is_err()
            );
        }
        let current =
            super::super::public_base_keys::source_before(PUBLIC_PATH, CURRENT_PUBLIC.to_owned());
        for mutated in [
            current.replacen(PUBLIC_AFTER, "", 1),
            current.replacen(PUBLIC_AFTER, PUBLIC_BEFORE, 1),
            format!("{current}\n"),
        ] {
            assert!(std::panic::catch_unwind(|| previous_public(&mutated)).is_err());
        }
    }
}

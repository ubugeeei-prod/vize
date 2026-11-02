//! Invert only the generic-overload stall guard before the tail-accumulator layer.
use super::support;
use serde_json::{Value, json};

const PATH: &str = "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt";
const BEFORE: &str = r#"type $OverloadUnionRecursive<TOverload, TPartialOverload = unknown, TDepth extends unknown[] = [], TAccumulated = never> = TDepth['length'] extends 900 ? TAccumulated : TOverload extends (...args: infer TArgs) => infer TReturn ? TPartialOverload extends TOverload ? TAccumulated : $OverloadUnionRecursive<TPartialOverload & TOverload, TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload>, [...TDepth, unknown], TAccumulated | ((...args: TArgs) => TReturn)> : TAccumulated;
"#;
const AFTER: &str = r#"type $OverloadUnionRecursive<TOverload, TPartialOverload = unknown, TDepth extends unknown[] = [], TAccumulated = never, TLast = never, TStalled = false> = TDepth['length'] extends 900 ? TAccumulated : TOverload extends (...args: infer TArgs) => infer TReturn ? TPartialOverload extends TOverload ? TAccumulated : (<G>() => G extends (...args: TArgs) => TReturn ? 1 : 2) extends <G>() => G extends TLast ? 1 : 2 ? TStalled extends true ? TAccumulated : $OverloadUnionRecursive<TPartialOverload & TOverload, TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload>, [...TDepth, unknown], TAccumulated, TLast, true> : $OverloadUnionRecursive<TPartialOverload & TOverload, TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload>, [...TDepth, unknown], TAccumulated | ((...args: TArgs) => TReturn), (...args: TArgs) => TReturn, false> : TAccumulated;
"#;
const DELTA: usize = 391;
const ASSET_BEFORE_SHA: &str = "084a0e36a868685e7a42532eb1c7d961f05725db32daa88c6be47b98de103540";
const ASSET_AFTER_SHA: &str = "c6f56093aaf00542757fcd96031ac872b5241e643de4516b8186469158f6a0ec";
const PREAMBLE_BEFORE_SHA: &str =
    "0e5c9ee5c5ee3278da0a5ded6609f7202d031167e6306a6577082413455d26e4";
const PREAMBLE_AFTER_SHA: &str = "1bf3fb3a3a3ce15e3c0f8695083af7530f18b079dc9c049bbac65f2cf0aaa0ed";

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
const PUBLIC_AFTER: &str = r#"  // Peeling infers each signature twice. A third identical inference means a
  // generic overload cannot be peeled, so stop instead of recursing to 900.
  type $OverloadUnionRecursive<
    TOverload,
    TPartialOverload = unknown,
    TDepth extends unknown[] = [],
    TAccumulated = never,
    TLast = never,
    TStalled = false,
  > = TDepth["length"] extends 900
    ? TAccumulated
    : TOverload extends (...args: infer TArgs) => infer TReturn
      ? TPartialOverload extends TOverload
        ? TAccumulated
        : (<G>() => G extends (...args: TArgs) => TReturn ? 1 : 2) extends <G>() => G extends TLast
              ? 1
              : 2
          ? TStalled extends true
            ? TAccumulated
            : $OverloadUnionRecursive<
                TPartialOverload & TOverload,
                TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload>,
                [...TDepth, unknown],
                TAccumulated,
                TLast,
                true
              >
          : $OverloadUnionRecursive<
              TPartialOverload & TOverload,
              TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload>,
              [...TDepth, unknown],
              TAccumulated | ((...args: TArgs) => TReturn),
              (...args: TArgs) => TReturn,
              false
            >
      : TAccumulated;
"#;
const PUBLIC_AFTER_BYTES: usize = 19_774;
const PUBLIC_BEFORE_SHA: &str = "14f9a36105a2d72aa6ddcbf25cef6c24839eb792cfa09f28450e3e22bf6fc234";
const PUBLIC_AFTER_SHA: &str = "94519d695e2eca5632a76f49cbd5ee0a38b6b000cb660cf23e5f4bad1f31affb";

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
    if path == PATH {
        previous(&current, 1664, ASSET_BEFORE_SHA, ASSET_AFTER_SHA)
    } else if path == PUBLIC_PATH {
        previous_public(&current)
    } else {
        current
    }
}

pub(super) fn preamble_before(current: &str) -> String {
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
        AFTER, BEFORE, PATH, PUBLIC_AFTER, PUBLIC_BEFORE, PUBLIC_PATH, preamble_before,
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
        assert_eq!(asset.replacen(BEFORE, AFTER, 1), CURRENT_ASSET);
        let preamble = preamble_before(vize_canon::virtual_ts::SHARED_PREAMBLE_DTS);
        assert_eq!(
            preamble.replacen(BEFORE, AFTER, 1),
            vize_canon::virtual_ts::SHARED_PREAMBLE_DTS
        );
        let public = source_before(PUBLIC_PATH, CURRENT_PUBLIC.to_owned());
        assert_eq!(
            public.replacen(PUBLIC_BEFORE, PUBLIC_AFTER, 1),
            CURRENT_PUBLIC
        );
        let other = "crates/vize_canon/src/virtual_ts/helpers/declarations/core-types.d.ts.txt";
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
        for mutated in [
            CURRENT_ASSET.replacen(AFTER, "", 1),
            format!("{CURRENT_ASSET}{AFTER}"),
            CURRENT_ASSET.replacen(AFTER, BEFORE, 1),
            format!("{CURRENT_ASSET}\n"),
        ] {
            assert!(std::panic::catch_unwind(|| source_before(PATH, mutated.clone())).is_err());
        }
        for mutated in [
            CURRENT_PUBLIC.replacen(PUBLIC_AFTER, "", 1),
            CURRENT_PUBLIC.replacen(PUBLIC_AFTER, PUBLIC_BEFORE, 1),
            format!("{CURRENT_PUBLIC}\n"),
        ] {
            assert!(std::panic::catch_unwind(|| previous_public(&mutated)).is_err());
        }
    }
}

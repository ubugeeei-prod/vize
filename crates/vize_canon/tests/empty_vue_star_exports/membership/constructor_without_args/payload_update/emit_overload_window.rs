//! Invert the #11435 emit-overload module split and windowed peeling first.
use super::support;
use serde_json::{Value, json};

pub(super) const GLOBALS_BEFORE: &str = r#"  // Emit-overload helpers (consumed by the per-file __EmitProps alias)

  type $OverloadProps<TOverload> = Pick<TOverload, keyof TOverload>;
  // One inference per step; repeating one means a generic overload is stuck.
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
  type $OverloadUnion<TOverload extends (...args: any[]) => any> = Exclude<
    $OverloadUnionRecursive<(() => never) & TOverload>,
    TOverload extends () => never ? never : () => never
  >;
  type $OverloadParameters<T extends (...args: any[]) => any> = Parameters<$OverloadUnion<T>>;
  type $IsStringLiteral<T> = T extends string ? (string extends T ? false : true) : false;
  type $ParametersToFns<T extends any[]> = string extends T[0]
    ? { [K in string]: (...args: T extends [e: any, ...args: infer P] ? P : any[]) => any }
    : {
        [K in T[0]]: $IsStringLiteral<K> extends true
          ? (
              ...args: T extends [e: infer E, ...args: infer P] ? (K extends E ? P : never) : never
            ) => any
          : never;
      };
  type __EmitOptions<T, S = __EmitShape<T>> = {
    [K in keyof S as K extends string ? K : never]: (...args: __EmitArgs<S, K>) => any;
  } & (__EmitShape<T> extends (...args: any[]) => any
    ? $ParametersToFns<$OverloadParameters<__EmitShape<T>>>
    : {});
  type $Camelize<S extends string> = S extends `${infer Head}-${infer Tail}`
    ? `${Head}${Capitalize<$Camelize<Tail>>}`
    : S;
  type $HandlerKey<K extends string> = `on${Capitalize<$Camelize<K>>}`;

  // Compiler-macro signatures (aliased inside each module's __setup() scope)
"#;
pub(super) const GLOBALS_AFTER: &str =
    "  // Compiler-macro signatures (aliased inside each module's __setup() scope)\n";
const ENTRY_BEFORE: &str =
    "import type {} from \"./virtual-ts-patterns.js\" with { \"resolution-mode\": \"import\" };\n";
const ENTRY_AFTER: &str = r#"import type {} from "./virtual-ts-patterns.js" with { "resolution-mode": "import" };
import type {} from "./virtual-ts-emit-overloads.js" with { "resolution-mode": "import" };
"#;
pub(super) const GENERATOR_BEFORE: &str = r#"
// The checked-in virtual-ts exports predate the canonical zero-call policy.
"#;
pub(super) const GENERATOR_AFTER: &str = r#"// The published emit-overload helpers are the canonical asset, declared globally.
const emitOverloads = await format(
  "virtual-ts-emit-overloads.d.ts",
  "// Generated from the canonical emit-overload helpers. Do not edit.\ndeclare global {\n" +
    readFileSync(path.join(root, helperRoot, "declarations/emit-overloads.d.ts.txt"), "utf8") +
    "}\n\nexport {};\n",
  { printWidth: 100 },
);
assert.deepEqual(emitOverloads.errors, [], "canonical emit-overload helpers must parse");
outputs.set("npm/cli/virtual-ts-emit-overloads.d.ts", emitOverloads.code);

// The checked-in virtual-ts exports predate the canonical zero-call policy.
"#;
pub(super) const MANIFEST_BEFORE: &str = "    \"virtual-ts-globals.d.ts\",\n";
pub(super) const MANIFEST_AFTER: &str = r#"    "virtual-ts-emit-overloads.d.ts",
    "virtual-ts-globals.d.ts",
"#;
pub(super) const ASSET_BEFORE: &str = "type $OverloadUnionRecursive<TOverload, TPartialOverload = unknown, TDepth extends unknown[] = [], TAccumulated = never, TLast = never> = TDepth['length'] extends 900 ? TAccumulated : TOverload extends (...args: infer TArgs) => infer TReturn ? TPartialOverload extends TOverload ? TAccumulated : (<G>() => G extends (...args: TArgs) => TReturn ? 1 : 2) extends <G>() => G extends TLast ? 1 : 2 ? TAccumulated : $OverloadUnionRecursive<TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload> & TOverload, TPartialOverload & ((...args: TArgs) => TReturn) & $OverloadProps<TOverload>, [...TDepth, unknown], TAccumulated | ((...args: TArgs) => TReturn), (...args: TArgs) => TReturn> : TAccumulated;\n";
pub(super) const ASSET_AFTER: &str = "type $OverloadUnionRecursive<TOverload, TPartialOverload = unknown, TDepth extends unknown[] = [], TAccumulated = never> = TDepth['length'] extends 120 ? TAccumulated : TOverload extends unknown ? TPartialOverload extends TOverload ? TAccumulated : TPartialOverload & TOverload extends { (...args: infer A0): infer R0; (...args: infer A1): infer R1; (...args: infer A2): infer R2; (...args: infer A3): infer R3; (...args: infer A4): infer R4; (...args: infer A5): infer R5; (...args: infer A6): infer R6; (...args: infer A7): infer R7; } ? (<G>() => G extends TAccumulated ? 1 : 2) extends <G>() => G extends | TAccumulated | ((...args: A0) => R0) | ((...args: A1) => R1) | ((...args: A2) => R2) | ((...args: A3) => R3) | ((...args: A4) => R4) | ((...args: A5) => R5) | ((...args: A6) => R6) | ((...args: A7) => R7) ? 1 : 2 ? TAccumulated : $OverloadUnionRecursive<TOverload, TPartialOverload & ((...args: A0) => R0) & ((...args: A1) => R1) & ((...args: A2) => R2) & ((...args: A3) => R3) & ((...args: A4) => R4) & ((...args: A5) => R5) & ((...args: A6) => R6) & ((...args: A7) => R7) & $OverloadProps<TOverload>, [...TDepth, unknown], | TAccumulated | ((...args: A0) => R0) | ((...args: A1) => R1) | ((...args: A2) => R2) | ((...args: A3) => R3) | ((...args: A4) => R4) | ((...args: A5) => R5) | ((...args: A6) => R6) | ((...args: A7) => R7)> : TAccumulated : never;\n";
const PREAMBLE_BEFORE_BYTES: usize = 44748;
const PREAMBLE_BEFORE_SHA: &str =
    "378516f3c59cecb166ab9197761b77679824ea7145efd84d752bf5afd3060b07";
const PREAMBLE_AFTER_SHA: &str = "00ed9d57c1fa391e77565be2785bd8508800c4d5ee733d4117335de6f6b53e35";

const PUBLISHED_PATH: &str = "npm/cli/virtual-ts-emit-overloads.d.ts";
const PUBLISHED_BYTES: usize = 3511;
const PUBLISHED_SHA: &str = "a48dd34f5cbcfa8078d88d7c915d1024225e6cb2b708e843b490b131299d3cec";

fn authenticate_published(current: &str) {
    assert_eq!(current.len(), PUBLISHED_BYTES);
    assert_eq!(support::hash(current.as_bytes()), PUBLISHED_SHA);
}

fn authenticate_current_module() {
    let body = std::fs::read_to_string(support::repository().join(PUBLISHED_PATH)).unwrap();
    let body = super::canon_leftovers::source_before(PUBLISHED_PATH, body);
    authenticate_published(&body);
}

struct File {
    path: &'static str,
    before: &'static str,
    after: &'static str,
    before_bytes: usize,
    before_sha: &'static str,
    after_sha: &'static str,
}

const FILES: [File; 7] = [
    File {
        path: "npm/cli/virtual-ts-globals.d.ts",
        before: GLOBALS_BEFORE,
        after: GLOBALS_AFTER,
        before_bytes: 19261,
        before_sha: "1077ed40b76672245658ad66ded95531ccfbe41405b713b44126c1704f6c1cd2",
        after_sha: "b2fbe44f76255c08b4ad14ba531222e6c25d0ee8f0576f8602d456b31f9846a3",
    },
    File {
        path: "npm/cli/virtual-ts.d.ts",
        before: ENTRY_BEFORE,
        after: ENTRY_AFTER,
        before_bytes: 15549,
        before_sha: "363aa2912d43f63c940a6af5f0eae6bc75dabac8cd97752fda1031b84c409f5e",
        after_sha: "88c58326ac0dbbffcec102e6ffa28a773a2f2814a6f7b6e30bb62ebf4d1a2dcd",
    },
    File {
        path: "npm/cli/virtual-ts.d.mts",
        before: ENTRY_BEFORE,
        after: ENTRY_AFTER,
        before_bytes: 15549,
        before_sha: "363aa2912d43f63c940a6af5f0eae6bc75dabac8cd97752fda1031b84c409f5e",
        after_sha: "88c58326ac0dbbffcec102e6ffa28a773a2f2814a6f7b6e30bb62ebf4d1a2dcd",
    },
    File {
        path: "npm/cli/virtual-ts.d.cts",
        before: ENTRY_BEFORE,
        after: ENTRY_AFTER,
        before_bytes: 15549,
        before_sha: "363aa2912d43f63c940a6af5f0eae6bc75dabac8cd97752fda1031b84c409f5e",
        after_sha: "88c58326ac0dbbffcec102e6ffa28a773a2f2814a6f7b6e30bb62ebf4d1a2dcd",
    },
    File {
        path: "npm/cli/scripts/generate-content-mapper-types.ts",
        before: GENERATOR_BEFORE,
        after: GENERATOR_AFTER,
        before_bytes: 17820,
        before_sha: "a32212a7833a6fcc7aa8998c0c9489dcdde895fb482a2143dad2b0708f932503",
        after_sha: "a846d1bb34256e11b44b6f0ab2933c3d379de821acecf6f3fd06024a5afb16db",
    },
    File {
        path: "npm/cli/package.json",
        before: MANIFEST_BEFORE,
        after: MANIFEST_AFTER,
        before_bytes: 4173,
        before_sha: "b6b57a0dd4222983f31bcd98c3a78455e58d03d1a2b175be6e240f032702fc7e",
        after_sha: "c1139648d97de1301df9bae5560033fcd50db25db1acfd6834b601b576e5e206",
    },
    File {
        path: "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt",
        before: ASSET_BEFORE,
        after: ASSET_AFTER,
        before_bytes: 1883,
        before_sha: "ef2a99a2e675a810472b7a3cc6c9a4c9a905f12369aa1366dc9605af94b9c6f5",
        after_sha: "924bb683376f6c2ab3b9225cd6eff73328f8dfe4e87529a1c1a8068c03923e74",
    },
];

fn invert(current: &str, file: &File) -> String {
    assert_eq!(
        current.len() + file.before.len(),
        file.before_bytes + file.after.len()
    );
    assert_eq!(support::hash(current.as_bytes()), file.after_sha);
    assert_eq!(current.matches(file.after).count(), 1);
    let previous = current.replacen(file.after, file.before, 1);
    assert_eq!(previous.len(), file.before_bytes);
    assert_eq!(support::hash(previous.as_bytes()), file.before_sha);
    previous
}

pub(super) fn source_before(path: &str, current: String) -> String {
    authenticate_current_module();
    match FILES.iter().find(|file| file.path == path) {
        Some(file) => invert(&current, file),
        None => current,
    }
}

pub(super) fn preamble_before(current: &str) -> String {
    authenticate_current_module();
    invert(
        current,
        &File {
            path: "vize_canon::virtual_ts::SHARED_PREAMBLE_DTS",
            before: ASSET_BEFORE,
            after: ASSET_AFTER,
            before_bytes: PREAMBLE_BEFORE_BYTES,
            before_sha: PREAMBLE_BEFORE_SHA,
            after_sha: PREAMBLE_AFTER_SHA,
        },
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
    manifest["provenance"]["currentEmitOverloadWindowUpdate"] = json!({
        "kind": "exact-current-emit-overload-module-window-inverse",
        "issue": 11435,
        "nativeAcceptanceCredit": false,
        "publishedModule": { "path": PUBLISHED_PATH, "bytes": PUBLISHED_BYTES, "sha256": PUBLISHED_SHA },
        "files": files,
        "compiledPreamble": { "path": "vize_canon::virtual_ts::SHARED_PREAMBLE_DTS",
            "beforeBytes": PREAMBLE_BEFORE_BYTES,
            "afterBytes": PREAMBLE_BEFORE_BYTES + ASSET_AFTER.len() - ASSET_BEFORE.len(),
            "beforeSha256": PREAMBLE_BEFORE_SHA, "afterSha256": PREAMBLE_AFTER_SHA }
    });
}

#[cfg(test)]
mod tests {
    use super::{FILES, invert, preamble_before, retain_provenance, source_before};
    use serde_json::json;

    #[test]
    fn published_module_is_complete_and_missing_or_foreign_bytes_are_refused() {
        let body = current(super::PUBLISHED_PATH);
        super::authenticate_published(&body);
        for changed in [
            String::new(),
            format!("{body}\n"),
            body.replacen("type $OverloadProps", "type $ForeignProps", 1),
        ] {
            assert_ne!(changed, body);
            assert!(std::panic::catch_unwind(|| super::authenticate_published(&changed)).is_err());
        }
    }

    fn current(path: &str) -> String {
        let current = std::fs::read_to_string(super::support::repository().join(path)).unwrap();
        super::super::canon_leftovers::source_before(path, current)
    }

    #[test]
    fn current_files_and_preamble_reconstruct_only_the_emit_overload_window_layer() {
        for file in &FILES {
            let physical = current(file.path);
            let integrated = super::super::integrated_source::source_before(file.path, physical);
            let prior = super::super::checker_augmentations::source_before(file.path, integrated);
            let body = prior.replacen(file.before, file.after, 1);
            assert_eq!(source_before(file.path, body), prior);
        }
        let physical = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
        let integrated = super::super::integrated_source::preamble_before(physical);
        let prior = super::super::current_loose_emit::preamble_before(&integrated);
        let preamble = prior.replacen(super::ASSET_BEFORE, super::ASSET_AFTER, 1);
        assert_eq!(preamble_before(&preamble), prior);
        let other = "crates/vize_canon/src/virtual_ts/helpers/declarations/core-types.d.ts.txt";
        assert_eq!(source_before(other, current(other)), current(other));
        let mut manifest = json!({ "provenance": { "existing": "unchanged" } });
        retain_provenance(&mut manifest);
        assert_eq!(manifest["provenance"]["existing"], "unchanged");
    }

    #[test]
    fn missing_duplicate_stale_and_extra_bytes_refuse() {
        for file in &FILES {
            let physical = current(file.path);
            let integrated = super::super::integrated_source::source_before(file.path, physical);
            let prior = super::super::checker_augmentations::source_before(file.path, integrated);
            let body = prior.replacen(file.before, file.after, 1);
            for mutated in [
                body.replacen(file.after, "", 1),
                format!("{body}{}", file.after),
                body.replacen(file.after, file.before, 1),
                format!("{body}\n"),
            ] {
                assert!(std::panic::catch_unwind(|| invert(&mutated, file)).is_err());
            }
        }
    }
}

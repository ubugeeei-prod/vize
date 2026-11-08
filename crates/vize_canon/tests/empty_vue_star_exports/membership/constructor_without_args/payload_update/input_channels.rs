//! Invert only the #11629 split of template and TSX input channels before #11630.
use super::support;
use serde_json::{Value, json};

const F0_BEFORE_0: &str = "type $RelaxedDynamicUnionExtra<T, Declared, Authored, K extends keyof Authored> = [keyof Declared] extends [never] ? $IsSinglePropKey<$RequiredKeys<T>> extends true ? K extends string ? K extends `on${string}` ? false : Authored[K] extends true ? false : true : false : false : false;\ntype $ExtraInputKeys<T, Declared, Authored, Allowed, K extends keyof Authored = Exclude<Exclude<keyof Authored, keyof Allowed>, typeof globalThis extends { __VLS_PROPS_FALLBACK: infer P; __vizeTemplateChecker?: never } ? keyof P : never>> = K extends unknown ? $RelaxedDynamicUnionExtra<T, Declared, Authored, K> extends true ? never : K : never;\ntype $ComponentMissingInputGuard<T, Authored> = T extends unknown ? [$MissingRequiredInputKeys<T, Authored>] extends [never] ? {} : { readonly __vizeInvalidComponentProps: { readonly missing: $MissingRequiredInputKeys<T, Authored>; readonly extra: never; } } : never;\n";
const F0_AFTER_0: &str = "type $RelaxedDynamicUnionExtra<T, Declared, Authored, K extends keyof Authored> = [keyof Declared] extends [never] ? $IsSinglePropKey<$RequiredKeys<T>> extends true ? K extends string ? K extends `on${string}` ? false : Authored[K] extends true ? false : true : false : false : false;\ntype $ExtraInputKeys<T, Declared, Authored, Allowed, K extends keyof Authored = Exclude<keyof Authored, keyof Allowed>> = K extends unknown ? $RelaxedDynamicUnionExtra<T, Declared, Authored, K> extends true ? never : K : never;\ntype $ComponentMissingInputGuard<T, Authored> = T extends unknown ? [$MissingRequiredInputKeys<T, Authored>] extends [never] ? {} : { readonly __vizeInvalidComponentProps: { readonly missing: $MissingRequiredInputKeys<T, Authored>; readonly extra: never; } } : never;\n";
const F0_BEFORE_1: &str = "type $AuthoredKeyWitness<T> = { [K in keyof T]: unknown };\ntype $IsUnknown<T> = unknown extends T ? [keyof T] extends [never] ? true : false : false;\n";
const F0_AFTER_1: &str = "type $AuthoredKeyWitness<T> = { [K in keyof T]: unknown };\n// Constructor arguments (templates, `new`) bind `Authored`; TSX binds `Jsx` through `$props` (#11629).\ntype $ComponentInstanceFor<I, T, Declared, Authored, Jsx> = $IsAny<Authored> extends true ? I : $IsUnknown<Authored> extends true ? $UsePublicInstance<Jsx> extends true ? I : $ReplaceComponentProps<I, $AuthoredKeyWitness<Jsx> & $ComponentInputProps<T, Declared> & $ComponentInputGuard<T, Declared, Jsx>> : $ReplaceComponentProps<I, $ComponentInputProps<T, Declared> & $ComponentMissingInputGuard<T, Authored>>;\ntype $IsUnknown<T> = unknown extends T ? [keyof T] extends [never] ? true : false : false;\n";
const F1_BEFORE_0: &str = "declare var __VLS_PROPS_FALLBACK: Record<string, unknown>;\n// vize checks these templates itself, so component input guards serve only\n// TS and TSX callers and keep extra keys exact.\ndeclare var __vizeTemplateChecker: true;\n\n";
const F1_AFTER_0: &str = "declare var __VLS_PROPS_FALLBACK: Record<string, unknown>;\n\n";
const F2_BEFORE_0: &str = "    Allowed,\n    K extends keyof Authored = Exclude<\n      Exclude<keyof Authored, keyof Allowed>,\n      typeof globalThis extends { __VLS_PROPS_FALLBACK: infer P; __vizeTemplateChecker?: never }\n        ? keyof P\n        : never\n    >,\n  > = K extends unknown\n";
const F2_AFTER_0: &str = "    Allowed,\n    K extends keyof Authored = Exclude<keyof Authored, keyof Allowed>,\n  > = K extends unknown\n";
const F2_BEFORE_1: &str = "  type $AuthoredKeyWitness<T> = { [K in keyof T]: unknown };\n  type $IsUnknown<T> = unknown extends T ? ([keyof T] extends [never] ? true : false) : false;\n";
const F2_AFTER_1: &str = "  type $AuthoredKeyWitness<T> = { [K in keyof T]: unknown };\n  // Constructor arguments (templates, `new`) bind `Authored`; TSX binds `Jsx` through `$props` (#11629).\n  type $ComponentInstanceFor<I, T, Declared, Authored, Jsx> =\n    $IsAny<Authored> extends true\n      ? I\n      : $IsUnknown<Authored> extends true\n        ? $UsePublicInstance<Jsx> extends true\n          ? I\n          : $ReplaceComponentProps<\n              I,\n              $AuthoredKeyWitness<Jsx> &\n                $ComponentInputProps<T, Declared> &\n                $ComponentInputGuard<T, Declared, Jsx>\n            >\n        : $ReplaceComponentProps<\n            I,\n            $ComponentInputProps<T, Declared> & $ComponentMissingInputGuard<T, Authored>\n          >;\n  type $IsUnknown<T> = unknown extends T ? ([keyof T] extends [never] ? true : false) : false;\n";
const F3_BEFORE_0: &str = "  Allowed,\n  K extends keyof Authored = Exclude<\n    Exclude<keyof Authored, keyof Allowed>,\n    typeof globalThis extends { __VLS_PROPS_FALLBACK: infer P; __vizeTemplateChecker?: never }\n      ? keyof P\n      : never\n  >,\n> = K extends unknown\n";
const F3_AFTER_0: &str = "  Allowed,\n  K extends keyof Authored = Exclude<keyof Authored, keyof Allowed>,\n> = K extends unknown\n";
const F3_BEFORE_1: &str = "type $AuthoredKeyWitness<T> = { [K in keyof T]: unknown };\ntype $IsUnknown<T> = unknown extends T ? ([keyof T] extends [never] ? true : false) : false;\n";
const F3_AFTER_1: &str = "type $AuthoredKeyWitness<T> = { [K in keyof T]: unknown };\n// Constructor arguments (templates, `new`) bind `Authored`; TSX binds `Jsx` through `$props` (#11629).\ntype $ComponentInstanceFor<I, T, Declared, Authored, Jsx> =\n  $IsAny<Authored> extends true\n    ? I\n    : $IsUnknown<Authored> extends true\n      ? $UsePublicInstance<Jsx> extends true\n        ? I\n        : $ReplaceComponentProps<\n            I,\n            $AuthoredKeyWitness<Jsx> &\n              $ComponentInputProps<T, Declared> &\n              $ComponentInputGuard<T, Declared, Jsx>\n          >\n      : $ReplaceComponentProps<\n          I,\n          $ComponentInputProps<T, Declared> & $ComponentMissingInputGuard<T, Authored>\n        >;\ntype $IsUnknown<T> = unknown extends T ? ([keyof T] extends [never] ? true : false) : false;\n";
const F3_BEFORE_2: &str = "  $AuthoredKeyWitness,\n  $IsUnknown,\n";
const F3_AFTER_2: &str = "  $AuthoredKeyWitness,\n  $ComponentInstanceFor,\n  $IsUnknown,\n";
const F4_BEFORE_0: &str = "  $AuthoredKeyWitness,\n  $IsUnknown,\n";
const F4_AFTER_0: &str = "  $AuthoredKeyWitness,\n  $ComponentInstanceFor,\n  $IsUnknown,\n";
const PREAMBLE_BEFORE_BYTES: usize = 44_748;
const PREAMBLE_BEFORE_SHA: &str =
    "08ebeeb3d5303bd130213edd33583849ef8e078d33c0e366562d40efd3a9985d";
const PREAMBLE_AFTER_SHA: &str = "6deae073a61ccf9fd622bb398200aa33afe890d3bd0e68b77023e550dc4995e7";

struct File {
    path: &'static str,
    edits: &'static [(&'static str, &'static str)],
    before_bytes: usize,
    before_sha: &'static str,
    after_sha: &'static str,
}

const PREAMBLE_EDITS: &[(&str, &str)] = &[
    (F0_BEFORE_0, F0_AFTER_0),
    (F0_BEFORE_1, F0_AFTER_1),
    (F1_BEFORE_0, F1_AFTER_0),
];

const FILES: [File; 5] = [
    File {
        path: "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt",
        edits: &[(F0_BEFORE_0, F0_AFTER_0), (F0_BEFORE_1, F0_AFTER_1)],
        before_bytes: 5367,
        before_sha: "3371eb7d3919662cbfde16a142a58c3a36739aefa127daa706307ce578808cab",
        after_sha: "19a4efd540f8f23954f5eaaf2a42a8ea73c8ae05077bee6f17ef7db532e6f8e0",
    },
    File {
        path: "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-header.d.ts.txt",
        edits: &[(F1_BEFORE_0, F1_AFTER_0)],
        before_bytes: 1133,
        before_sha: "0ffa4d0f7c368167cf5072cbf63fcd077b0595d39fff43969f341fa9759d10b2",
        after_sha: "1c0fc02eb692116e0b06156b95cfc5e7d1b6e6d023d24bb21ab09c993937bbba",
    },
    File {
        path: "npm/cli/virtual-ts-globals.d.ts",
        edits: &[(F2_BEFORE_0, F2_AFTER_0), (F2_BEFORE_1, F2_AFTER_1)],
        before_bytes: 18877,
        before_sha: "5e8c28f2eec18eb6d55412c05f95c9333d464c542bc5b94f8681bcb3944349d3",
        after_sha: "b2eada86a579cce97cdf1caec84c8329c746603bbaa4e8709b9ba2ad7b5fa890",
    },
    File {
        path: "npm/cli/generated/content-mapper-types.d.ts",
        edits: &[
            (F3_BEFORE_0, F3_AFTER_0),
            (F3_BEFORE_1, F3_AFTER_1),
            (F3_BEFORE_2, F3_AFTER_2),
        ],
        before_bytes: 11420,
        before_sha: "8fff1e27182013fa69009b04ea502394ee4193081c8e1ff966aeb4cbc5b16215",
        after_sha: "f9f530ba3b0471c39512f9f7b503658ebec642c01dc63552d9778ef7aa974219",
    },
    File {
        path: "crates/vize_canon/src/virtual_ts/helpers/declarations/content-mapper-type-imports.d.ts.txt",
        edits: &[(F4_BEFORE_0, F4_AFTER_0)],
        before_bytes: 1213,
        before_sha: "142da5fca98318c5bd0511022bedccaa4166a2e8d626c92f20dd105d89e942c1",
        after_sha: "24d7be5cadd447d3a8935e213258dd10161ccd112f585b9b3790bffdd77e646d",
    },
];

fn delta(edits: &[(&str, &str)]) -> isize {
    edits
        .iter()
        .map(|(before, after)| after.len() as isize - before.len() as isize)
        .sum()
}

fn invert(
    current: &str,
    edits: &[(&str, &str)],
    before_bytes: usize,
    before_sha: &str,
    after_sha: &str,
) -> String {
    assert_eq!(current.len() as isize, before_bytes as isize + delta(edits));
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

pub(super) fn source_edits_before(path: &str, mut current: String) -> String {
    let Some(file) = FILES.iter().find(|file| file.path == path) else {
        return current;
    };
    for (before, after) in file.edits {
        assert_eq!(current.matches(after).count(), 1);
        current = current.replacen(after, before, 1);
    }
    current
}

pub(super) fn preamble_edits_before(mut current: String) -> String {
    for (before, after) in PREAMBLE_EDITS {
        assert_eq!(current.matches(after).count(), 1);
        current = current.replacen(after, before, 1);
    }
    current
}

pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::props_fallback_segment::source_before(path, current);
    let current = super::jsx_exact_keys::source_before(path, current);
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
    let current = super::jsx_exact_keys::preamble_before(current);
    invert(
        &current,
        PREAMBLE_EDITS,
        PREAMBLE_BEFORE_BYTES,
        PREAMBLE_BEFORE_SHA,
        PREAMBLE_AFTER_SHA,
    )
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    super::jsx_exact_keys::retain_provenance(manifest);
    let files = FILES
        .iter()
        .map(|file| {
            json!({ "path": file.path, "beforeBytes": file.before_bytes,
                "afterBytes": file.before_bytes as isize + delta(file.edits),
                "beforeSha256": file.before_sha, "afterSha256": file.after_sha })
        })
        .collect::<Vec<_>>();
    manifest["provenance"]["currentInputChannelsUpdate"] = json!({
        "kind": "exact-current-input-channels-inverse",
        "issue": 11629,
        "nativeAcceptanceCredit": false,
        "files": files,
        "compiledPreamble": { "path": "vize_canon::virtual_ts::SHARED_PREAMBLE_DTS",
            "beforeBytes": PREAMBLE_BEFORE_BYTES,
            "afterBytes": PREAMBLE_BEFORE_BYTES as isize + delta(PREAMBLE_EDITS),
            "beforeSha256": PREAMBLE_BEFORE_SHA, "afterSha256": PREAMBLE_AFTER_SHA }
    });
}

#[cfg(test)]
mod tests {
    use super::{
        FILES, PREAMBLE_EDITS, preamble_edits_before, retain_provenance, source_edits_before,
    };
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
    fn current_assets_preamble_and_sdk_reconstruct_only_the_input_channels_layer() {
        for file in &FILES {
            let body =
                super::super::props_fallback_segment::source_before(file.path, current(file.path));
            let body = super::super::strict_extra_keys::source_before(file.path, body);
            let body = super::super::integrated_source::repair_before(file.path, body);
            let layer = super::super::jsx_exact_keys::source_edits_before(file.path, body);
            assert_eq!(
                redo(source_edits_before(file.path, layer.clone()), file.edits),
                layer
            );
        }
        let preamble = super::super::integrated_source::repaired_preamble_before(
            vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
        );
        let layer = super::super::jsx_exact_keys::preamble_edits_before(preamble);
        assert_eq!(
            redo(preamble_edits_before(layer.clone()), PREAMBLE_EDITS),
            layer
        );
        let other = "crates/vize_canon/src/virtual_ts/helpers/declarations/core-types.d.ts.txt";
        assert_eq!(source_edits_before(other, current(other)), current(other));
        let mut manifest = json!({ "provenance": { "existing": "unchanged" } });
        retain_provenance(&mut manifest);
        assert_eq!(manifest["provenance"]["existing"], "unchanged");
    }

    #[test]
    fn missing_duplicate_stale_and_extra_bytes_refuse() {
        for file in &FILES {
            let body =
                super::super::props_fallback_segment::source_before(file.path, current(file.path));
            let body = super::super::strict_extra_keys::source_before(file.path, body);
            let body = super::super::integrated_source::repair_before(file.path, body);
            let body = super::super::jsx_exact_keys::source_edits_before(file.path, body);
            let (before, after) = file.edits[0];
            for mutated in [
                body.replacen(after, "", 1),
                format!("{body}{after}"),
                body.replacen(after, before, 1),
                format!("{body}\n"),
            ] {
                assert!(
                    std::panic::catch_unwind(|| source_edits_before(file.path, mutated.clone()))
                        .is_err()
                        || std::panic::catch_unwind(|| {
                            super::super::integrated_source::source_before(
                                file.path,
                                mutated.clone(),
                            )
                        })
                        .is_err()
                );
            }
        }
    }
}

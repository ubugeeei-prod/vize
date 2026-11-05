//! Reconstruct the exact main source beneath the reviewed #11964 join.
use super::support;
use serde_json::{json, Value};

struct File {
    path: &'static str,
    current_bytes: usize,
    current_sha: &'static str,
    base_bytes: usize,
    base_sha: &'static str,
}

struct RepairFile {
    path: &'static str,
    before_bytes: usize,
    before_sha: &'static str,
    after_bytes: usize,
    after_sha: &'static str,
    before: &'static str,
    after: &'static str,
}

const COMPONENT_INSTANCE_BEFORE: &str = "$ReplaceComponentProps<I, Pick<$AuthoredKeyWitness<Jsx>, keyof Jsx & keyof $ComponentInputProps<T, Declared>> & $ComponentInputProps<T, Declared> & $ComponentMissingInputGuard<T, Jsx>>";
const COMPONENT_INSTANCE_AFTER: &str = "$ReplaceComponentProps<I, T extends unknown ? Pick<$AuthoredKeyWitness<Jsx>, keyof Jsx & keyof $ComponentInputProps<T, Declared>> & $ComponentInputProps<T, Declared> & $ComponentMissingInputGuard<T, Jsx> : never>";
const GENERATED_INSTANCE_BEFORE: &str = concat!(
    "            Pick<$AuthoredKeyWitness<Jsx>, keyof Jsx & keyof $ComponentInputProps<T, Declared>> &\n",
    "              $ComponentInputProps<T, Declared> &\n",
    "              $ComponentMissingInputGuard<T, Jsx>"
);
const GENERATED_INSTANCE_AFTER: &str = concat!(
    "            T extends unknown\n",
    "              ? Pick<\n",
    "                  $AuthoredKeyWitness<Jsx>,\n",
    "                  keyof Jsx & keyof $ComponentInputProps<T, Declared>\n",
    "                > &\n",
    "                  $ComponentInputProps<T, Declared> &\n",
    "                  $ComponentMissingInputGuard<T, Jsx>\n",
    "              : never"
);
const GLOBAL_INSTANCE_BEFORE: &str = concat!(
    "              Pick<$AuthoredKeyWitness<Jsx>, keyof Jsx & keyof $ComponentInputProps<T, Declared>> &\n",
    "                $ComponentInputProps<T, Declared> &\n",
    "                $ComponentMissingInputGuard<T, Jsx>"
);
const GLOBAL_INSTANCE_AFTER: &str = concat!(
    "              T extends unknown\n",
    "                ? Pick<\n",
    "                    $AuthoredKeyWitness<Jsx>,\n",
    "                    keyof Jsx & keyof $ComponentInputProps<T, Declared>\n",
    "                  > &\n",
    "                    $ComponentInputProps<T, Declared> &\n",
    "                    $ComponentMissingInputGuard<T, Jsx>\n",
    "                : never"
);
const COMPONENT_INSTANCE_SUCCESSOR_AFTER: &str = "$ReplaceComponentProps<I, Pick<$AuthoredKeyWitness<Jsx>, keyof Jsx & $ComponentMetadataKeys<$ComponentInputProps<T, Declared>>> & (T extends unknown ? $ComponentInputProps<T, Declared> & $ComponentInputGuard<T, Declared, Jsx> : never)>";
const GENERATED_INSTANCE_SUCCESSOR_AFTER: &str = concat!(
    "            Pick<\n",
    "              $AuthoredKeyWitness<Jsx>,\n",
    "              keyof Jsx & $ComponentMetadataKeys<$ComponentInputProps<T, Declared>>\n",
    "            > &\n",
    "              (T extends unknown\n",
    "                ? $ComponentInputProps<T, Declared> & $ComponentInputGuard<T, Declared, Jsx>\n",
    "                : never)"
);
const GLOBAL_INSTANCE_SUCCESSOR_AFTER: &str = concat!(
    "              Pick<\n",
    "                $AuthoredKeyWitness<Jsx>,\n",
    "                keyof Jsx & $ComponentMetadataKeys<$ComponentInputProps<T, Declared>>\n",
    "              > &\n",
    "                (T extends unknown\n",
    "                  ? $ComponentInputProps<T, Declared> & $ComponentInputGuard<T, Declared, Jsx>\n",
    "                  : never)"
);

const REPAIRS: [RepairFile; 3] = [
    RepairFile {
        path: "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt",
        before_bytes: 5819,
        before_sha: "5a2b00dd75e8e6ade9a79db43b84b20532d3125514f0d1a4c5c6314f97591d76",
        after_bytes: 5847,
        after_sha: "989c87b3f04ffab6a0b8be1c4249bb38d98dd383a6199a90522c18cf7cb13f0d",
        before: COMPONENT_INSTANCE_BEFORE,
        after: COMPONENT_INSTANCE_AFTER,
    },
    RepairFile {
        path: "npm/cli/generated/content-mapper-types.d.ts",
        before_bytes: 12005,
        before_sha: "ef6af6da1991dddf75504f7b5e4743127d7fd0dae8c7085dd385f7d228844c6c",
        after_bytes: 12123,
        after_sha: "a0b8256f90095ea7e6f16f8e9e154d63c31f138dcd47e35fbfbacb4dc1be1a0a",
        before: GENERATED_INSTANCE_BEFORE,
        after: GENERATED_INSTANCE_AFTER,
    },
    RepairFile {
        path: "npm/cli/virtual-ts-globals.d.ts",
        before_bytes: 17057,
        before_sha: "fcb7574e88374a8de27f330be3733e486d1be465f372122965ea1302f34512b8",
        after_bytes: 17185,
        after_sha: "ad293c067fbdb952343569882f0076e12ec671a8ccc9473cf5c31c171e02bba4",
        before: GLOBAL_INSTANCE_BEFORE,
        after: GLOBAL_INSTANCE_AFTER,
    },
];

const SUCCESSOR_REPAIRS: [RepairFile; 3] = [
    RepairFile {
        path: "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt",
        before_bytes: 5847,
        before_sha: "989c87b3f04ffab6a0b8be1c4249bb38d98dd383a6199a90522c18cf7cb13f0d",
        after_bytes: 5870,
        after_sha: "3a3c9fb284feba5cdd7fc639ae1b99d19c31e5cb344339968acd553734adff53",
        before: COMPONENT_INSTANCE_AFTER,
        after: COMPONENT_INSTANCE_SUCCESSOR_AFTER,
    },
    RepairFile {
        path: "npm/cli/generated/content-mapper-types.d.ts",
        before_bytes: 12123,
        before_sha: "a0b8256f90095ea7e6f16f8e9e154d63c31f138dcd47e35fbfbacb4dc1be1a0a",
        after_bytes: 12116,
        after_sha: "c62f373226b2967df18826a699d7116605020149093885a4bb0bcfed41cf725c",
        before: GENERATED_INSTANCE_AFTER,
        after: GENERATED_INSTANCE_SUCCESSOR_AFTER,
    },
    RepairFile {
        path: "npm/cli/virtual-ts-globals.d.ts",
        before_bytes: 17185,
        before_sha: "ad293c067fbdb952343569882f0076e12ec671a8ccc9473cf5c31c171e02bba4",
        after_bytes: 17176,
        after_sha: "a92231c9b924778d38ba10c25d7908d1f3a70e1516aad96f8a653fe46e11977b",
        before: GLOBAL_INSTANCE_AFTER,
        after: GLOBAL_INSTANCE_SUCCESSOR_AFTER,
    },
];

const FILES: [File; 13] = [
    File {
        path: "npm/cli/virtual-ts.d.ts",
        current_bytes: 15058,
        current_sha: "c8761ce60abafd2b898506bfa80ee5c0c9de6dff7adcf4e5cccbf09a03717597",
        base_bytes: 15549,
        base_sha: "363aa2912d43f63c940a6af5f0eae6bc75dabac8cd97752fda1031b84c409f5e",
    },
    File {
        path: "npm/cli/virtual-ts.d.mts",
        current_bytes: 15058,
        current_sha: "c8761ce60abafd2b898506bfa80ee5c0c9de6dff7adcf4e5cccbf09a03717597",
        base_bytes: 15549,
        base_sha: "363aa2912d43f63c940a6af5f0eae6bc75dabac8cd97752fda1031b84c409f5e",
    },
    File {
        path: "npm/cli/virtual-ts.d.cts",
        current_bytes: 15058,
        current_sha: "c8761ce60abafd2b898506bfa80ee5c0c9de6dff7adcf4e5cccbf09a03717597",
        base_bytes: 15549,
        base_sha: "363aa2912d43f63c940a6af5f0eae6bc75dabac8cd97752fda1031b84c409f5e",
    },
    File {
        path: "npm/cli/virtual-ts-globals.d.ts",
        current_bytes: 17057,
        current_sha: "fcb7574e88374a8de27f330be3733e486d1be465f372122965ea1302f34512b8",
        base_bytes: 18877,
        base_sha: "b75a5ae2d5e020e819088ebc68eaf0a8cd9a5b57de900b700cd3759b936a9fb0",
    },
    File {
        path: "npm/cli/generated/content-mapper-types.d.ts",
        current_bytes: 12005,
        current_sha: "ef6af6da1991dddf75504f7b5e4743127d7fd0dae8c7085dd385f7d228844c6c",
        base_bytes: 11420,
        base_sha: "1acd0abc0a47eeea5d3a6bbe6a43ace358d394c29620d8230bb1cdc01cf701f1",
    },
    File {
        path: "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt",
        current_bytes: 5819,
        current_sha: "5a2b00dd75e8e6ade9a79db43b84b20532d3125514f0d1a4c5c6314f97591d76",
        base_bytes: 5367,
        base_sha: "3371eb7d3919662cbfde16a142a58c3a36739aefa127daa706307ce578808cab",
    },
    File {
        path: "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-header.d.ts.txt",
        current_bytes: 967,
        current_sha: "1c0fc02eb692116e0b06156b95cfc5e7d1b6e6d023d24bb21ab09c993937bbba",
        base_bytes: 1133,
        base_sha: "0ffa4d0f7c368167cf5072cbf63fcd077b0595d39fff43969f341fa9759d10b2",
    },
    File {
        path: "crates/vize_canon/src/virtual_ts/helpers/declarations/content-mapper-type-imports.d.ts.txt",
        current_bytes: 1238,
        current_sha: "24d7be5cadd447d3a8935e213258dd10161ccd112f585b9b3790bffdd77e646d",
        base_bytes: 1213,
        base_sha: "142da5fca98318c5bd0511022bedccaa4166a2e8d626c92f20dd105d89e942c1",
    },
    File {
        path: "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-inference.d.ts.txt",
        current_bytes: 2775,
        current_sha: "82e9767d55f20b032024840ae4ace6e9bbc72b8419362876593a5dd174a6505a",
        base_bytes: 2593,
        base_sha: "e8266b8848296e2786cf2f527eda9b1aa58bf314e4d5347e3f6d96e6315edd2c",
    },
    File {
        path: "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-emit-inference.d.ts.txt",
        current_bytes: 2846,
        current_sha: "d6daefb4cb7cf6d7b8a17b55d487e555e9c3ee6cf4d9b66c7f521a463dc77106",
        base_bytes: 2662,
        base_sha: "def7e4807e8f598098478e89184e65c0e805daa4e0e4d304808e62455a3e5e66",
    },
    File {
        path: "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt",
        current_bytes: 2535,
        current_sha: "924bb683376f6c2ab3b9225cd6eff73328f8dfe4e87529a1c1a8068c03923e74",
        base_bytes: 1883,
        base_sha: "ef2a99a2e675a810472b7a3cc6c9a4c9a905f12369aa1366dc9605af94b9c6f5",
    },
    File {
        path: "npm/cli/scripts/generate-content-mapper-types.ts",
        current_bytes: 18554,
        current_sha: "9df0e4ea2c0f466c3f717192102e3b6f14350540987d9dc3bc573111823a13c4",
        base_bytes: 17820,
        base_sha: "a32212a7833a6fcc7aa8998c0c9489dcdde895fb482a2143dad2b0708f932503",
    },
    File {
        path: "npm/cli/package.json",
        current_bytes: 4240,
        current_sha: "cb84f2c91b84e60fbec19e3745e4fad6b8e0a35f821f54ea9b9d6e5d8e3478d8",
        base_bytes: 4173,
        base_sha: "b6b57a0dd4222983f31bcd98c3a78455e58d03d1a2b175be6e240f032702fc7e",
    },
];

fn authenticate(text: &str, bytes: usize, sha: &str) {
    assert_eq!(text.len(), bytes);
    assert_eq!(support::hash(text.as_bytes()), sha);
}

fn apply_repair_before(path: &str, current: String, repairs: &[RepairFile]) -> String {
    let Some(repair) = repairs.iter().find(|repair| repair.path == path) else {
        return current;
    };
    authenticate(&current, repair.after_bytes, repair.after_sha);
    assert_eq!(current.matches(repair.after).count(), 1);
    let previous = current.replacen(repair.after, repair.before, 1);
    authenticate(&previous, repair.before_bytes, repair.before_sha);
    previous
}

pub(super) fn repair_before(path: &str, current: String) -> String {
    let current = apply_repair_before(path, current, &SUCCESSOR_REPAIRS);
    apply_repair_before(path, current, &REPAIRS)
}

pub(super) fn repaired_preamble_before(current: &str) -> String {
    authenticate(
        current,
        46_557,
        "635acca0b25877f2132902fe777d894287100a7c993066066a67812deab2f255",
    );
    assert_eq!(
        current.matches(COMPONENT_INSTANCE_SUCCESSOR_AFTER).count(),
        1
    );
    let current = current.replacen(
        COMPONENT_INSTANCE_SUCCESSOR_AFTER,
        COMPONENT_INSTANCE_AFTER,
        1,
    );
    authenticate(
        &current,
        46_534,
        "7be160e1798f4368642597b67db232b1a72309dc18eb637a4ec7d868f3457bcb",
    );
    assert_eq!(current.matches(COMPONENT_INSTANCE_AFTER).count(), 1);
    let previous = current.replacen(COMPONENT_INSTANCE_AFTER, COMPONENT_INSTANCE_BEFORE, 1);
    authenticate(
        &previous,
        46_506,
        "f3f628ff406ce57acce118f1213e000930f8a70d20afdb07fbfedc6e274b32e9",
    );
    previous
}

pub(super) fn overload_before(path: &str, mut current: String) -> String {
    let (before, after) = match path {
        "npm/cli/virtual-ts.d.ts" | "npm/cli/virtual-ts.d.mts" | "npm/cli/virtual-ts.d.cts" => (
            "",
            "import type {} from \"./virtual-ts-emit-overloads.js\" with { \"resolution-mode\": \"import\" };\n",
        ),
        "npm/cli/scripts/generate-content-mapper-types.ts" => (
            super::emit_overload_window::GENERATOR_BEFORE,
            super::emit_overload_window::GENERATOR_AFTER,
        ),
        "npm/cli/package.json" => (
            super::emit_overload_window::MANIFEST_BEFORE,
            super::emit_overload_window::MANIFEST_AFTER,
        ),
        "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt" => (
            super::emit_overload_window::ASSET_BEFORE,
            super::emit_overload_window::ASSET_AFTER,
        ),
        "npm/cli/virtual-ts-globals.d.ts" => (
            super::emit_overload_window::GLOBALS_BEFORE,
            super::emit_overload_window::GLOBALS_AFTER,
        ),
        _ => return current,
    };
    assert_eq!(current.matches(after).count(), 1);
    current = current.replacen(after, before, 1);
    current
}

pub(super) fn preamble_before_input_and_overload(current: &str) -> String {
    let current = repaired_preamble_before(current);
    let current = super::jsx_exact_keys::preamble_edits_before(current);
    let current = super::input_channels::preamble_edits_before(current);
    overload_before(
        "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt",
        current,
    )
}

pub(super) fn source_before(path: &str, current: String) -> String {
    let Some(file) = FILES.iter().find(|file| file.path == path) else {
        return current;
    };
    let current = repair_before(path, current);
    authenticate(&current, file.current_bytes, file.current_sha);
    let previous = match path {
        "npm/cli/virtual-ts.d.ts" | "npm/cli/virtual-ts.d.mts" | "npm/cli/virtual-ts.d.cts" => {
            const OVERLOAD_IMPORT: &str = "import type {} from \"./virtual-ts-emit-overloads.js\" with { \"resolution-mode\": \"import\" };\n";
            assert_eq!(current.matches(OVERLOAD_IMPORT).count(), 1);
            let previous = current.replacen(OVERLOAD_IMPORT, "", 1);
            super::loose_emit_sdk::source_before(path, previous)
        }
        "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-inference.d.ts.txt"
        | "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-emit-inference.d.ts.txt" => {
            super::generic_event_specialization::source_before(path, current)
        }
        "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt" => {
            assert_eq!(
                current
                    .matches(super::emit_overload_window::ASSET_AFTER)
                    .count(),
                1
            );
            current.replacen(
                super::emit_overload_window::ASSET_AFTER,
                super::emit_overload_window::ASSET_BEFORE,
                1,
            )
        }
        "npm/cli/scripts/generate-content-mapper-types.ts" | "npm/cli/package.json" => {
            let (before, after) = if path.ends_with("package.json") {
                (
                    super::emit_overload_window::MANIFEST_BEFORE,
                    super::emit_overload_window::MANIFEST_AFTER,
                )
            } else {
                (
                    super::emit_overload_window::GENERATOR_BEFORE,
                    super::emit_overload_window::GENERATOR_AFTER,
                )
            };
            let mut previous = current;
            assert_eq!(previous.matches(after).count(), 1);
            previous = previous.replacen(after, before, 1);
            super::loose_emit_sdk::source_edits_before(path, previous)
        }
        _ => {
            let previous = super::jsx_exact_keys::source_edits_before(path, current);
            let mut previous = super::input_channels::source_edits_before(path, previous);
            if path == "npm/cli/virtual-ts-globals.d.ts" {
                assert_eq!(
                    previous
                        .matches(super::emit_overload_window::GLOBALS_AFTER)
                        .count(),
                    1
                );
                previous = previous.replacen(
                    super::emit_overload_window::GLOBALS_AFTER,
                    super::emit_overload_window::GLOBALS_BEFORE,
                    1,
                );
            }
            previous
        }
    };
    authenticate(&previous, file.base_bytes, file.base_sha);
    previous
}

pub(super) fn preamble_before(current: &str) -> String {
    let current = repaired_preamble_before(current);
    authenticate(
        &current,
        46_506,
        "f3f628ff406ce57acce118f1213e000930f8a70d20afdb07fbfedc6e274b32e9",
    );
    let current = super::jsx_exact_keys::preamble_edits_before(current);
    let mut current = super::input_channels::preamble_edits_before(current);
    assert_eq!(
        current
            .matches(super::emit_overload_window::ASSET_AFTER)
            .count(),
        1
    );
    current = current.replacen(
        super::emit_overload_window::ASSET_AFTER,
        super::emit_overload_window::ASSET_BEFORE,
        1,
    );
    let previous = super::generic_event_specialization::preamble_edits_before(current);
    authenticate(
        &previous,
        45_384,
        "317bad0e932dd8054c21bc22e75ac319243448867bfbc40b4052ff7403e7e3e1",
    );
    previous
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["integratedSourceTransition11964"] = json!({
        "kind": "exact-integrated-source-inverse",
        "pullRequest": 11964,
        "baseSource": "e967ca6c76de2b12018e141f34fdf754a256ee37",
        "nativeAcceptanceCredit": false,
        "files": FILES.iter().map(|file| json!({
            "path": file.path,
            "beforeBytes": file.base_bytes,
            "beforeSha256": file.base_sha,
            "afterBytes": file.current_bytes,
            "afterSha256": file.current_sha,
        })).collect::<Vec<_>>(),
        "compiledPreamble": {
            "beforeBytes": 45384,
            "beforeSha256": "317bad0e932dd8054c21bc22e75ac319243448867bfbc40b4052ff7403e7e3e1",
            "afterBytes": 46506,
            "afterSha256": "f3f628ff406ce57acce118f1213e000930f8a70d20afdb07fbfedc6e274b32e9",
        },
    });
    manifest["provenance"]["discriminatedJsxUnionRepair11964"] = json!({
        "kind": "exact-source-finite-inverse",
        "pullRequest": 11964,
        "observedRegressionRun": 42086332303_i64,
        "nativeAcceptanceCredit": false,
        "files": REPAIRS.iter().map(|repair| json!({
            "path": repair.path,
            "beforeBytes": repair.before_bytes,
            "beforeSha256": repair.before_sha,
            "afterBytes": repair.after_bytes,
            "afterSha256": repair.after_sha,
        })).collect::<Vec<_>>(),
        "compiledPreamble": {
            "beforeBytes": 46506,
            "beforeSha256": "f3f628ff406ce57acce118f1213e000930f8a70d20afdb07fbfedc6e274b32e9",
            "afterBytes": 46534,
            "afterSha256": "7be160e1798f4368642597b67db232b1a72309dc18eb637a4ec7d868f3457bcb",
        },
    });
    manifest["provenance"]["discriminatedJsxUnionSuccessor11964"] = json!({
        "kind": "exact-source-finite-inverse",
        "pullRequest": 11964,
        "observedRegressionRun": 42094909495_i64,
        "nativeAcceptanceCredit": false,
        "files": SUCCESSOR_REPAIRS.iter().map(|repair| json!({
            "path": repair.path,
            "beforeBytes": repair.before_bytes,
            "beforeSha256": repair.before_sha,
            "afterBytes": repair.after_bytes,
            "afterSha256": repair.after_sha,
        })).collect::<Vec<_>>(),
        "compiledPreamble": {
            "beforeBytes": 46534,
            "beforeSha256": "7be160e1798f4368642597b67db232b1a72309dc18eb637a4ec7d868f3457bcb",
            "afterBytes": 46557,
            "afterSha256": "635acca0b25877f2132902fe777d894287100a7c993066066a67812deab2f255",
        },
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_integrated_source_reconstructs_the_merge_base() {
        for file in &FILES {
            let current = std::fs::read_to_string(support::repository().join(file.path)).unwrap();
            source_before(file.path, current);
        }
        preamble_before(vize_canon::virtual_ts::SHARED_PREAMBLE_DTS);
        assert_eq!(source_before("unrelated", "same".to_owned()), "same");
    }

    #[test]
    fn missing_duplicate_and_foreign_integrated_bytes_are_refused() {
        for file in &FILES {
            let current = std::fs::read_to_string(support::repository().join(file.path)).unwrap();
            for changed in [
                String::new(),
                format!("{current}{current}"),
                format!("{current}\n"),
            ] {
                assert!(std::panic::catch_unwind(|| source_before(file.path, changed)).is_err());
            }
        }
        let current = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
        assert!(std::panic::catch_unwind(|| preamble_before(&format!("{current}\n"))).is_err());
    }
}

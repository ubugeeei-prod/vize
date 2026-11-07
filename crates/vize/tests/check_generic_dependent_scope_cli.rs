//! #11450: a declaration that reads a generic alias stays in setup scope only.
#![cfg(test)]
use crate::support::corsa_requirement;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use vize_l0::String;

const CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-alias-dependent-scope-11450/tsconfig.json.txt"
));

const PROBE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-alias-dependent-scope-11450/Probe.vue.txt"
));

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
}

fn check(corsa: &Path, config: &str, source: &str) -> Vec<String> {
    let dir = tempfile::Builder::new()
        .prefix("generic-dependent-11450-")
        .tempdir()
        .unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("tsconfig.json"), config).unwrap();
    fs::write(dir.path().join("src/Probe.vue"), source).unwrap();
    let provider = root().join("tests/node_modules");
    assert!(
        provider.join("vue/package.json").is_file(),
        "Vue provider required"
    );
    #[cfg(unix)]
    std::os::unix::fs::symlink(provider, dir.path().join("node_modules")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(provider, dir.path().join("node_modules")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(dir.path())
        .env("CORSA_PATH", corsa)
        .args(["check", "--format", "json", "src/Probe.vue"])
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
        panic!(
            "{}\n{}",
            std::str::from_utf8(&output.stdout).unwrap(),
            std::str::from_utf8(&output.stderr).unwrap()
        )
    });
    let diagnostics: Vec<String> = report["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| file["diagnostics"].as_array().unwrap())
        .map(|row| row.as_str().unwrap().into())
        .collect();
    assert_eq!(
        output.status.code(),
        Some(if diagnostics.is_empty() { 0 } else { 1 })
    );
    diagnostics
}

#[test]
fn generic_alias_dependents_are_not_unused_module_copies() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let unused = "error:4:11 [TS6196] 'Bar' is declared but never used.";
    let cases = [
        ("original", CONFIG.into(), PROBE.into(), vec![]),
        (
            "type-alias",
            CONFIG.into(),
            PROBE
                .replace("interface Bar {", "type Bar = {")
                .replace("  items: Foo[];\n}", "  items: Foo[];\n};"),
            vec![],
        ),
        (
            "transitive",
            CONFIG.into(),
            PROBE
                .replace("interface Bar", "type Baz = Foo[];\ninterface Bar")
                .replace("items: Foo[]", "items: Baz"),
            vec![],
        ),
        (
            "emits",
            CONFIG.into(),
            PROBE.replace(
                "defineProps<Bar>();",
                "defineEmits<{ pick: [bar: Bar] }>();",
            ),
            vec![],
        ),
        (
            "slots",
            CONFIG.into(),
            PROBE.replace(
                "defineProps<Bar>();",
                "defineSlots<{ default(props: { bar: Bar }): any }>();",
            ),
            vec![],
        ),
        (
            "module-alias-reads-dependent",
            CONFIG.into(),
            PROBE.replace(
                "defineProps<Bar>();",
                "type Qux = { bar: Bar; raw: TBaz };\ndefineProps<Qux>();",
            ),
            vec![],
        ),
        (
            "no-unused-locals-off",
            CONFIG.replace("\"noUnusedLocals\": true", "\"noUnusedLocals\": false"),
            PROBE.into(),
            vec![],
        ),
        (
            "authored-unused",
            CONFIG.into(),
            PROBE.replace("defineProps<Bar>();", "defineProps<{ value: Foo }>();"),
            vec![unused],
        ),
        (
            "authored-value-error",
            CONFIG.into(),
            PROBE.replace(
                "defineProps<Bar>();",
                "defineProps<Bar>();\nconst bad: number = \"x\";\nvoid bad;",
            ),
            vec!["error:8:7 [TS2322] Type 'string' is not assignable to type 'number'."],
        ),
        (
            "authored-export",
            CONFIG.into(),
            PROBE.replace("interface Bar {", "export interface Bar {"),
            vec![],
        ),
        (
            "authored-export-type-alias",
            CONFIG.into(),
            PROBE
                .replace("interface Bar {", "export type Bar = {")
                .replace("  items: Foo[];\n}", "  items: Foo[];\n};"),
            vec![],
        ),
        (
            "authored-export-own-type-parameter",
            CONFIG.into(),
            PROBE
                .replace(
                    "interface Bar {",
                    "export interface Bar<TItem extends Foo> {",
                )
                .replace("items: Foo[]", "items: TItem[]")
                .replace("defineProps<Bar>();", "defineProps<Bar<Foo>>();"),
            vec![],
        ),
        (
            "multiple-authored-exports",
            CONFIG.into(),
            PROBE
                .replace(
                    "interface Bar {",
                    "export type Extra = string;\nexport interface Bar {",
                )
                .replace(
                    "defineProps<Bar>();",
                    "defineProps<Bar & { extra: Extra }>();",
                ),
            vec![],
        ),
        (
            "authored-export-after-non-ascii",
            CONFIG.into(),
            PROBE.replace("interface Bar {", "/* 日本語 */ export interface Bar {"),
            vec![],
        ),
        (
            "authored-export-without-sfc-generics",
            CONFIG.into(),
            PROBE
                .replace(" generic=\"TBaz\"", "")
                .replace("TBaz", "string")
                .replace("interface Bar {", "export interface Bar {"),
            vec![],
        ),
    ];
    for (id, config, source, expected) in cases {
        let actual = check(&corsa, &config, &source);
        assert_eq!(actual, expected, "{id}");
    }
}

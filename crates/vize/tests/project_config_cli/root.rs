//! The public CLI follows Vite's selected project without changing explicit inputs.
#![expect(
    clippy::disallowed_types,
    reason = "complete fixture artifacts use std strings"
)]

use super::support::{assert_success, run, write};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

const SETTINGS: &str = include_str!(
    "../../../../tests/_fixtures/differential/config/vite-root-8371/vite.config.mjs.txt"
);
const APP: &str =
    include_str!("../../../../tests/_fixtures/differential/config/vite-root-8371/App.vue.txt");
const IMAGE: &str =
    include_str!("../../../../tests/_fixtures/differential/config/vite-root-8371/Image.vue.txt");
const TSCONFIG: &str = include_str!(
    "../../../../tests/_fixtures/differential/config/vite-root-8371/tsconfig.json.txt"
);
const ARRAY: &str = "export default {root:'app',vize:[{linter:{preset:'essential',rules:{'a11y/alt-text':'off'}}},{files:['src/**/*.vue'],ignores:['src/Ignored.vue'],linter:{rules:{'a11y/alt-text':'error'}}}]};";
const DEDICATED: &str = r#"{"linter":{"preset":"essential","rules":{"a11y/alt-text":"off"}}}"#;
const IGNORE_SETTINGS: &str = include_str!(
    "../../../../tests/_fixtures/differential/config/vite-root-8371/ignore.config.mjs.txt"
);

macro_rules! oracle {
    ($name:literal) => {
        serde_json::from_str::<Value>(include_str!(concat!(
            "../../../../tests/_fixtures/differential/config/vite-root-8371/oracles/",
            $name
        )))
        .unwrap()
    };
}

fn project() -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    for (name, source) in [
        ("package.json", "{\"type\":\"module\"}"),
        ("vite.config.mjs", SETTINGS),
        ("app/src/App.vue", APP),
        ("app/src/Image.vue", IMAGE),
        ("app/src/Ignored.vue", IMAGE),
        ("src/Decoy.vue", IMAGE),
        ("app/tsconfig.json", TSCONFIG),
        ("tsconfig.json", TSCONFIG),
        ("app/src/valid.ts", "export const count: number = 1;\n"),
        ("src/invalid.ts", "export const count: number = 'wrong';\n"),
    ] {
        write(root, name, source);
    }
    project
}

fn ignore_project() -> tempfile::TempDir {
    let project = project();
    let root = project.path();
    write(root, "vite.config.mjs", IGNORE_SETTINGS);
    for name in [
        "app/generated/DropItem.vue",
        "app/generated/KeepItem.vue",
        "app/src/[id].vue",
        "app/src/id.vue",
        "app/absolute/DropItem.vue",
        "app/absolute/KeepItem.vue",
    ] {
        write(root, name, APP);
    }
    for name in [
        "app/generated/DropItem.ts",
        "app/generated/KeepItem.ts",
        "app/src/[id].ts",
        "app/src/id.ts",
        "app/absolute/DropItem.ts",
        "app/absolute/KeepItem.ts",
    ] {
        write(root, name, "export const count: number = 'wrong';\n");
    }
    write(
        root,
        "app/tsconfig.json",
        &TSCONFIG.replace(
            "\"src/**/*.ts\"",
            "\"src/**/*.ts\", \"generated/**/*.ts\", \"absolute/**/*.ts\"",
        ),
    );
    project
}

// Compare every directory and complete file bytes. Empty extra directories,
// unrequested sibling outputs and dedicated configs are observable artifacts.
fn inventory(root: &Path) -> Value {
    fn visit(root: &Path, directory: &Path, result: &mut BTreeMap<String, Value>) {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let name = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let kind = entry.file_type().unwrap();
            let artifact = if kind.is_dir() {
                json!({"directory":true})
            } else if kind.is_symlink() {
                json!({"symlink":fs::read_link(&path).unwrap().to_string_lossy()})
            } else {
                json!({"text":fs::read_to_string(&path).unwrap()})
            };
            result.insert(name, artifact);
            if kind.is_dir() {
                visit(root, &path, result);
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    serde_json::to_value(result).unwrap()
}

// Retain every field, diagnostic, option and message. Only the temporary
// project identity differs between runs; path separator spelling is portable.
fn normalize_report(root: &Path, value: &mut Value) {
    match value {
        Value::String(value) => {
            *value = value
                .replace(
                    fs::canonicalize(root).unwrap().to_str().unwrap(),
                    "<project>",
                )
                .replace(root.to_str().unwrap(), "<project>")
                .replace('\\', "/");
        }
        Value::Array(values) => {
            for value in values {
                normalize_report(root, value);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                normalize_report(root, value);
            }
        }
        _ => {}
    }
}

fn reset_evaluations(root: &Path) {
    let _ = fs::remove_file(root.join(".evaluations"));
}

fn assert_configuration(root: &Path, vite: &str, dedicated: bool, evaluation: Option<&str>) {
    let filenames = [
        "vize.config.pkl",
        "vize.config.ts",
        "vize.config.js",
        "vize.config.mjs",
        "vize.config.json",
    ];
    let actual: Vec<_> = filenames
        .into_iter()
        .map(|name| (name, root.join(name).is_file()))
        .collect();
    assert_eq!(
        actual,
        vec![
            ("vize.config.pkl", false),
            ("vize.config.ts", false),
            ("vize.config.js", false),
            ("vize.config.mjs", false),
            ("vize.config.json", dedicated)
        ]
    );
    assert_eq!(
        fs::read_to_string(root.join("vite.config.mjs")).unwrap(),
        vite
    );
    assert_eq!(
        fs::read_to_string(root.join(".evaluations"))
            .ok()
            .as_deref(),
        evaluation
    );
}

#[test]
fn omitted_build_inputs_use_vite_root_and_explicit_inputs_remain_local() {
    let project = project();
    let root = project.path();
    assert_success(&run(root, &["build", "-o", "configured"]));
    assert_eq!(
        inventory(&root.join("configured")),
        oracle!("build-configured.json")
    );
    assert_configuration(root, SETTINGS, false, Some("evaluated\n"));

    reset_evaluations(root);
    assert_success(&run(
        root,
        &[
            "build",
            "src/Decoy.vue",
            "-o",
            "explicit",
            "--config",
            "vite.config.mjs",
        ],
    ));
    assert_eq!(
        inventory(&root.join("explicit")),
        oracle!("build-explicit.json")
    );
    assert_configuration(root, SETTINGS, false, Some("evaluated\n"));

    reset_evaluations(root);
    assert_success(&run(root, &["build", "--no-config", "-o", "defaults"]));
    assert_eq!(
        inventory(&root.join("defaults")),
        oracle!("build-defaults.json")
    );
    assert_configuration(root, SETTINGS, false, None);
}

fn lint_report(root: &Path, args: &[&str]) -> (std::process::Output, Value) {
    let mut command = vec!["lint", "--format", "json"];
    command.extend_from_slice(args);
    let output = run(root, &command);
    let mut report: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {output:?}"));
    normalize_report(root, &mut report);
    // File enumeration is unordered; preserve every complete record and the
    // rule/message ordering inside each record.
    report
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a["file"].as_str().cmp(&b["file"].as_str()));
    (output, report)
}

#[test]
fn omitted_lint_inputs_scope_entry_rules_and_ignores_to_vite_root() {
    let project = project();
    let root = project.path();
    let (output, report) = lint_report(root, &[]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(report, oracle!("lint-configured.json"));
    assert_configuration(root, SETTINGS, false, Some("evaluated\n"));

    reset_evaluations(root);
    let (explicit, report) = lint_report(root, &["src/Decoy.vue", "--config", "vite.config.mjs"]);
    assert_eq!(explicit.status.code(), Some(1), "{explicit:?}");
    assert_eq!(report, oracle!("lint-explicit.json"));
    assert_configuration(root, SETTINGS, false, Some("evaluated\n"));

    reset_evaluations(root);
    let (defaults, report) = lint_report(root, &["--no-config"]);
    assert_eq!(defaults.status.code(), Some(0), "{defaults:?}");
    assert_eq!(report, oracle!("lint-defaults.json"));
    assert_configuration(root, SETTINGS, false, None);
}

#[cfg(feature = "glyph")]
#[test]
fn omitted_format_inputs_follow_vite_root_without_writing_siblings() {
    let project = project();
    let root = project.path();
    write(root, "src/Decoy.vue", APP);
    assert_success(&run(root, &["fmt", "--write"]));
    let mut expected = oracle!("fmt-configured.json");
    assert_eq!(inventory(root), expected);
    assert_configuration(root, SETTINGS, false, Some("evaluated\n"));

    reset_evaluations(root);
    assert_success(&run(
        root,
        &[
            "fmt",
            "--write",
            "src/Decoy.vue",
            "--config",
            "vite.config.mjs",
        ],
    ));
    expected["src/Decoy.vue"] = json!({"text":include_str!("../../../../tests/_fixtures/differential/config/vite-root-8371/oracles/fmt-explicit.vue.txt")});
    assert_eq!(inventory(root), expected);
    assert_configuration(root, SETTINGS, false, Some("evaluated\n"));

    reset_evaluations(root);
    let missing = run(root, &["fmt", "--write", "src/Image.vue"]);
    assert_eq!(missing.status.code(), Some(1), "{missing:?}");
    assert_eq!(inventory(root), expected);
    assert_configuration(root, SETTINGS, false, Some("evaluated\n"));

    reset_evaluations(root);
    write(root, "src/Decoy.vue", APP);
    assert_success(&run(
        root,
        &["fmt", "--write", "src/Decoy.vue", "--no-config"],
    ));
    expected["src/Decoy.vue"] = json!({"text":include_str!("../../../../tests/_fixtures/differential/config/vite-root-8371/oracles/fmt-default.vue.txt")});
    expected.as_object_mut().unwrap().remove(".evaluations");
    assert_eq!(inventory(root), expected);
    assert_configuration(root, SETTINGS, false, None);
}

#[test]
fn native_vite_arrays_keep_global_settings_and_root_relative_scoped_entries() {
    let project = project();
    let root = project.path();
    write(root, "vite.config.mjs", ARRAY);
    let (output, report) = lint_report(root, &[]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(report, oracle!("lint-array.json"));
    assert_configuration(root, ARRAY, false, None);
}

#[test]
fn vite_root_global_ignore_negation_and_escapes_reach_lint_discovery() {
    let project = ignore_project();
    let root = project.path();
    let (output, report) = lint_report(root, &[]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(report, oracle!("lint-ignore-syntax.json"));
    assert_configuration(root, IGNORE_SETTINGS, false, None);
}

#[cfg(feature = "glyph")]
#[test]
fn vite_root_global_ignore_negation_and_escapes_reach_formatter_discovery() {
    let project = ignore_project();
    let root = project.path();
    assert_success(&run(root, &["fmt", "--write"]));
    assert_eq!(inventory(root), oracle!("fmt-ignore-syntax.json"));
    assert_configuration(root, IGNORE_SETTINGS, false, None);
}

#[test]
fn vite_root_global_ignore_negation_and_escapes_reach_check_programs() {
    use super::{corsa_path, corsa_requirement, typecheck};
    use std::process::Command;
    let Some(corsa) =
        corsa_requirement::required_or_skip(corsa_path::resolve(typecheck::workspace_root()))
    else {
        return;
    };
    let project = ignore_project();
    let root = project.path();
    typecheck::link_vue(root);
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("CORSA_PATH", &corsa)
        .env("RAYON_NUM_THREADS", "1")
        .args(["check", "--format", "json"])
        .output()
        .unwrap();
    let mut report: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {output:?}"));
    normalize_report(root, &mut report);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(report, oracle!("check-ignore-syntax.json"));
    assert_configuration(root, IGNORE_SETTINGS, false, None);
}

#[test]
fn dedicated_configuration_keeps_the_existing_invocation_root() {
    let project = project();
    let root = project.path();
    write(root, "vize.config.json", DEDICATED);
    let (output, report) = lint_report(root, &[]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(report, oracle!("lint-dedicated.json"));
    assert_success(&run(root, &["build", "-o", "dedicated"]));
    assert_eq!(
        inventory(&root.join("dedicated")),
        oracle!("build-dedicated.json")
    );
    assert_eq!(
        fs::read_to_string(root.join("vize.config.json")).unwrap(),
        DEDICATED
    );
    assert_configuration(root, SETTINGS, true, None);
}

#[test]
fn omitted_check_inputs_use_vite_tsconfig_and_explicit_tsconfig_stays_local() {
    use super::{corsa_path, corsa_requirement, typecheck};
    use std::process::Command;

    let Some(corsa) =
        corsa_requirement::required_or_skip(corsa_path::resolve(typecheck::workspace_root()))
    else {
        return;
    };
    let project = project();
    let root = project.path();
    typecheck::link_vue(root);
    let check = |args: &[&str]| {
        reset_evaluations(root);
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(root)
            .env("CORSA_PATH", &corsa)
            .env("RAYON_NUM_THREADS", "1")
            .args(["check", "--format", "json"])
            .args(args)
            .output()
            .unwrap();
        let mut report: Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|error| panic!("{error}: {output:?}"));
        normalize_report(root, &mut report);
        (output, report)
    };
    let (output, report) = check(&[]);
    assert_success(&output);
    assert_eq!(report, oracle!("check-configured.json"));
    assert_configuration(root, SETTINGS, false, Some("evaluated\n"));

    let cases = [
        (
            &["--tsconfig", "tsconfig.json"][..],
            oracle!("check-tsconfig.json"),
            Some("evaluated\n"),
        ),
        (
            &["src/invalid.ts"][..],
            oracle!("check-explicit.json"),
            Some("evaluated\n"),
        ),
        (&["--no-config"][..], oracle!("check-defaults.json"), None),
    ];
    for (args, expected, evaluation) in cases {
        let (output, report) = check(args);
        assert_eq!(output.status.code(), Some(1), "{args:?}: {output:?}");
        assert_eq!(report, expected);
        assert_configuration(root, SETTINGS, false, evaluation);
    }
    write(
        root,
        "manual.json",
        r#"{"typeChecker":{"tsconfig":"tsconfig.json"}}"#,
    );
    let (output, report) = check(&["--config", "manual.json"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(report, oracle!("check-manual.json"));
    assert_eq!(
        fs::read_to_string(root.join("manual.json")).unwrap(),
        r#"{"typeChecker":{"tsconfig":"tsconfig.json"}}"#
    );
    assert_configuration(root, SETTINGS, false, None);
}

use super::super::support::{run, write};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

pub(super) const SETTINGS: &str = include_str!(
    "../../../../../tests/_fixtures/differential/config/vite-root-8371/vite.config.mjs.txt"
);
pub(super) const APP: &str =
    include_str!("../../../../../tests/_fixtures/differential/config/vite-root-8371/App.vue.txt");
pub(super) const IMAGE: &str =
    include_str!("../../../../../tests/_fixtures/differential/config/vite-root-8371/Image.vue.txt");
pub(super) const TSCONFIG: &str = include_str!(
    "../../../../../tests/_fixtures/differential/config/vite-root-8371/tsconfig.json.txt"
);
pub(super) const ARRAY: &str = "export default {root:'app',vize:[{__vizeProjectRoot:'decoy',linter:{preset:'essential',rules:{'a11y/alt-text':'off'}}},{files:['src/**/*.vue'],ignores:['src/Ignored.vue'],linter:{rules:{'a11y/alt-text':'error'}}}]};";
pub(super) const DEDICATED: &str =
    r#"{"linter":{"preset":"essential","rules":{"a11y/alt-text":"off"}}}"#;
pub(super) const IGNORE_SETTINGS: &str = include_str!(
    "../../../../../tests/_fixtures/differential/config/vite-root-8371/ignore.config.mjs.txt"
);

macro_rules! oracle {
    ($name:literal) => {
        serde_json::from_str::<Value>(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/config/vite-root-8371/oracles/",
            $name
        )))
        .unwrap()
    };
}

pub(super) fn project() -> tempfile::TempDir {
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

pub(super) fn ignore_project() -> tempfile::TempDir {
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
pub(super) fn inventory(root: &Path) -> Value {
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
// project identity differs between runs.
pub(super) fn normalize_report(root: &Path, value: &mut Value) {
    match value {
        Value::String(value) => {
            *value = value
                .replace(
                    fs::canonicalize(root).unwrap().to_str().unwrap(),
                    "<project>",
                )
                .replace(root.to_str().unwrap(), "<project>");
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

pub(super) fn reset_evaluations(root: &Path) {
    let _ = fs::remove_file(root.join(".evaluations"));
}

pub(super) fn assert_configuration(
    root: &Path,
    vite: &str,
    dedicated: bool,
    evaluation: Option<&str>,
) {
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

pub(super) fn lint_report(root: &Path, args: &[&str]) -> (std::process::Output, Value) {
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

pub(super) use oracle;

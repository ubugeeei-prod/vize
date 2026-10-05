#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/typechecker/undefined-default-prop/App.vue.txt"
);
const CONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/typechecker/undefined-default-prop/tsconfig.json.txt"
);

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
}

fn write(root: &Path, file: &str, source: &str) {
    let path = root.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}

fn fixture(root: &Path, source: &str) {
    write(root, "src/A.vue", source);
    write(root, "tsconfig.json", CONFIG);
}

fn case() -> tempfile::TempDir {
    let parent = workspace_root().join("npm/cli/target/vize-tests/undefined-default-props");
    std::fs::create_dir_all(&parent).unwrap();
    tempfile::tempdir_in(parent).unwrap()
}

fn check(root: &Path, corsa: &Path, expected: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("CORSA_PATH", corsa)
        .args(["check", "--no-config", "--format", "json"])
        .output()
        .unwrap();
    capture(root, corsa, "cli", &output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        output.status.success(),
        expected.is_empty(),
        "{stdout}\n{stderr}"
    );
    assert_eq!(stderr, "");
    let json: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], expected.len());
    assert_eq!(json["warningCount"], 0);
    let diagnostics = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|file| !file["diagnostics"].as_array().unwrap().is_empty())
        .map(|file| {
            (
                file["file"].as_str().unwrap().replace('\\', "/"),
                file["diagnostics"].clone(),
            )
        })
        .collect::<Vec<_>>();
    let expected = if expected.is_empty() {
        vec![]
    } else {
        vec![("src/A.vue", serde_json::json!(expected))]
    };
    assert_eq!(serde_json::json!(diagnostics), serde_json::json!(expected));
    json
}

fn native_macro_oracle(root: &Path, corsa: &Path, defaults: &str, template: &str, expected: &str) {
    let version = Command::new(corsa).arg("--version").output().unwrap();
    assert_eq!(
        String::from_utf8(version.stdout).unwrap().trim(),
        "Version 7.0.2"
    );
    write(
        root,
        "oracle.ts",
        &format!(
            "import {{ defineProps, withDefaults }} from 'vue';\nconst props = withDefaults(defineProps<{{ onLoad?: () => void }}>(), {defaults});\nconst onLoad = props.onLoad;\n{template}\n"
        ),
    );
    write(
        root,
        "oracle.tsconfig.json",
        r#"{"compilerOptions":{"strict":true,"module":"ESNext","moduleResolution":"Bundler","target":"ESNext","skipLibCheck":true,"noEmit":true,"types":[]},"files":["oracle.ts"]}"#,
    );
    let output = Command::new(corsa)
        .current_dir(root)
        .args(["-p", "oracle.tsconfig.json", "--pretty", "false"])
        .output()
        .unwrap();
    capture(root, corsa, "oracle", &output);
    assert_eq!(output.status.success(), expected.is_empty(), "{:?}", output);
    assert_eq!(
        String::from_utf8(output.stdout)
            .unwrap()
            .replace("\r\n", "\n"),
        expected
    );
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

fn capture(root: &Path, corsa: &Path, phase: &str, output: &Output) {
    let Some(capture) = std::env::var_os("VIZE_DEFAULT_PROP_CAPTURE") else {
        return;
    };
    let thread = std::thread::current();
    let capture = PathBuf::from(capture).join(thread.name().unwrap_or("unknown"));
    std::fs::create_dir_all(&capture).unwrap();
    std::fs::write(capture.join(format!("{phase}.stdout.txt")), &output.stdout).unwrap();
    std::fs::write(capture.join(format!("{phase}.stderr.txt")), &output.stderr).unwrap();
    std::fs::write(capture.join(format!("{phase}.json")), serde_json::json!({
        "exitCode": output.status.code(), "cliBinary": env!("CARGO_BIN_EXE_vize"), "nativeBinary": corsa
    }).to_string()).unwrap();
    for file in [
        "src/A.vue",
        "src/defaults.ts",
        "tsconfig.json",
        "oracle.ts",
        "oracle.tsconfig.json",
    ] {
        let source = root.join(file);
        if source.is_file() {
            let target = capture.join("inputs").join(file);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(source, target).unwrap();
        }
    }
}

#[test]
fn undefined_callable_default_keeps_original_cli_conditions_optional() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let case = case();
    fixture(case.path(), ORIGINAL);
    native_macro_oracle(
        case.path(),
        &corsa,
        "{ onLoad: undefined }",
        "if (onLoad) {}\nif (props.onLoad) {}",
        "",
    );
    check(case.path(), &corsa, &[]);
}

#[test]
fn imported_callable_default_remains_defined_in_all_template_prop_views() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let case = case();
    fixture(
        case.path(),
        r#"<script setup lang="ts">
import { defaults } from './defaults';
const props = withDefaults(defineProps<{ onLoad?: () => void }>(), defaults);
</script>
<template>
  <div>{{ onLoad() }}{{ props.onLoad() }}{{ $props.onLoad() }}</div>
</template>
"#,
    );
    write(
        case.path(),
        "src/defaults.ts",
        "export const defaults = { onLoad: () => {} };\n",
    );
    native_macro_oracle(
        case.path(),
        &corsa,
        "{ onLoad: () => {} }",
        "onLoad();\nprops.onLoad();",
        "",
    );
    check(case.path(), &corsa, &[]);
}

#[test]
fn undefined_callable_default_still_rejects_unguarded_template_calls() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let case = case();
    fixture(
        case.path(),
        r#"<script setup lang="ts">
const props = withDefaults(defineProps<{ onLoad?: () => void }>(), { onLoad: undefined });
</script>
<template>
  <div>{{ onLoad() }}</div>
  <div>{{ props.onLoad() }}</div>
</template>
"#,
    );
    native_macro_oracle(
        case.path(),
        &corsa,
        "{ onLoad: undefined }",
        "onLoad();\nprops.onLoad();",
        concat!(
            "oracle.ts(4,1): error TS2722: Cannot invoke an object which is possibly 'undefined'.\n",
            "oracle.ts(5,1): error TS2722: Cannot invoke an object which is possibly 'undefined'.\n"
        ),
    );
    check(
        case.path(),
        &corsa,
        &[
            "error:5:11 [TS2722] Cannot invoke an object which is possibly 'undefined'.",
            "error:6:11 [TS2722] Cannot invoke an object which is possibly 'undefined'.",
        ],
    );
}

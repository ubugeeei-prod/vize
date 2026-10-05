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
    "../../../tests/_fixtures/differential/typechecker/template-dollar-emit/App.vue.txt"
);
const CONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/typechecker/template-dollar-emit/tsconfig.json.txt"
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
    let parent = workspace_root().join("npm/cli/target/vize-tests/template-dollar-emit");
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
    assert_eq!(
        stderr,
        format!(
            "Building Corsa virtual project for 1 files under {}...\nRunning Corsa diagnostics for 1 files...\n",
            root.display()
        )
    );
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

fn native_emit_oracle(root: &Path, corsa: &Path, declaration: &str, calls: &str, expected: &str) {
    let version = Command::new(corsa).arg("--version").output().unwrap();
    assert_eq!(
        String::from_utf8(version.stdout).unwrap().trim(),
        "Version 7.0.2"
    );
    write(
        root,
        "oracle.ts",
        &format!("import {{ defineEmits, defineModel }} from 'vue';\n{declaration}\n{calls}\n"),
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
    assert_eq!(output.status.success(), expected.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout)
            .unwrap()
            .replace("\r\n", "\n"),
        expected
    );
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

fn capture(root: &Path, corsa: &Path, phase: &str, output: &Output) {
    let Some(capture) = std::env::var_os("VIZE_TEMPLATE_EMIT_CAPTURE") else {
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
        "src/events.ts",
        "nuxt.config.ts",
        ".nuxt/imports.d.ts",
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
fn original_unassigned_macro_rejects_complete_event_and_payload_vector() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let case = case();
    fixture(case.path(), ORIGINAL);
    native_emit_oracle(
        case.path(),
        &corsa,
        "const emit = defineEmits<{ click: []; change: [value: number] }>();",
        "emit('clik');\nemit('change', 'x');",
        concat!(
            "oracle.ts(3,6): error TS2345: Argument of type '\"clik\"' is not assignable to parameter of type '\"click\"'.\n",
            "oracle.ts(4,16): error TS2345: Argument of type 'string' is not assignable to parameter of type 'number'.\n"
        ),
    );
    check(
        case.path(),
        &corsa,
        &[
            "error:9:39 [TS2345] Argument of type '\"clik\"' is not assignable to parameter of type '\"click\"'.",
            "error:10:49 [TS2345] Argument of type 'string' is not assignable to parameter of type 'number'.",
        ],
    );
}

#[test]
fn unassigned_macro_accepts_declared_events_and_numeric_payloads() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let case = case();
    fixture(
        case.path(),
        &ORIGINAL.replace("'clik'", "'click'").replace("'x'", "1"),
    );
    native_emit_oracle(
        case.path(),
        &corsa,
        "const emit = defineEmits<{ click: []; change: [value: number] }>();",
        "emit('click');\nemit('change', 1);",
        "",
    );
    check(case.path(), &corsa, &[]);
}

#[test]
fn runtime_array_macro_keeps_event_names_without_inventing_payload_constraints() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let case = case();
    fixture(
        case.path(),
        r#"<script setup lang="ts">
defineEmits(['click', 'change']);
</script>
<template>
  <button @click="$emit('change', 'x')">ok</button>
  <button @click="$emit('clik')">bad</button>
</template>
"#,
    );
    native_emit_oracle(
        case.path(),
        &corsa,
        "const emit = defineEmits(['click', 'change']);",
        "emit('change', 'x');\nemit('clik');",
        "oracle.ts(4,6): error TS2345: Argument of type '\"clik\"' is not assignable to parameter of type '\"click\" | \"change\"'.\n",
    );
    check(
        case.path(),
        &corsa,
        &[
            "error:6:25 [TS2345] Argument of type '\"clik\"' is not assignable to parameter of type '\"click\" | \"change\"'.",
        ],
    );
}

#[test]
fn renamed_macro_result_and_local_call_signature_keep_payload_types() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let case = case();
    fixture(
        case.path(),
        r#"<script setup lang="ts">
type Events = { (event: 'change', value: number): void };
const dispatch = defineEmits<Events>();
</script>
<template>
  <button @click="$emit('change', 1); dispatch('change', 2)">ok</button>
</template>
"#,
    );
    native_emit_oracle(
        case.path(),
        &corsa,
        "type Events = { (event: 'change', value: number): void }; const emit = defineEmits<Events>();",
        "emit('change', 1);",
        "",
    );
    check(case.path(), &corsa, &[]);
}

#[test]
fn shadowed_macro_retains_the_public_instance_emit_contract() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let case = case();
    fixture(
        case.path(),
        r#"<script setup lang="ts">
function defineEmits<T>() { return (value: T) => value; }
const ordinary = defineEmits<number>();
ordinary(1);
</script>
<template>
  <button @click="$emit('anything', 'x')">ok</button>
</template>
"#,
    );
    native_emit_oracle(
        case.path(),
        &corsa,
        "const emit: import('vue').ComponentPublicInstance['$emit'] = () => {};",
        "emit('anything', 'x');",
        "",
    );
    check(case.path(), &corsa, &[]);
}

#[test]
fn absent_macro_retains_the_public_instance_emit_contract() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let case = case();
    fixture(
        case.path(),
        "<template>\n  <button @click=\"$emit('anything', 'x')\">ok</button>\n</template>\n",
    );
    native_emit_oracle(
        case.path(),
        &corsa,
        "const emit: import('vue').ComponentPublicInstance['$emit'] = () => {};",
        "emit('anything', 'x');",
        "",
    );
    check(case.path(), &corsa, &[]);
}

#[path = "support/template_emit_controls.rs"]
mod controls;

//! `html/cross-component-nesting` end to end (Davinci P4-11b): the
//! `examples/html-conformance` project, whose every template is valid on its
//! own, linted with `--cross-file`. The full JSON report is compared exactly.
//!
//! Regenerate after an intended change:
//! `VIZE_UPDATE_CROSS_COMPONENT=1 cargo test -p vize --test lint_cross_component_cli`

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use std::{path::PathBuf, process::Command};

#[test]
fn composed_example_reports_every_cross_component_collision() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/html-conformance");
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&root)
        .args(["lint", "--cross-file", "--format", "json", "src/**/*.vue"])
        .output()
        .expect("run vize lint");
    let stdout = String::from_utf8(output.stdout).expect("utf-8 report");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("json report");
    let rendered = serde_json::to_string_pretty(&parsed).expect("render") + "\n";
    let expected_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/lint_cross_component_cli.json");
    if std::env::var_os("VIZE_UPDATE_CROSS_COMPONENT").is_some() {
        std::fs::write(&expected_path, &rendered).expect("write expectation");
    }
    let expected = std::fs::read_to_string(&expected_path).unwrap_or_default();
    assert_eq!(rendered, expected);
    assert_eq!(output.status.code(), Some(1));
}

fn write(root: &std::path::Path, path: &str, source: &str) {
    let path = root.join(path);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(path, source).expect("write fixture");
}

/// Composition resolves through the parent's imports only: an aliased
/// import (`Fancy` from `./a/Card.vue`, a `<span>` root) is resolved and
/// conforming, and a same-basename `Card.vue` elsewhere (a `<div>` root)
/// is never picked by name, so the unimported `<Card />` stays unknown.
#[test]
fn composition_follows_imports_not_names() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    write(
        root,
        "src/App.vue",
        "<script setup>\nimport Fancy from './a/Card.vue'\n</script>\n\n<template>\n  <p><Fancy /><Card /></p>\n</template>\n",
    );
    write(
        root,
        "src/a/Card.vue",
        "<template>\n  <span>a</span>\n</template>\n",
    );
    write(
        root,
        "src/b/Card.vue",
        "<template>\n  <div>b</div>\n</template>\n",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(["lint", "--cross-file", "--format", "json", "src/**/*.vue"])
        .output()
        .expect("run vize lint");
    let stdout = String::from_utf8(output.stdout).expect("utf-8 report");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("json report");
    let messages: Vec<&serde_json::Value> = parsed
        .as_array()
        .expect("file list")
        .iter()
        .flat_map(|file| file["messages"].as_array().into_iter().flatten())
        .collect();
    assert_eq!(messages, Vec::<&serde_json::Value>::new());
    assert_eq!(output.status.code(), Some(0));
}

/// FP-3: a child's `<button v-if="copy">` is pruned at a usage that leaves
/// `copy` unpassed only when the child's script proves the absent value
/// falsy — a `null` `withDefaults` default or a `false` destructure default —
/// and checked when the prop is passed or its default is truthy.
#[test]
fn unpassed_falsy_props_prune_guarded_branches() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let child = |script: &str| {
        format!(
            "<script setup lang=\"ts\">\n{script}\nconst pick = () => {{}}\n</script>\n\n<template>\n  <span><button v-if=\"copy\" @click=\"pick\">c</button></span>\n</template>\n"
        )
    };
    let children = [
        (
            "src/Kv.vue",
            "withDefaults(defineProps<{ copy?: string | null; oneline?: boolean }>(), { copy: null, oneline: false })",
        ),
        (
            "src/Kv2.vue",
            "withDefaults(defineProps<{ copy?: string }>(), { copy: 'yes' })",
        ),
        (
            "src/Kv3.vue",
            "const { copy = false } = defineProps<{ copy?: boolean }>()",
        ),
        (
            "src/Kv4.vue",
            "defineProps({ copy: { type: Boolean, default: true } })",
        ),
    ];
    for (path, script) in children {
        write(root, path, &child(script));
    }
    write(
        root,
        "src/App.vue",
        "<script setup>\nimport Kv from './Kv.vue'\nimport Kv2 from './Kv2.vue'\nimport Kv3 from './Kv3.vue'\nimport Kv4 from './Kv4.vue'\n</script>\n\n<template>\n  <button><Kv /></button>\n  <button><Kv copy=\"x\" /></button>\n  <button><Kv2 /></button>\n  <button><Kv3 /></button>\n  <button><Kv4 /></button>\n</template>\n",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(["lint", "--cross-file", "--format", "json", "src/**/*.vue"])
        .output()
        .expect("run vize lint");
    let stdout = String::from_utf8(output.stdout).expect("utf-8 report");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("json report");
    let found: Vec<String> = parsed
        .as_array()
        .expect("file list")
        .iter()
        .flat_map(|file| file["messages"].as_array().into_iter().flatten())
        .filter(|message| message["ruleId"] == "html/cross-component-nesting")
        .map(|message| format!("{}:{}", message["line"], message["column"]))
        .collect();
    assert_eq!(found, ["10:11", "11:11", "13:11"]);
}

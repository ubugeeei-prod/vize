#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    reason = "whole CLI fixtures use std strings"
)]
#[path = "support/vite_config.rs"]
mod vite_config;

use std::{fs, path::Path, process::Command};

const SOURCE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vue2-duplicate-attribute-vue2-elm/App.vue.txt"
);
const VUE2_CONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vue2-duplicate-attribute-vue2-elm/vize.config.json.txt"
);
const VUE27_RENDER: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vue2-duplicate-attribute-vue2-elm/vue27-render.txt"
);

#[test]
fn vue2_dialect_keeps_the_last_repeated_class_and_warns() {
    let (code, output) = build(Some(VUE2_CONFIG));
    assert_eq!(code, Some(0), "{output}");
    let built: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(
        built["warnings"],
        serde_json::json!(["Duplicate attribute `class`. Vue 2 keeps the last value."])
    );
    // Vue 2.7.16 keeps the last static value, like its attrs map.
    assert_eq!(
        static_classes(built["code"].as_str().unwrap_or_default()),
        vec![vue2_static_class()]
    );
}

#[test]
fn vue3_still_rejects_the_repeated_class() {
    let (code, _) = build(None);
    assert_eq!(code, Some(1));
}

fn vue2_static_class() -> String {
    VUE27_RENDER
        .split_once("staticClass: \"")
        .and_then(|(_, rest)| rest.split_once('"'))
        .map(|(value, _)| value.to_owned())
        .unwrap_or_default()
}

/// String literals inside the `_normalizeClass([...])` array.
fn static_classes(code: &str) -> Vec<String> {
    code.split_once("_normalizeClass([")
        .and_then(|(_, rest)| rest.split_once("])"))
        .map(|(items, _)| {
            items
                .split(", ")
                .filter_map(|item| item.strip_prefix('"')?.strip_suffix('"'))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn build(config: Option<&str>) -> (Option<i32>, String) {
    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join("App.vue"), SOURCE).unwrap();
    if let Some(config) = config {
        fs::write(
            project.path().join("vite.config.mjs"),
            vite_config::source(config),
        )
        .unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .env("RAYON_NUM_THREADS", "1")
        .args([
            "build",
            "--slow-threshold",
            "600000",
            "-o",
            "out",
            "-f",
            "json",
            "App.vue",
        ])
        .output()
        .unwrap();
    let json = fs::read_to_string(Path::new(project.path()).join("out/App.json"))
        .unwrap_or_else(|_| String::from_utf8_lossy(&output.stderr).into_owned());
    (output.status.code(), json)
}

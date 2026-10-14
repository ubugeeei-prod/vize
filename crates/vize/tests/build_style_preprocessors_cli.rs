#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "whole CLI fixtures use std strings"
)]

//! `vize build` runs the project's sass/less/stylus before scoping (#8905–#8907).
//! The project installs the stand-ins from npm/builder/shared.

use std::{fs, path::Path, process::Command};

const STAND_INS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../npm/builder/shared/test-preprocessors.cjs"
);

const CARD: &str = r#"<script setup>
const accent = "red";
</script>

<template>
  <div class="root"><p class="title">x</p><span :class="$style.badge" /></div>
</template>

<style scoped lang="scss">
$pad: 4px;
.root { padding: $pad; color: v-bind(accent); .title { color: blue; } :deep(.inner) { color: green; } }
</style>

<style module lang="stylus">
.badge
  .icon
    color red
</style>
"#;

const BROKEN: &str = r#"<template><div /></template>

<style lang="less">
.a {
  @error "bad less";
}
</style>
"#;

fn project(with_preprocessors: bool, name: &str, source: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    if with_preprocessors {
        let status = Command::new("node")
            .arg("-e")
            .arg(format!(
                "require({:?}).installStandIns(process.cwd())",
                STAND_INS
            ))
            .current_dir(dir.path())
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(0));
    }
    fs::write(dir.path().join(name), source).unwrap();
    dir
}

fn build(dir: &Path, name: &str) -> (Option<i32>, String, serde_json::Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(dir)
        .env("RAYON_NUM_THREADS", "1")
        .args(["build", "--no-config", "-o", "out", "-f", "json", name])
        .output()
        .unwrap();
    let json = fs::read_to_string(dir.join("out").join(name.replace(".vue", ".json")))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        json,
    )
}

#[test]
fn preprocessor_output_is_scoped_bound_and_module_mapped() {
    let dir = project(true, "Card.vue", CARD);
    let (code, stderr, built) = build(dir.path(), "Card.vue");
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(built["warnings"], serde_json::json!([]));
    let css = built["css"].as_str().unwrap_or_default();
    let id = css
        .split_once("[data-v-")
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(id, _)| id)
        .unwrap_or_default();
    let code = built["code"].as_str().unwrap_or_default();
    let class = |name: &str| {
        code.split_once(&format!("\"{name}\": \""))
            .and_then(|(_, rest)| rest.split_once('"'))
            .map(|(class, _)| class.to_owned())
            .unwrap_or_default()
    };
    let (badge, icon) = (class("badge"), class("icon"));
    assert_eq!(
        css,
        format!(
            ".root[data-v-{id}]{{ padding: 4px; color: var(--{id}-accent); }}.root .title[data-v-{id}]{{ color: blue; }}.root[data-v-{id}] .inner{{ color: green; }}\n.{badge} .{icon} {{\n  color: red;\n}}\n"
        )
    );
    assert_eq!((badge.is_empty(), icon.is_empty()), (false, false));
}

#[test]
fn preprocessor_errors_point_into_the_sfc() {
    let dir = project(true, "Broken.vue", BROKEN);
    let (code, stderr, _) = build(dir.path(), "Broken.vue");
    assert_eq!(code, Some(1));
    let reported: Vec<&str> = stderr
        .lines()
        .map(str::trim)
        .filter(|line| line.ends_with("less error: bad less"))
        .collect();
    assert_eq!(reported, vec!["Broken.vue:5:3: less error: bad less"]);
}

#[test]
fn missing_preprocessor_keeps_the_block_and_warns() {
    let source = "<template><div class=\"a\" /></template>\n<style scoped lang=\"scss\">\n.a { color: red; }\n</style>\n";
    let dir = project(false, "Plain.vue", source);
    let (code, stderr, built) = build(dir.path(), "Plain.vue");
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(
        built["warnings"],
        serde_json::json!([
            "<style lang=\"scss\"> was not preprocessed: install sass in the project"
        ])
    );
}

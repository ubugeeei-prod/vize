#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::string_slice,
    reason = "whole CLI fixtures use std strings"
)]

//! One realistic component per style preprocessor through every CLI surface
//! (#8905–#8907): `vize build` preprocesses, scopes and maps the blocks,
//! `vize fmt` leaves them byte-identical, `vize lint` reads only plain CSS,
//! and `vize check` type-checks the component around them.

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "support/vite_config.rs"]
mod vite_config;

use std::{fs, path::Path, process::Command};

const STAND_INS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../npm/builder/shared/test-preprocessors.cjs"
);

const SCRIPT: &str = r#"<script setup lang="ts">
const accent = "tomato";
const count: number = 2;
</script>

<template>
  <section   class="card">
    <h2 :class="$style.title">{{ count }}</h2>
  </section>
</template>
"#;

const PLAIN: &str = "<style>\n.legacy { color: red !important; }\n</style>\n";

/// `(lang, scoped body, module body, built scoped CSS)` written the way each
/// language is used; `{id}` is the component's scope id.
const FIXTURES: &[(&str, &str, &str, &str)] = &[
    (
        "scss",
        "$gap: 8px;\n.card {\n  padding: $gap;\n  color: v-bind(accent) !important;\n  &__body { margin: 0; }\n}\n",
        ".title {\n  &--large { font-size: 2em; }\n}\n",
        ".card[data-v-{id}]{ padding: 8px; color: var(--{id}-accent) !important; }.card__body[data-v-{id}]{ margin: 0; }",
    ),
    (
        "sass",
        "$gap: 8px\n.card\n  padding: $gap\n  color: v-bind(accent) !important\n",
        ".title\n  &--large\n    font-size: 2em\n",
        ".card[data-v-{id}]{ padding: 8px; color: var(--{id}-accent) !important; }",
    ),
    (
        "less",
        ".card {\n  padding: 8px;\n  color: v-bind(accent) !important;\n  &:hover { opacity: 0.8; }\n}\n",
        ".title {\n  &--large { font-size: 2em; }\n}\n",
        ".card[data-v-{id}]{ padding: 8px; color: var(--{id}-accent) !important; }.card[data-v-{id}]:hover{ opacity: 0.8; }",
    ),
    (
        "stylus",
        ".card\n  padding 8px\n  color v-bind(accent) !important\n",
        ".title\n  &--large\n    font-size 2em\n",
        ".card[data-v-{id}]{ padding: 8px; color: var(--{id}-accent) !important; }",
    ),
];

fn component(lang: &str, scoped: &str, module: &str) -> String {
    format!(
        "{SCRIPT}\n<style scoped lang=\"{lang}\">\n{scoped}</style>\n\n<style module lang=\"{lang}\">\n{module}</style>\n\n{PLAIN}"
    )
}

fn vize(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(dir)
        .env("RAYON_NUM_THREADS", "1")
        .args(args)
        .output()
        .unwrap()
}

fn details(output: &std::process::Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn project(lang: &str, scoped: &str, module: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let status = Command::new("node")
        .arg("-e")
        .arg(format!(
            "require({STAND_INS:?}).installStandIns(process.cwd())"
        ))
        .current_dir(dir.path())
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    fs::write(dir.path().join("Card.vue"), component(lang, scoped, module)).unwrap();
    fs::write(
        dir.path().join("vite.config.mjs"),
        vite_config::source(
            r#"{ "linter": { "preset": "incremental", "rules": { "css/no-important": "warn" } } }"#,
        ),
    )
    .unwrap();
    dir
}

#[test]
fn preprocessor_components_build_scoped_and_module_mapped() {
    for (lang, scoped, module, built_scoped) in FIXTURES {
        let dir = project(lang, scoped, module);
        let output = vize(
            dir.path(),
            &[
                "build",
                "--no-config",
                "-o",
                "out",
                "-f",
                "json",
                "Card.vue",
            ],
        );
        assert_eq!(
            output.status.code(),
            Some(0),
            "{lang}: {}",
            details(&output)
        );
        let built: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join("out/Card.json")).unwrap())
                .unwrap();
        assert_eq!(built["warnings"], serde_json::json!([]), "{lang}");
        let css = built["css"].as_str().unwrap_or_default();
        let id = css
            .split_once("[data-v-")
            .and_then(|(_, rest)| rest.split_once(']'))
            .map_or("", |(id, _)| id);
        // The module class is hashed; the code's class map names it.
        let code = built["code"].as_str().unwrap_or_default();
        let class = code
            .split_once("\"title--large\": \"")
            .and_then(|(_, rest)| rest.split_once('"'))
            .map_or("", |(class, _)| class);
        assert_eq!(
            css,
            format!(
                "{}\n.{class} {{\n  font-size: 2em;\n}}\n\n\n.legacy {{ color: red !important; }}\n",
                built_scoped.replace("{id}", id)
            ),
            "{lang}"
        );
    }
}

#[test]
fn preprocessor_blocks_survive_formatting_byte_identical() {
    for (lang, scoped, module, _) in FIXTURES {
        let dir = project(lang, scoped, module);
        let before = fs::read_to_string(dir.path().join("Card.vue")).unwrap();
        let output = vize(dir.path(), &["fmt", "--no-config", "--write", "Card.vue"]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{lang}: {}",
            details(&output)
        );
        let after = fs::read_to_string(dir.path().join("Card.vue")).unwrap();
        // The template and the plain CSS are formatted; both preprocessor
        // bodies stay byte-identical.
        let expected = before
            .replace("<section   class", "<section class")
            .replace(
                &format!("<style module lang=\"{lang}\">"),
                &format!("<style lang=\"{lang}\" module>"),
            )
            .replace(
                ".legacy { color: red !important; }",
                ".legacy {\n  color: red !important;\n}",
            );
        assert_eq!(after, expected, "{lang}");
        let check = vize(dir.path(), &["fmt", "--no-config", "--check", "Card.vue"]);
        assert_eq!(check.status.code(), Some(0), "{lang}: {}", details(&check));
    }
}

#[test]
fn css_rules_report_only_the_plain_css_block() {
    for (lang, scoped, module, _) in FIXTURES {
        let dir = project(lang, scoped, module);
        let source = component(lang, scoped, module);
        let legacy_line = source[..source.find(".legacy").unwrap()]
            .matches('\n')
            .count()
            + 1;
        let output = vize(
            dir.path(),
            &[
                "lint",
                "--config",
                "vite.config.mjs",
                "--format",
                "json",
                "Card.vue",
            ],
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|_| panic!("{lang}: {}", details(&output)));
        let messages: Vec<(String, u64)> = report[0]["messages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|message| {
                (
                    message["ruleId"].as_str().unwrap_or_default().to_owned(),
                    message["line"].as_u64().unwrap_or_default(),
                )
            })
            .collect();
        assert_eq!(
            messages,
            [("css/no-important".to_owned(), legacy_line as u64)],
            "{lang}: {}",
            details(&output)
        );
    }
}

#[test]
fn preprocessor_components_type_check_around_their_styles() {
    let Some(corsa) = corsa_requirement::required_or_skip(None::<std::path::PathBuf>) else {
        return;
    };
    for (lang, scoped, module, _) in FIXTURES {
        let dir = project(lang, scoped, module);
        let broken = component(lang, scoped, module).replace("{{ count }}", "{{ count.trim() }}");
        let line = broken[..broken.find("count.trim").unwrap()]
            .matches('\n')
            .count()
            + 1;
        for (source, expected) in [
            (component(lang, scoped, module), vec![]),
            (broken, vec![line]),
        ] {
            fs::write(dir.path().join("Card.vue"), source).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_vize"))
                .current_dir(dir.path())
                .env("CORSA_PATH", &corsa)
                .args(["check", "Card.vue", "--format", "json"])
                .output()
                .unwrap();
            let report: serde_json::Value = serde_json::from_slice(&output.stdout)
                .unwrap_or_else(|_| panic!("{lang}: {}", details(&output)));
            let lines: Vec<usize> = report["files"]
                .as_array()
                .into_iter()
                .flatten()
                .flat_map(|file| file["diagnostics"].as_array().cloned().unwrap_or_default())
                .filter_map(|diagnostic| {
                    let rest = diagnostic.as_str()?.strip_prefix("error:")?;
                    rest.split(':').next()?.parse().ok()
                })
                .collect();
            assert_eq!(lines, expected, "{lang}: {}", details(&output));
        }
    }
}

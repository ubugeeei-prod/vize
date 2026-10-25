#![cfg(test)]
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;
use vize_l0::cstr;

fn temp_project_dir(test_name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(
        cstr!(
            "vize-build-profile-cli-{}-{}-{}",
            std::process::id(),
            test_name,
            nonce
        )
        .as_str(),
    )
}

fn write_project_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(file_path, content).unwrap();
}

#[test]
fn build_stats_profile_reports_source_plate_facts() {
    let project_root = temp_project_dir("stats-profile-source-facts");
    let source = r#"<template><div>Hello</div></template>
"#;
    let self_component_source = r#"<template><App /></template>
"#;
    write_project_file(&project_root, "src/App.vue", self_component_source);
    write_project_file(&project_root, "src/Foo.vue", source);
    write_project_file(&project_root, "src/Bar.vue", source);

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "build",
            "--format",
            "stats",
            "--profile",
            "--threads",
            "1",
            "src/App.vue",
            "src/Bar.vue",
            "src/Foo.vue",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        std::str::from_utf8(&output.stdout).unwrap_or("<non-utf8 stdout>"),
        std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>")
    );

    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>");
    for expected in [
        "Stats compile cache",
        "cache.stats_compile.hits",
        "cache.stats_compile.misses",
        "cache.stats_compile.stores",
        "cache.stats_compile.bypasses",
        "cache.stats_compile.bypass.self_component",
        "Source facts",
        "source.plate.sfc.requests",
        "source.block.template.bytes",
        "source.cache.hit.files",
        "source.cache.bypass.self_component.files",
        "Davinci plan",
        "davinci.build.files",
        "davinci.build.planned_groups",
        "davinci.build.planned_passes",
        "Product lanes",
        "lane.atelier.dom.requests",
        "Vue dialects",
        "dialect.vue3.files",
        "Template syntax",
        "template_syntax.standard.files",
        "lane atelier.dom, plate source.sfc",
        "davinci planned groups 2 / passes 2",
    ] {
        assert!(stderr.contains(expected), "missing {expected:?}\n{stderr}");
    }

    let _ = fs::remove_dir_all(project_root);
}

fn profile_row<'a>(profile: &'a Value, section: &str, key: &str) -> &'a Value {
    profile
        .get(section)
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .find(|row| row.get("key").and_then(Value::as_str) == Some(key))
        .unwrap_or_else(|| panic!("missing {section} row {key}"))
}

#[test]
fn early_build_phases_and_bounded_walk_keep_complete_compiled_artifacts() {
    let project = temp_project_dir("early-phases");
    let source = "<script setup lang=\"ts\">const a = 1;</script><template>{{ a }}</template>\n";
    write_project_file(&project, "src/A.vue", source);
    write_project_file(&project, "src/B.vue", source);
    write_project_file(&project, "serialized-config.json", r#"{"vue":{}}"#);
    for index in 0..64 {
        write_project_file(
            &project,
            cstr!("node_modules/unrelated/dist/{index}.js").as_str(),
            "export const x = 1;\n",
        );
    }
    let mut reference: Option<Vec<(PathBuf, Vec<u8>)>> = None;
    for (name, configured, profiled) in [
        ("plain", false, false),
        ("profiled", false, true),
        ("configured", true, true),
    ] {
        let destination = project.join(name);
        let profile_path = project.join(cstr!("{name}.profile.json").as_str());
        let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
        command
            .current_dir(&project)
            .args([
                "build",
                "src/**/*.vue",
                "--format",
                "json",
                "--threads",
                "1",
                "--output",
            ])
            .arg(&destination);
        if configured {
            command
                .arg("--config")
                .arg(project.join("serialized-config.json"));
        } else {
            command.arg("--no-config");
        }
        if profiled {
            command
                .args(["--profile", "--profile-json"])
                .arg(&profile_path);
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{name}: {}",
            std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>")
        );
        let mut artifacts: Vec<_> = fs::read_dir(&destination)
            .unwrap()
            .map(|entry| {
                let path = entry.unwrap().path();
                (
                    path.strip_prefix(&destination).unwrap().to_path_buf(),
                    fs::read(&path).unwrap(),
                )
            })
            .collect();
        artifacts.sort_by(|left, right| left.0.cmp(&right.0));
        assert_eq!(artifacts.len(), 2);
        if let Some(expected) = &reference {
            assert_eq!(
                &artifacts, expected,
                "all JSON/code/metadata bytes must stay identical"
            );
        } else {
            reference = Some(artifacts);
        }
        if !profiled {
            assert!(!profile_path.exists());
            continue;
        }
        let profile: Value = serde_json::from_slice(&fs::read(profile_path).unwrap()).unwrap();
        assert_eq!(
            profile.get("truncation").unwrap(),
            &serde_json::json!({"dropped_spans":0,"dropped_counters":0})
        );
        for key in [
            "cli.build.config.load",
            "cli.build.collect.files",
            "cli.build.outputs.plan",
        ] {
            assert_eq!(
                profile_row(&profile, "spans", key)
                    .get("count")
                    .and_then(Value::as_u64),
                Some(1)
            );
        }
        for (key, expected) in [
            ("cli.build.collect.input_patterns", 1),
            ("cli.build.collect.matched_files", 2),
            ("cli.build.collect.walk_roots", 1),
            ("cli.build.collect.walk_entries", 3),
        ] {
            assert_eq!(
                profile_row(&profile, "counters", key)
                    .get("total")
                    .and_then(Value::as_u64),
                Some(expected),
                "{key}"
            );
        }
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        for phase in [
            "load config",
            "configure build",
            "collect files",
            "plan outputs",
        ] {
            assert!(stderr.contains(phase), "missing separate phase {phase}");
        }
    }
    let _ = fs::remove_dir_all(project);
}

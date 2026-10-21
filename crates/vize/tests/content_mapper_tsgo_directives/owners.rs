//! One standard backend tuple must own both parent errors and no child error.
use std::path::{Path, PathBuf};

use serde_json::json;
use vize_l0::cstr;

pub(super) const COMPONENT: &str = "<script setup lang=\"ts\">\ndefineProps<{ id: 'ok' }>();\ndefineEmits<{ save: [value: number] }>();\n</script>\n<template><div /></template>\n";

#[test]
fn parent_expect_keeps_child_visible_after_each_parent_repair() {
    let Some(tsgo) = std::env::var_os(super::TSGO_ENV).map(PathBuf::from) else {
        eprintln!(
            "skipping exact directive owner conformance: {} is not set",
            super::TSGO_ENV
        );
        return;
    };
    assert!(tsgo.is_file(), "configured backend must exist");
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    let cases_root = super::workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-owner-")
        .tempdir_in(&cases_root)
        .unwrap();
    super::copy_fixture(&fixture, project.path());
    super::install_mapper_manifest(project.path());
    super::install_vue_package(project.path());
    std::fs::write(project.path().join("directives/OwnerComp.vue"), COMPONENT).unwrap();
    let config = json!({
        "compilerOptions": { "strict": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler", "noEmit": true },
        "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
        "files": ["directives/Owner.vue"]
    });
    std::fs::write(
        project.path().join("tsconfig.owner.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
    let retained = capture_root().join("cli");
    std::fs::create_dir_all(&retained).unwrap();
    std::fs::write(retained.join("OwnerComp.vue"), COMPONENT).unwrap();
    std::fs::write(
        retained.join("tsconfig.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
    let mut index = 0;
    let mut failures = Vec::new();
    for newline in ["\n", "\r\n"] {
        for prefix in ["", "😀日本"] {
            for (id, handler) in [
                ("bad", "wrong"),
                ("ok", "wrong"),
                ("bad", "correct"),
                ("ok", "correct"),
            ] {
                let source = source(newline, prefix, id, handler);
                std::fs::write(
                    project.path().join("directives/Owner.vue"),
                    source.as_bytes(),
                )
                .unwrap();
                let output = super::check_project(&tsgo, project.path(), "tsconfig.owner.json");
                let case = retained.join(cstr!("case-{index}").as_str());
                std::fs::create_dir_all(&case).unwrap();
                std::fs::write(case.join("Owner.vue"), source.as_bytes()).unwrap();
                std::fs::write(case.join("stdout.bin"), &output.stdout).unwrap();
                std::fs::write(case.join("stderr.bin"), &output.stderr).unwrap();
                std::fs::write(case.join("outcome.json"), serde_json::to_vec_pretty(&json!({
                    "id": id, "handler": handler, "newline": newline, "prefix": prefix,
                    "status": output.status.code(), "success": output.status.success(),
                    "note": "@missing is retained as a mapping stressor; standard mapper checkUnknownEvents remains false"
                })).unwrap()).unwrap();
                let child = source.find("value.bad").unwrap() + "value.".len();
                let (line, column) = line_column(source.as_str(), child);
                let mut expected = Vec::new();
                if id == "ok" && handler == "correct" {
                    let token = source.find("@vue-expect-error").unwrap();
                    let (line, column) = line_column(source.as_str(), token);
                    expected.push(cstr!("directives/Owner.vue({line},{column}): error vize4: Unused '@vue-expect-error' directive"));
                }
                expected.push(cstr!("directives/Owner.vue({line},{column}): error TS2339: Property 'bad' does not exist on type '1'."));
                let expected = cstr!(
                    "{}\n",
                    expected
                        .iter()
                        .map(|line| line.as_str())
                        .collect::<Vec<_>>()
                        .join("\n")
                );
                std::fs::write(case.join("expected.bin"), expected.as_bytes()).unwrap();
                if output.status.code().is_none()
                    || output.status.success()
                    || !output.stderr.is_empty()
                    || output.stdout != expected.as_bytes()
                {
                    failures.push(cstr!(
                        "case {index}: expected {expected:?}; {}",
                        super::output_text(&output)
                    ));
                }
                index += 1;
            }
        }
    }
    assert_eq!(index, 16);
    calibrate(&tsgo, project.path(), &retained, &mut failures);
    assert!(
        failures.is_empty(),
        "all complete observations retained under {}: {failures:#?}",
        retained.display()
    );
}

fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let prefix = source.get(..offset).unwrap();
    (
        prefix.bytes().filter(|byte| *byte == b'\n').count() + 1,
        prefix
            .rsplit_once('\n')
            .map_or(prefix, |(_, tail)| tail)
            .encode_utf16()
            .count()
            + 1,
    )
}

pub(super) fn capture_root() -> PathBuf {
    std::env::var_os("VIZE_TEST_OWNER_CAPTURE_DIR").map_or_else(
        || {
            super::workspace_root()
                .join("target/vize-tests/content-mapper-directive-owners")
                .join(cstr!("process-{}", std::process::id()).as_str())
        },
        PathBuf::from,
    )
}

pub(super) fn source(newline: &str, prefix: &str, id: &str, handler: &str) -> vize_l0::String {
    cstr!(
        "<script setup lang=\"ts\">{newline}import Comp from './OwnerComp.vue';{newline}const value = 1;{newline}const wrong = (value: string) => {{}};{newline}const correct = (value: number) => {{}};{newline}</script>{newline}<template>{prefix}{newline}<!-- @vue-expect-error -->{newline}<Comp{newline} id=\"{id}\"{newline} @save=\"{handler}\"{newline} @missing=\"correct\"{newline}>{{{{ value.bad }}}}</Comp>{newline}</template>{newline}"
    )
}

fn calibrate(tsgo: &Path, project: &Path, retained: &Path, failures: &mut Vec<vize_l0::String>) {
    for (index, (id, handler, count)) in [
        ("bad", "wrong", 5),
        ("ok", "wrong", 2),
        ("bad", "correct", 4),
        ("ok", "correct", 1),
    ]
    .into_iter()
    .enumerate()
    {
        let source = source("\n", "", id, handler).replace("<!-- @vue-expect-error -->\n", "");
        std::fs::write(project.join("directives/Owner.vue"), &source).unwrap();
        let output = super::check_project(tsgo, project, "tsconfig.owner.json");
        let case = retained.join(cstr!("calibration-{index}").as_str());
        std::fs::create_dir_all(&case).unwrap();
        std::fs::write(case.join("Owner.vue"), source.as_bytes()).unwrap();
        std::fs::write(case.join("stdout.bin"), &output.stdout).unwrap();
        std::fs::write(case.join("stderr.bin"), &output.stderr).unwrap();
        std::fs::write(
            case.join("outcome.json"),
            serde_json::to_vec_pretty(&json!({
                "id": id, "handler": handler, "status": output.status.code(),
                "success": output.status.success(), "expectedCount": count,
                "contract": "unchanged no-directive fallback; bad-id inference mapping remains unresolved",
            }))
            .unwrap(),
        )
        .unwrap();
        let expected = calibration_expected(id, handler);
        std::fs::write(case.join("expected.bin"), expected.as_bytes()).unwrap();
        if output.status.code().is_none()
            || output.status.success()
            || !output.stderr.is_empty()
            || output.stdout != expected.as_bytes()
        {
            failures.push(cstr!(
                "calibration {index}: {}",
                super::output_text(&output)
            ));
        }
    }
}

// Frozen complete d6 dynamic-config output. Each event inference call checks
// the id, so the same mapped id diagnostic is reported three times.
fn calibration_expected(id: &str, handler: &str) -> vize_l0::String {
    const BAD_ID: &str = "error TS2322: Type '\"bad\"' is not assignable to type '\"ok\"'.\n";
    let mut output = vize_l0::String::from(
        "directives/Owner.vue(12,11): error TS2339: Property 'bad' does not exist on type '1'.\n",
    );
    if id == "bad" {
        output.push_str(cstr!("directives/Owner.vue(9,2): {BAD_ID}").as_str());
    }
    if handler == "wrong" {
        output.push_str("directives/Owner.vue(10,3): error TS2322: Type '(value: string) => void' is not assignable to type '(value: number) => any'.\n  Types of parameters 'value' and 'value' are incompatible.\n    Type 'number' is not assignable to type 'string'.\n");
    }
    if id == "bad" {
        output.push_str(
            cstr!("directives/Owner.vue(9,2): {BAD_ID}directives/Owner.vue(9,2): {BAD_ID}")
                .as_str(),
        );
    }
    output
}

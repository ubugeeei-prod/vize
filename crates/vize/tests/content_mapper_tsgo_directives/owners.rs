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
                if output.status.code() != Some(2)
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
    calibrate_plain_typescript(&tsgo, project.path(), &retained, &mut failures);
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
        ("bad", "wrong", 3),
        ("ok", "wrong", 2),
        ("bad", "correct", 2),
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
                "contract": "one complete diagnostic per invalid child, handler or prop",
            }))
            .unwrap(),
        )
        .unwrap();
        let expected = calibration_expected(id, handler);
        std::fs::write(case.join("expected.bin"), expected.as_bytes()).unwrap();
        std::fs::write(
            case.join("historical-expected.bin"),
            historical_calibration_expected(id, handler).as_bytes(),
        )
        .unwrap();
        if output.status.code() != Some(2)
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

fn calibration_expected(id: &str, handler: &str) -> vize_l0::String {
    let mut output = vize_l0::String::from(
        "directives/Owner.vue(12,11): error TS2339: Property 'bad' does not exist on type '1'.\n",
    );
    if handler == "wrong" {
        output.push_str("directives/Owner.vue(10,3): error TS2322: Type '(value: string) => void' is not assignable to type '(value: number) => any'.\n  Types of parameters 'value' and 'value' are incompatible.\n    Type 'number' is not assignable to type 'string'.\n");
    }
    if id == "bad" {
        output.push_str("directives/Owner.vue(9,2): error TS2322: Type '\"bad\"' is not assignable to type '\"ok\"'.\n");
    }
    output
}

// Preserve the previous complete calibration packet, including its duplicate
// prop diagnostic, as evidence rather than retaining that defect as a law.
fn historical_calibration_expected(id: &str, handler: &str) -> vize_l0::String {
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
        output.push_str(cstr!("directives/Owner.vue(9,2): {BAD_ID}").as_str());
    }
    output
}

/// An independent native TypeScript program proves the three type obligations
/// without Vue transforms, virtual constructors or diagnostic directives.
fn calibrate_plain_typescript(
    tsgo: &Path,
    project: &Path,
    retained: &Path,
    failures: &mut Vec<vize_l0::String>,
) {
    let config = json!({
        "compilerOptions": { "strict": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler", "noEmit": true },
        "files": ["directives/OwnerReference.ts"]
    });
    let config = serde_json::to_vec_pretty(&config).unwrap();
    std::fs::write(project.join("tsconfig.reference.json"), &config).unwrap();
    std::fs::write(retained.join("tsconfig.reference.json"), &config).unwrap();
    let mut index = 0;
    for newline in ["\n", "\r\n"] {
        for prefix in ["", "/*😀日本*/"] {
            for (id, handler) in [
                ("bad", "wrong"),
                ("ok", "wrong"),
                ("bad", "correct"),
                ("ok", "correct"),
            ] {
                let source = cstr!(
                    "export {{}};{newline}const wrong = (_value: string) => {{}};{newline}const correct = (_value: number) => {{}};{newline}{prefix}const id: 'ok' = '{id}';{newline}{prefix}const handler: (value: number) => any = {handler};{newline}const child = 1;{newline}{prefix}child.bad;{newline}"
                );
                std::fs::write(project.join("directives/OwnerReference.ts"), &source).unwrap();
                let output = super::check_project(tsgo, project, "tsconfig.reference.json");
                let mut expected = vize_l0::String::default();
                if id == "bad" {
                    let offset = source.find("const id").unwrap() + "const ".len();
                    let (line, column) = line_column(&source, offset);
                    expected.push_str(cstr!("directives/OwnerReference.ts({line},{column}): error TS2322: Type '\"bad\"' is not assignable to type '\"ok\"'.\n").as_str());
                }
                if handler == "wrong" {
                    let offset = source.find("const handler").unwrap() + "const ".len();
                    let (line, column) = line_column(&source, offset);
                    expected.push_str(cstr!("directives/OwnerReference.ts({line},{column}): error TS2322: Type '(_value: string) => void' is not assignable to type '(value: number) => any'.\n  Types of parameters '_value' and 'value' are incompatible.\n    Type 'number' is not assignable to type 'string'.\n").as_str());
                }
                let offset = source.find("child.bad").unwrap() + "child.".len();
                let (line, column) = line_column(&source, offset);
                expected.push_str(cstr!("directives/OwnerReference.ts({line},{column}): error TS2339: Property 'bad' does not exist on type '1'.\n").as_str());
                let case = retained.join(cstr!("plain-ts-reference-{index}").as_str());
                std::fs::create_dir_all(&case).unwrap();
                std::fs::write(case.join("OwnerReference.ts"), source.as_bytes()).unwrap();
                std::fs::write(case.join("stdout.bin"), &output.stdout).unwrap();
                std::fs::write(case.join("stderr.bin"), &output.stderr).unwrap();
                std::fs::write(case.join("expected.bin"), expected.as_bytes()).unwrap();
                std::fs::write(
                    case.join("outcome.json"),
                    serde_json::to_vec_pretty(&json!({
                        "id": id, "handler": handler, "newline": newline, "prefix": prefix,
                        "status": output.status.code(), "success": output.status.success(),
                        "producer": "handwritten native TypeScript; no content mapper"
                    }))
                    .unwrap(),
                )
                .unwrap();
                if output.status.code() != Some(2)
                    || !output.stderr.is_empty()
                    || output.stdout != expected.as_bytes()
                {
                    failures.push(cstr!(
                        "plain TypeScript reference {index}: expected {expected:?}; {}",
                        super::output_text(&output)
                    ));
                }
                index += 1;
            }
        }
    }
    assert_eq!(index, 16);
}

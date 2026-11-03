//! Bound-prop inference for original generic library call signatures (#11447).
#![expect(
    clippy::unwrap_used,
    reason = "fixture setup and assertions after raw capture"
)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use serde_json::{Value, json};
use vize_l0::{String, cstr};

use super::super::{TSGO_ENV, install_mapper_manifest, install_vue_package, workspace_root};

#[path = "../reported_dynamic_targets/capture.rs"]
mod capture;

const INPUTS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/reported-library-generic-events-11447/input.json"
));
const CAPTURE: &str = "VIZE_TEST_REPORTED_LIBRARY_GENERIC_EVENTS_CAPTURE_DIR";
const REQUIRED: &str = "VIZE_TEST_REPORTED_LIBRARY_GENERIC_EVENTS_REQUIRED";

type Files = BTreeMap<String, String>;

struct Case {
    issue: u64,
    role: &'static str,
    files: Files,
    expected: Expected,
}

enum Expected {
    Clean,
    Exact(String),
}

fn changed(source: &str, from: &str, to: &str) -> String {
    assert_eq!(source.matches(from).count(), 1, "derived control preimage");
    source.replacen(from, to, 1).into()
}

fn edit(files: &Files, file: &str, from: &str, to: &str) -> Files {
    let mut result = files.clone();
    result.insert(file.into(), changed(&files[file], from, to));
    result
}

fn anchor(source: &str, token: &str) -> (usize, usize) {
    assert_eq!(
        source.matches(token).count(),
        1,
        "unique authored error anchor"
    );
    let offset = source.find(token).unwrap();
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix.rsplit('\n').next().unwrap().encode_utf16().count() + 1;
    (line, column)
}

fn error(files: &Files, file: &str, token: &str, code: u32, message: &str) -> Expected {
    let (line, column) = anchor(&files[file], token);
    Expected::Exact(cstr!(
        "{file}({line},{column}): error TS{code}: {message}\n"
    ))
}

fn push(cases: &mut Vec<Case>, issue: u64, role: &'static str, files: Files, expected: Expected) {
    cases.push(Case {
        issue,
        role,
        files,
        expected,
    });
}

fn original(inputs: &Value, issue: u64) -> Files {
    inputs["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["issue"] == issue)
        .unwrap()["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            let text = file["text"].as_str().unwrap();
            assert_eq!(file["bytes"], text.len());
            assert_eq!(
                file["sha256"].as_str(),
                Some(capture::hash(text.as_bytes()).as_str())
            );
            (
                String::from(file["path"].as_str().unwrap()),
                String::from(text),
            )
        })
        .collect()
}

fn cases(inputs: &Value) -> Vec<Case> {
    let mut cases = Vec::new();
    let files = original(inputs, 11447);
    let poison = edit(
        &files,
        "src/Probe.vue",
        "if (typeof value === 'string') model = value;",
        "const payload: never = value;",
    );
    for (role, current) in [
        ("original", files.clone()),
        ("payload-poison", poison.clone()),
        ("repair", files.clone()),
        ("payload-poison-restored", poison),
    ] {
        let expected = if role.starts_with("payload-poison") {
            error(
                &current,
                "src/Probe.vue",
                "payload",
                2322,
                "Type 'TBaz' is not assignable to type 'never'.\n  Type 'string' is not assignable to type 'never'.",
            )
        } else {
            Expected::Clean
        };
        push(&mut cases, 11447, role, current, expected);
    }
    let no_any = edit(
        &files,
        "src/Probe.vue",
        "if (typeof value === 'string') model = value;",
        "const noAny: false = null as unknown as (0 extends (1 & typeof value) ? true : false);\n        const exact: TBaz = value;\n        if (typeof value === 'string') model = value;",
    );
    push(
        &mut cases,
        11447,
        "payload-no-any-exact",
        no_any,
        Expected::Clean,
    );
    let unconstrained = edit(&files, "src/Probe.vue", "TBaz extends string", "TBaz");
    push(
        &mut cases,
        11447,
        "unconstrained-generic",
        unconstrained,
        Expected::Clean,
    );
    let mut dotted = edit(
        &files,
        "src/lib.ts",
        "export declare const Foo",
        "declare const Foo",
    );
    let lib = cstr!(
        "{}\nexport declare const Parts: {{ Foo: typeof Foo }};\n",
        dotted["src/lib.ts"]
    );
    dotted.insert("src/lib.ts".into(), lib);
    let dotted = edit(
        &dotted,
        "src/Probe.vue",
        "import { Foo }",
        "import { Parts }",
    );
    let dotted = edit(&dotted, "src/Probe.vue", "<Foo", "<Parts.Foo");
    push(
        &mut cases,
        11447,
        "dotted-library-export",
        dotted,
        Expected::Clean,
    );
    let explicit_unknown = edit(&files, "src/Probe.vue", "(value) =>", "(value: unknown) =>");
    let no_context = edit(
        &files,
        "src/lib.ts",
        "VNode & { __ctx?: NonNullable<Awaited<typeof setup>> }",
        "VNode",
    );
    for (role, current) in [
        ("explicit-unknown-not-widened", explicit_unknown),
        ("no-context-keeps-fallback", no_context),
    ] {
        let expected = error(
            &current,
            "src/Probe.vue",
            "model = value",
            2322,
            "Type 'string' is not assignable to type 'TBaz'.\n  'string' is assignable to the constraint of type 'TBaz', but 'TBaz' could be instantiated with a different subtype of constraint 'string'.",
        );
        push(&mut cases, 11447, role, current, expected);
    }
    for (role, number_first) in [
        ("number-overload-first", true),
        ("number-overload-last", false),
    ] {
        let number = "(props: { modelValue: number }): VNode & { __ctx?: { props: { 'onUpdate:modelValue'?: (value: number) => void } } };";
        let string = "(props: { modelValue: string }): VNode & { __ctx?: { props: { 'onUpdate:modelValue'?: (value: string) => void } } };";
        let (first, last) = if number_first {
            (number, string)
        } else {
            (string, number)
        };
        let mut current = files.clone();
        current.insert("src/lib.ts".into(), cstr!("import type {{ VNode }} from 'vue';\nexport declare const Foo: {{\n{first}\n{last}\n}};\n"));
        current.insert("src/Probe.vue".into(), "<script setup lang=\"ts\">\nimport { Foo } from './lib';\nconst model = defineModel<number>({ required: true });\n</script>\n<template>\n<Foo :model-value=\"model\" @update:model-value=\"value => { const noAny: false = null as unknown as (0 extends (1 & typeof value) ? true : false); value.toFixed(); }\" />\n</template>\n".into());
        push(&mut cases, 11447, role, current.clone(), Expected::Clean);
        let bad = edit(&current, "src/Probe.vue", "toFixed", "toUpperCase");
        let expected = error(
            &bad,
            "src/Probe.vue",
            "toUpperCase",
            2339,
            "Property 'toUpperCase' does not exist on type 'number'.",
        );
        push(
            &mut cases,
            11447,
            if number_first {
                "first-overload-wrong-payload"
            } else {
                "last-overload-wrong-payload"
            },
            bad,
            expected,
        );
    }
    cases
}

fn assert_case(case: &Case, packet: &Value) {
    let process = &packet["checker"];
    assert!(process["error"].is_null(), "{packet}");
    assert!(process["signal"].is_null(), "{packet}");
    assert_eq!(process["stderrBytes"], json!([]), "{packet}");
    let stdout = process["stdoutUtf8"]
        .as_str()
        .expect("complete UTF-8 native output");
    match &case.expected {
        Expected::Clean => {
            assert_eq!(process["status"], 0, "{packet}");
            assert_eq!(stdout, "", "{packet}");
        }
        Expected::Exact(expected) => {
            assert_eq!(process["status"], 2, "{packet}");
            assert_eq!(stdout, expected.as_str(), "{packet}");
        }
    }
    let companion = &packet["mapperCompanion"];
    assert_eq!(companion["process"]["status"], 0, "{packet}");
    assert!(companion["process"]["error"].is_null(), "{packet}");
    assert_eq!(companion["process"]["stderrBytes"], json!([]), "{packet}");
    assert!(companion["protocolError"].is_null(), "{packet}");
    assert!(
        companion["responses"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r.get("error").is_none()),
        "{packet}"
    );
}

#[test]
fn originals_payload_errors_and_repairs_keep_library_generics() {
    let required = std::env::var_os(REQUIRED);
    assert!(required.is_none() || required.as_deref() == Some(std::ffi::OsStr::new("1")));
    let Some(host) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        assert!(required.is_none(), "{TSGO_ENV} is required");
        eprintln!("skipping unconfigured library-event native host");
        return;
    };
    assert!(host.is_file(), "{TSGO_ENV} must name the actual host");
    let capture_override = std::env::var_os(CAPTURE).map(PathBuf::from);
    assert!(
        required.is_none() || capture_override.is_some(),
        "persistent capture override required in the mandatory role"
    );
    // The full workflow calls this child in both the early syntax group and
    // the later project group. Keep both raw populations without overwriting.
    let capture_dir = Some(capture_override.unwrap_or_else(|| {
        workspace_root()
            .join(
                "target/vize-tests/content-mapper-directive-owners/reported-library-generic-events",
            )
            .join(std::process::id().to_string())
    }));
    if let Some(path) = &capture_dir {
        std::fs::create_dir_all(path).unwrap();
    }
    let inputs: Value = serde_json::from_str(INPUTS).unwrap();
    let cases = cases(&inputs);
    let root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&root).unwrap();
    let mut packets = Vec::new();
    let mut providers = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        let project = tempfile::Builder::new()
            .prefix("reported-library-generic-events-")
            .tempdir_in(&root)
            .unwrap();
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        for (name, text) in &case.files {
            let path = project.path().join(name.as_str());
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text.as_bytes()).unwrap();
        }
        if index == 0 {
            providers.push(capture::provider(&host, project.path()));
        }
        let args = [
            "--runExternalCode",
            "--noEmit",
            "-p",
            "tsconfig.json",
            "--pretty",
            "false",
        ];
        let packet = json!({"issue":case.issue,"role":case.role,"cwd":project.path(),"argv":args,"inputs":case.files,
            "checker":capture::process(Command::new(&host).current_dir(project.path()).args(args),None),
            "mapperCompanion":capture::mapper_companion(project.path(),&case.files),"publicAcceptance":false});
        if let Some(path) = &capture_dir {
            capture::save(
                &path.join(cstr!("{index:02}-{}-{}.json", case.issue, case.role).as_str()),
                &packet,
            );
        }
        packets.push(packet);
        if index + 1 == cases.len() {
            providers.push(capture::provider(&host, project.path()));
        }
    }
    let envelope = json!({"schema":"vize.reportedLibraryGenericEvents.native.v1","inputs":inputs,"providers":providers,"cases":packets,"semanticVerdictNotYetApplied":true,"publicAcceptance":false});
    if let Some(path) = &capture_dir {
        capture::save(&path.join("whole-capture.json"), &envelope);
    }
    println!("REPORTED_LIBRARY_GENERIC_EVENTS_RAW={envelope}");
    let providers = envelope["providers"].as_array().unwrap();
    for key in ["checkerImage", "sourceCliImage"] {
        assert!(providers[0][key]["sha256"].is_string(), "{envelope}");
        assert_eq!(
            providers[0][key]["sha256"], providers[1][key]["sha256"],
            "{envelope}"
        );
    }
    assert_eq!(
        providers[0]["vueManifestBytes"], providers[1]["vueManifestBytes"],
        "{envelope}"
    );
    for provider in providers {
        assert_eq!(
            provider["checkerVersionCompanion"]["status"], 0,
            "{envelope}"
        );
        assert_eq!(
            provider["checkerVersionCompanion"]["stderrBytes"],
            json!([]),
            "{envelope}"
        );
    }
    for (case, packet) in cases.iter().zip(envelope["cases"].as_array().unwrap()) {
        assert_case(case, packet);
    }
}

//! Source-bound originals for #11362/#11363/#11366/#11367; public Windows acceptance is separate.
#![expect(
    clippy::unwrap_used,
    reason = "fixture setup and post-capture test assertions"
)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use serde_json::{Value, json};
use vize_l0::{String, cstr};

use super::support::{install_mapper_manifest, install_vue_package, workspace_root};
use super::{TSGO_ENV, VUE_ENV};

mod capture;

const REQUIRED: &str = "VIZE_TEST_REPORTED_DYNAMIC_TARGETS_REQUIRED";
const CAPTURE: &str = "VIZE_TEST_REPORTED_DYNAMIC_TARGETS_CAPTURE_DIR";
const PROVIDER_ROLE: &str = "VIZE_TEST_REPORTED_DYNAMIC_TARGETS_PROVIDER_ROLE";
const FIXTURES: &str = "tests/_fixtures/differential/typechecker/reported-dynamic-targets-11362";

// Complete pinned provider representation of the separate invalid-key control,
// in addition to its independently authored missing-key/code/location law.
// Since 2fb13a1f97 (#11635/#11637) the component type is an alias named after
// its file, so both known hosts print `ChildComponent` instead of its nested
// generated constructor type. A declared role still selects the provider before
// running cases, and the captured host is validated afterward.
const INDEX_KEY_MESSAGE: &str =
    "Type 'TKey' cannot be used to index type '{ a: ChildComponent; }'.";

fn provider_contract(role: &std::ffi::OsStr) -> Option<(&'static str, &'static str, &'static str)> {
    match role.to_str()? {
        "standard-go-d6-vue35" => Some((INDEX_KEY_MESSAGE, "Version 7.1.0-dev\n", "3.5.35")),
        "reported-tsrs-vue44" => Some((
            INDEX_KEY_MESSAGE,
            "Version 7.1.0-beta (tsrs 0.20.1-ts7.1.0-beta, microsoft/TypeScript@3e9236361fa9)\n",
            "3.5.44",
        )),
        _ => None,
    }
}

struct Case {
    issue: u64,
    role: &'static str,
    files: BTreeMap<String, String>,
    expected: Expected,
}

enum Expected {
    Clean,
    Error(u64, &'static str),
}

fn changed(source: &str, from: &str, to: &str) -> String {
    assert_eq!(source.matches(from).count(), 1, "derived control preimage");
    String::from(source.replacen(from, to, 1))
}

fn fixture(name: &str) -> (Value, BTreeMap<String, String>) {
    let bytes = std::fs::read(workspace_root().join(FIXTURES).join(name)).unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    let files = value["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            let text = file["content"].as_str().unwrap();
            assert_eq!(file["sha256"], capture::hash(text.as_bytes()).as_str());
            assert_eq!(file["utf8Bytes"], text.len());
            (
                String::from(file["file"].as_str().unwrap()),
                String::from(text),
            )
        })
        .collect();
    (value, files)
}

fn push_case(
    cases: &mut Vec<Case>,
    issue: u64,
    role: &'static str,
    original: &BTreeMap<String, String>,
    probe: String,
    expected: Expected,
) {
    let mut files = original.clone();
    *files.get_mut("src/Probe.vue").unwrap() = probe;
    cases.push(Case {
        issue,
        role,
        files,
        expected,
    });
}

fn cases(index_key_message: &'static str) -> (Vec<Value>, Vec<Case>) {
    let mut inputs = Vec::new();
    let mut cases = Vec::new();
    let (input, files) = fixture("guarded-member.json");
    inputs.push(input);
    let original = &files["src/Probe.vue"];
    let bad = changed(original, " v-if=\"foo\"", "");
    for (role, probe, expected) in [
        ("original", original.clone(), Expected::Clean),
        (
            "unguarded-error",
            bad.clone(),
            Expected::Error(18048, "'foo' is possibly 'undefined'."),
        ),
        ("guard-repair", original.clone(), Expected::Clean),
        (
            "unguarded-restored",
            bad,
            Expected::Error(18048, "'foo' is possibly 'undefined'."),
        ),
    ] {
        push_case(&mut cases, 11362, role, &files, probe, expected);
    }

    let (input, files) = fixture("dotted-tag.json");
    inputs.push(input);
    let original = &files["src/Probe.vue"];
    let repair = changed(original, "bar=\"baz\"", ":bar=\"123\"");
    let direct = changed(
        &changed(
            original,
            "import * as Parts from \"./parts\";",
            "import Foo from \"./Foo.vue\";",
        ),
        "Parts.Foo",
        "Foo",
    );
    for (role, probe, expected) in [
        (
            "original",
            original.clone(),
            Expected::Error(2322, "Type 'string' is not assignable to type 'number'."),
        ),
        ("number-repair", repair, Expected::Clean),
        (
            "wrong-prop-restored",
            original.clone(),
            Expected::Error(2322, "Type 'string' is not assignable to type 'number'."),
        ),
        (
            "direct-import-error",
            direct,
            Expected::Error(2322, "Type 'string' is not assignable to type 'number'."),
        ),
    ] {
        push_case(&mut cases, 11363, role, &files, probe, expected);
    }

    let (input, files) = fixture("generic-index-model.json");
    inputs.push(input);
    let original = &files["src/Probe.vue"];
    let wrong = changed(original, "defineModel<string>", "defineModel<number>");
    let nongeneric = changed(
        &changed(original, " generic=\"TKey extends 'a' | 'b'\"", ""),
        "foo: TKey",
        "foo: 'a' | 'b'",
    );
    for (role, probe, expected) in [
        ("original", original.clone(), Expected::Clean),
        (
            "number-model-error",
            wrong.clone(),
            Expected::Error(2322, "Type 'number' is not assignable to type 'string'."),
        ),
        ("string-model-repair", original.clone(), Expected::Clean),
        (
            "number-model-restored",
            wrong,
            Expected::Error(2322, "Type 'number' is not assignable to type 'string'."),
        ),
        ("nongeneric-control", nongeneric, Expected::Clean),
    ] {
        push_case(&mut cases, 11366, role, &files, probe, expected);
    }

    let (input, files) = fixture("slot-index-event.json");
    inputs.push(input);
    let original = &files["src/Probe.vue"];
    let wrong = changed(original, "value.toFixed()", "value.nope()");
    let outside = changed(
        &changed(&wrong, "  <Panel>\n    <template #foo>\n", ""),
        "    </template>\n  </Panel>\n",
        "",
    );
    for (role, probe, expected) in [
        ("original", original.clone(), Expected::Clean),
        (
            "number-member-error",
            wrong.clone(),
            Expected::Error(2339, "Property 'nope' does not exist on type 'number'."),
        ),
        ("number-member-repair", original.clone(), Expected::Clean),
        (
            "number-member-restored",
            wrong,
            Expected::Error(2339, "Property 'nope' does not exist on type 'number'."),
        ),
        (
            "outside-slot-error",
            outside,
            Expected::Error(2339, "Property 'nope' does not exist on type 'number'."),
        ),
    ] {
        push_case(&mut cases, 11367, role, &files, probe, expected);
    }
    // A separate derived control proves the original authored indexing error
    // survives homogeneous target normalization; the original eighteen stay exact.
    let (_, files) = fixture("generic-index-model.json");
    let original = &files["src/Probe.vue"];
    let missing_key = changed(
        original,
        "const map = { a: Child, b: Child };",
        "const map = { a: Child };",
    );
    for (role, probe, expected) in [
        (
            "invalid-key-error",
            missing_key.clone(),
            Expected::Error(2536, index_key_message),
        ),
        ("invalid-key-repair", original.clone(), Expected::Clean),
        (
            "invalid-key-restored",
            missing_key,
            Expected::Error(2536, index_key_message),
        ),
    ] {
        push_case(&mut cases, 11366, role, &files, probe, expected);
    }
    // An imported assertion may refine one entry after equal initializers.
    // Keep this separate from all twenty-one existing original/control states.
    let (_, mut files) = fixture("generic-index-model.json");
    files.insert(
        String::from("src/Functional.ts"),
        String::from("import type { FunctionalComponent } from 'vue';\ndeclare const Child: FunctionalComponent<{ value: string }>;\nexport default Child;\n"),
    );
    files.insert(
        String::from("src/refine.ts"),
        String::from("import type { FunctionalComponent } from 'vue';\nimport Child from './Functional';\nexport declare function refine(value: { a: typeof Child; b: typeof Child }): asserts value is { a: typeof Child & FunctionalComponent<{ value: string; extra: number }>; b: typeof Child };\n"),
    );
    let probe = String::from(
        "<script setup lang=\"ts\" generic=\"Unused\">\nimport Child from './Functional';\nimport { refine } from './refine';\nconst map = { a: Child, b: Child };\nconst key = 'b';\nrefine(map);\n</script>\n<template>\n  <component :is=\"map[key]\" value=\"ok\" />\n</template>\n",
    );
    push_case(
        &mut cases,
        11366,
        "opaque-assertion-refinement-control",
        &files,
        probe,
        Expected::Clean,
    );
    (inputs, cases)
}

// Exact authored diagnostic anchors from the frozen originals and the explicit
// token edits above; never infer an expected location from a received diagnostic.
fn expected_error_location(case: &Case) -> Option<(usize, usize)> {
    match (case.issue, case.role) {
        (11362, "unguarded-error" | "unguarded-restored") => Some((6, 19)),
        (11363, "original" | "wrong-prop-restored") => Some((6, 14)),
        (11363, "direct-import-error") => Some((6, 8)),
        (11366, "number-model-error" | "number-model-restored") => Some((10, 29)),
        (11366, "invalid-key-error" | "invalid-key-restored") => Some((10, 19)),
        (11367, "number-member-error" | "number-member-restored") => Some((12, 59)),
        (11367, "outside-slot-error") => Some((10, 59)),
        _ => None,
    }
}

fn assert_case(case: &Case, packet: &Value) {
    let output = &packet["checker"];
    assert!(output["error"].is_null(), "{packet}");
    assert!(output["signal"].is_null(), "{packet}");
    assert_eq!(output["stderrBytes"], json!([]), "{packet}");
    let stdout = output["stdoutUtf8"].as_str().expect("UTF-8 checker stdout");
    match case.expected {
        Expected::Clean => {
            assert_eq!(output["status"], 0, "{packet}");
            assert_eq!(stdout, "", "{packet}");
        }
        Expected::Error(code, message) => {
            let (line, column) =
                expected_error_location(case).expect("registered source error role");
            let expected = cstr!("src/Probe.vue({line},{column}): error TS{code}: {message}\n");
            assert_eq!(output["status"], 2, "{packet}");
            assert_eq!(stdout, expected.as_str(), "{packet}");
            let authored = case.files["src/Probe.vue"]
                .lines()
                .nth(line - 1)
                .expect("authored diagnostic line");
            assert!(
                column > 0 && column <= authored.encode_utf16().count(),
                "{packet}"
            );
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
            .all(|response| response.get("error").is_none()),
        "{packet}"
    );
}

#[test]
fn originals_and_error_repair_restore_keep_component_target_types() {
    assert_eq!(provider_contract(std::ffi::OsStr::new("")), None);
    assert_eq!(provider_contract(std::ffi::OsStr::new("unknown")), None);
    let provider_role = std::env::var_os(PROVIDER_ROLE);
    let contract = provider_role
        .as_deref()
        .map(|role| provider_contract(role).expect("unknown dynamic-target provider role"));
    let required = std::env::var_os(REQUIRED);
    assert!(
        required.is_none() || required.as_deref() == Some(std::ffi::OsStr::new("1")),
        "unknown required-provider role"
    );
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        assert!(
            required.is_none(),
            "{TSGO_ENV} is mandatory in the reported role"
        );
        eprintln!("skipping reported dynamic targets: {TSGO_ENV} is unset");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} must name a physical checker");
    if required.is_some() {
        assert!(
            std::env::var_os(VUE_ENV).is_some(),
            "reported role requires explicit real Vue"
        );
        assert!(
            std::env::var_os(CAPTURE).is_some(),
            "reported role requires persistent raw capture"
        );
    }
    let capture_dir = std::env::var_os(CAPTURE).map(PathBuf::from);
    if let Some(path) = &capture_dir {
        std::fs::create_dir_all(path).unwrap();
    }
    let (index_key_message, checker_version, vue_version) =
        contract.expect("native dynamic-target cases require a declared provider role");
    let (inputs, cases) = cases(index_key_message);
    let root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("reported-dynamic-targets-")
        .tempdir_in(root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    let provider = capture::provider(&tsgo, project.path());
    let mut packets = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        // Separate physical projects retain each original include roster without stale prior-case files.
        let project = tempfile::Builder::new()
            .prefix("reported-dynamic-target-case-")
            .tempdir_in(workspace_root().join("target/vize-tests/tests"))
            .unwrap();
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        for (name, content) in &case.files {
            let path = project.path().join(name.as_str());
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        }
        let arguments = [
            "--runExternalCode",
            "--noEmit",
            "-p",
            "tsconfig.json",
            "--pretty",
            "false",
        ];
        let checker = capture::process(
            &mut Command::new(&tsgo)
                .current_dir(project.path())
                .args(arguments),
            None,
        );
        let packet = json!({
            "issue":case.issue,"role":case.role,"cwd":project.path(),"argv":arguments,
            "inputs":case.files,"checker":checker,
            "mapperCompanion":capture::mapper_companion(project.path(), &case.files),
            "companionIsCheckerProcess":false,"publicAcceptance":false,
        });
        if let Some(path) = &capture_dir {
            capture::save(
                &path.join(cstr!("{index:02}-{}-{}.json", case.issue, case.role).as_str()),
                &packet,
            );
        }
        packets.push(packet);
    }
    let envelope = json!({"schema":"vize.reportedDynamicTargetsCapture.v1","inputs":inputs,"provider":provider,"providerRole":provider_role.as_deref().and_then(std::ffi::OsStr::to_str),"cases":packets,"semanticVerdictNotYetApplied":true,"publicAcceptance":false});
    if let Some(path) = &capture_dir {
        capture::save(&path.join("whole-capture.json"), &envelope);
    }
    println!("REPORTED_DYNAMIC_TARGETS_RAW={envelope}");
    // Validate the declared role only after all raw cases survive. Metadata
    // validates the choice; observed diagnostic text never chooses a golden.
    let observed_provider = &envelope["provider"];
    assert_eq!(observed_provider["checkerVersionCompanion"]["status"], 0);
    assert_eq!(
        observed_provider["checkerVersionCompanion"]["stdoutUtf8"].as_str(),
        Some(checker_version),
    );
    assert_eq!(
        observed_provider["checkerVersionCompanion"]["stderrBytes"],
        json!([])
    );
    let vue_bytes: Vec<u8> = serde_json::from_value(observed_provider["vueManifestBytes"].clone())
        .expect("complete captured Vue manifest bytes");
    let vue: Value = serde_json::from_slice(&vue_bytes).expect("captured Vue manifest JSON");
    assert_eq!(vue["version"].as_str(), Some(vue_version));
    // Every checker and whole transform response above survives the first failing expectation.
    for (case, packet) in cases.iter().zip(envelope["cases"].as_array().unwrap()) {
        assert_case(case, packet);
    }
}

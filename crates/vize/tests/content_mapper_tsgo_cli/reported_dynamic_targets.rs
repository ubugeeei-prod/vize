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
const FIXTURES: &str = "tests/_fixtures/differential/typechecker/reported-dynamic-targets-11362";

struct Case {
    issue: u64,
    role: &'static str,
    files: BTreeMap<String, String>,
    expected: Expected,
}

enum Expected {
    Clean,
    Error(u64, &'static str),
    ErrorPrefix(u64, &'static str),
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

fn cases() -> (Vec<Value>, Vec<Case>) {
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
            Expected::ErrorPrefix(2536, "Type 'TKey' cannot be used to index type"),
        ),
        ("invalid-key-repair", original.clone(), Expected::Clean),
        (
            "invalid-key-restored",
            missing_key,
            Expected::ErrorPrefix(2536, "Type 'TKey' cannot be used to index type"),
        ),
    ] {
        push_case(&mut cases, 11366, role, &files, probe, expected);
    }
    (inputs, cases)
}

fn assert_case(case: &Case, packet: &Value) {
    let output = &packet["checker"];
    assert!(output["error"].is_null(), "{packet}");
    assert!(output["signal"].is_null(), "{packet}");
    assert_eq!(output["stderrBytes"], json!([]), "{packet}");
    let stdout = output["stdoutUtf8"].as_str().expect("UTF-8 checker stdout");
    let diagnostics: Vec<_> = stdout
        .lines()
        .filter(|line| line.contains(": error TS"))
        .collect();
    match case.expected {
        Expected::Clean => {
            assert_eq!(output["status"], 0, "{packet}");
            assert_eq!(stdout, "", "{packet}");
        }
        Expected::Error(code, message) | Expected::ErrorPrefix(code, message) => {
            assert_ne!(output["status"], 0, "{packet}");
            assert!(output["status"].is_i64(), "{packet}");
            assert_eq!(diagnostics.len(), 1, "{packet}");
            assert!(diagnostics[0].starts_with("src/Probe.vue("), "{packet}");
            assert!(
                diagnostics[0].contains(cstr!("): error TS{code}: {message}").as_str()),
                "{packet}"
            );
            let line_and_column = diagnostics[0]
                .split_once('(')
                .unwrap()
                .1
                .split_once(')')
                .unwrap()
                .0;
            let (line, column) = line_and_column.split_once(',').unwrap();
            let line: usize = line.parse().unwrap();
            let column: usize = column.parse().unwrap();
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
    let (inputs, cases) = cases();
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
    let envelope = json!({"schema":"vize.reportedDynamicTargetsCapture.v1","inputs":inputs,"provider":provider,"cases":packets,"semanticVerdictNotYetApplied":true,"publicAcceptance":false});
    if let Some(path) = &capture_dir {
        capture::save(&path.join("whole-capture.json"), &envelope);
    }
    println!("REPORTED_DYNAMIC_TARGETS_RAW={envelope}");
    // Every checker and whole transform response above survives the first failing expectation.
    for (case, packet) in cases.iter().zip(envelope["cases"].as_array().unwrap()) {
        assert_case(case, packet);
    }
}

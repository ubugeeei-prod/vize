//! Exact reported #11373 inputs and independent typed negative/inverse controls.
use std::{fs, io::Write, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::json;

use super::support::run_tsgo;
use super::{TSGO_ENV, VUE_ENV, install_mapper_manifest, install_vue_package, workspace_root};

const REQUIRE: &str = "VIZE_TEST_REQUIRE_GLOBAL_COMPONENT_LISTENER";
const CAPTURE: &str = "VIZE_REPORTED_GLOBAL_COMPONENT_LISTENER_CAPTURE";
const PACK: &str = include_str!(
    "../../../../tests/_fixtures/differential/typechecker/global-component-listener-11373/cases.json"
);
const CONTROLS: &str = include_str!(
    "../../../../tests/_fixtures/differential/typechecker/global-component-listener-11373/controls.json"
);

#[derive(Clone, Deserialize, Serialize)]
struct Input {
    file: String,
    source: String,
}
#[derive(Deserialize)]
struct Pack {
    cases: Vec<BaseCase>,
}
#[derive(Deserialize)]
struct BaseCase {
    inputs: Vec<Input>,
}
#[derive(Deserialize)]
struct Controls {
    cases: Vec<Case>,
}
#[derive(Deserialize, Serialize)]
struct Case {
    id: String,
    overrides: Vec<Input>,
    success: bool,
    diagnostics: Vec<Diagnostic>,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Diagnostic {
    file: String,
    code: u32,
    source_needle: String,
    message: Option<String>,
    message_contains: Option<String>,
}

fn write_capture(path: &std::path::Path, value: &serde_json::Value) {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    serde_json::to_writer_pretty(&mut file, value).unwrap();
    file.write_all(b"\n").unwrap();
}

#[test]
fn reported_components_and_typed_controls_keep_real_diagnostics() {
    match std::env::var(REQUIRE).as_deref() {
        Ok("1") => {}
        Err(std::env::VarError::NotPresent) | Ok("0") => {
            eprintln!(
                "#11373 dedicated exact-profile conformance not selected; no semantic qualification"
            );
            return;
        }
        other => panic!("unsupported {REQUIRE}: {other:?}"),
    }
    let tsgo = PathBuf::from(std::env::var_os(TSGO_ENV).expect("required #11373 checker"));
    assert!(tsgo.is_file(), "{TSGO_ENV}: {}", tsgo.display());
    let vue = PathBuf::from(std::env::var_os(VUE_ENV).expect("required #11373 Vue package"));
    let vue_bytes = fs::read(vue.join("package.json")).unwrap();
    let vue_manifest: serde_json::Value = serde_json::from_slice(&vue_bytes).unwrap();
    assert_eq!(
        vue_manifest["version"], "3.5.43",
        "separate exact #11373 profile"
    );
    let capture = PathBuf::from(std::env::var_os(CAPTURE).expect("required owned artifact child"));
    fs::create_dir_all(&capture).unwrap();
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/global-component-listener-11373");
    let pack: Pack = serde_json::from_str(PACK).unwrap();
    let controls: Controls = serde_json::from_str(CONTROLS).unwrap();
    assert_eq!(pack.cases.len(), 1);
    assert_eq!(pack.cases[0].inputs.len(), 6);
    assert_eq!(controls.cases.len(), 11);
    let git = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(workspace_root())
        .output()
        .unwrap();
    assert!(git.status.success());
    write_capture(
        &capture.join("header.json"),
        &json!({
            "schema":"vize.reported-global-component-listener.source.v1", "sourceGitStdoutBytes":git.stdout,
            "sourceGitStderrBytes":git.stderr, "sourceGitStatus":git.status.code(),
            "checkerPath":tsgo,"vuePath":vue,"vuePackageBytes":vue_bytes,
            "pack":serde_json::from_str::<serde_json::Value>(PACK).unwrap(),
            "controls":serde_json::from_str::<serde_json::Value>(CONTROLS).unwrap(),
            "scope":"Source CLI mapper with exact selected #11373 provider. Installed-public Mac role remains separate."
        }),
    );
    let project_parent = workspace_root().join("target/vize-tests/tests");
    fs::create_dir_all(&project_parent).unwrap();
    let mut captured = Vec::new();
    for case in controls.cases {
        let project = tempfile::Builder::new()
            .prefix("reported-global-listener-")
            .tempdir_in(&project_parent)
            .unwrap();
        let mut inputs = pack.cases[0].inputs.clone();
        for replacement in &case.overrides {
            if let Some(prior) = inputs
                .iter_mut()
                .find(|input| input.file == replacement.file)
            {
                *prior = replacement.clone();
            } else {
                inputs.push(replacement.clone());
            }
        }
        let mut sources = Vec::new();
        let mut projections = Vec::new();
        for input in &inputs {
            let bytes = fs::read(fixture.join(&input.source)).unwrap();
            let target = project.path().join(&input.file);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(&target, &bytes).unwrap();
            sources.push((input.file.clone(), bytes.clone()));
            if input.file.ends_with(".vue") {
                let text = std::str::from_utf8(&bytes).unwrap();
                let transform = vize_canon::generate_vue_content_mapper_transform(&target, text);
                projections.push(json!({"file":input.file,"defaultApiProjectionSeparateFromHost":match transform {
                    Ok(value) => serde_json::to_value(value).unwrap(),
                    Err(error) => json!({"error":format!("{error:#?}")}),
                }}));
            }
        }
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        let mapper_manifest =
            fs::read(project.path().join("node_modules/vize/package.json")).unwrap();
        write_capture(
            &capture.join(format!("{}.started.json", case.id)),
            &json!({
                "id":case.id,"inputs":inputs,"sources":sources,"defaultApiProjections":projections,
                "mapperManifestBytes":mapper_manifest,"argv":["--runExternalCode","-p","tsconfig.json","--pretty","false"]
            }),
        );
        let output = run_tsgo(
            &tsgo,
            project.path(),
            &[
                "--runExternalCode",
                "-p",
                "tsconfig.json",
                "--pretty",
                "false",
            ],
        );
        #[cfg(unix)]
        let signal = {
            use std::os::unix::process::ExitStatusExt;
            output.status.signal()
        };
        #[cfg(not(unix))]
        let signal: Option<i32> = None;
        let after = sources
            .iter()
            .map(|(file, _)| (file.clone(), fs::read(project.path().join(file)).unwrap()))
            .collect::<Vec<_>>();
        let mapper_after = fs::read(project.path().join("node_modules/vize/package.json")).unwrap();
        let record = json!({"id":case.id,"expectedIntent":case,"status":output.status.code(),"signal":signal,
            "stdoutBytes":output.stdout,"stderrBytes":output.stderr,"sources":sources,"filesAfter":after,
            "defaultApiProjections":projections,"mapperManifestBytes":mapper_manifest,
            "mapperManifestAfterBytes":mapper_after});
        write_capture(&capture.join(format!("{}.json", case.id)), &record);
        println!("vize-reported-global-listener {}", record);
        captured.push((
            case,
            sources,
            after,
            signal,
            output,
            mapper_manifest,
            mapper_after,
            project,
        ));
    }
    // Every complete process packet and source snapshot precedes semantic verdicts.
    for (case, sources, after, signal, output, mapper_manifest, mapper_after, _project) in &captured
    {
        assert_eq!(
            mapper_manifest, mapper_after,
            "{}: selected mapper manifest changed",
            case.id
        );
        assert_eq!(
            sources, after,
            "{}: original/control inputs changed",
            case.id
        );
        assert_eq!(*signal, None, "{}: {output:?}", case.id);
        assert_eq!(
            output.status.success(),
            case.success,
            "{}: {output:?}",
            case.id
        );
        assert_eq!(
            output.status.code(),
            Some(if case.success { 0 } else { 2 }),
            "{}: {output:?}",
            case.id
        );
        assert!(output.stderr.is_empty(), "{}: {output:?}", case.id);
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        assert!(!stdout.contains("error TS2448:"), "{}: {stdout}", case.id);
        assert!(
            !stdout.contains("has no corresponding location"),
            "{}: {stdout}",
            case.id
        );
        let diagnostics = stdout
            .lines()
            .filter(|line| line.contains(": error TS"))
            .collect::<Vec<_>>();
        assert_eq!(
            diagnostics.len(),
            case.diagnostics.len(),
            "{}: {stdout}",
            case.id
        );
        if case.success {
            assert!(stdout.is_empty(), "{}: {stdout}", case.id);
        }
        for (actual, expected) in diagnostics.iter().zip(&case.diagnostics) {
            let (location, tail) = actual.split_once(": error TS").unwrap();
            let (code, message) = tail.split_once(": ").unwrap();
            assert_eq!(
                code.parse::<u32>().unwrap(),
                expected.code,
                "{}: {stdout}",
                case.id
            );
            if let Some(exact) = &expected.message {
                assert_eq!(message, exact, "{}: {stdout}", case.id);
            }
            if let Some(fragment) = &expected.message_contains {
                assert!(stdout.contains(fragment), "{}: {stdout}", case.id);
            }
            let prefix = format!("{}(", expected.file);
            let position = location
                .strip_prefix(&prefix)
                .unwrap()
                .strip_suffix(')')
                .unwrap();
            let (line, column) = position.split_once(',').unwrap();
            let line = line.parse::<usize>().unwrap();
            let column = column.parse::<usize>().unwrap();
            let source = std::str::from_utf8(
                &sources
                    .iter()
                    .find(|(name, _)| name == &expected.file)
                    .unwrap()
                    .1,
            )
            .unwrap();
            assert_eq!(source.matches(&expected.source_needle).count(), 1);
            let start = source.find(&expected.source_needle).unwrap();
            let end = start + expected.source_needle.len();
            let mut byte = 0;
            let actual_byte = source
                .split_inclusive('\n')
                .enumerate()
                .find_map(|(index, text)| {
                    let begin = byte;
                    byte += text.len();
                    (index + 1 == line).then_some(begin + column - 1)
                })
                .unwrap();
            assert!(
                (start..=end).contains(&actual_byte),
                "{}: authored owner {start}..{end}, actual {actual_byte}: {stdout}",
                case.id
            );
        }
    }
    write_capture(
        &capture.join("summary.json"),
        &json!({
            "schema":"vize.reported-global-component-listener.summary.v1","cases":captured.len(),
            "captureReady":true,"sourceSemanticQualified":true,"installedPublicQualified":false,
            "negativeAndInverseContracts":true,"noGeneratedTdz":true
        }),
    );
}

//! Complete real host outcomes; generated placeholders grant no authored error exemption.
use super::{install_mapper_manifest, install_vue_package, workspace_root};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use vize_canon::{
    ContentMapperTransformOptions, VirtualProject,
    generate_vue_content_mapper_transform_with_options,
};

mod contracts;
use contracts::{assert_mapper_geometry, diagnostics, raw_expected};

const NATIVE: &str = "VIZE_TEST_CONTEXT_UNDEFINED_NATIVE";
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Cases {
    version: u32,
    historical_issue: u32,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    file: String,
    source: String,
    sha256: String,
    expected: String,
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .concat()
}

fn write(root: &Path, file: &str, bytes: &[u8]) {
    let path = root.join(file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}
fn observe(
    command: &mut Command,
    cwd: &Path,
    capture: &Path,
    host: &str,
    inputs: &Value,
) -> Output {
    let argv: Vec<_> = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    let output = command.current_dir(cwd).output().unwrap();
    write(capture, &format!("{host}.stdout.raw"), &output.stdout);
    write(capture, &format!("{host}.stderr.raw"), &output.stderr);
    write(capture,&format!("{host}.json"),&serde_json::to_vec_pretty(&json!({"schema":"vize.typed-context-undefined-host","version":1,"host":host,"argv":argv,"cwd":cwd,"status":output.status.code(),"stdout":output.stdout,"stderr":output.stderr,"inputs":inputs})).unwrap());
    output
}
#[test]
#[ignore = "requires both actual physical native providers and published Vue graph"]
fn typed_context_values_preserve_complete_native_mapper_and_cli_packets() {
    let native = PathBuf::from(std::env::var_os(NATIVE).expect("actual native7.0.2 required"))
        .canonicalize()
        .unwrap();
    let beta =
        PathBuf::from(std::env::var_os(super::TSGO_ENV).expect("actual beta mapper required"))
            .canonicalize()
            .unwrap();
    let revision = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(workspace_root())
        .output()
        .unwrap();
    assert!(revision.status.success());
    let expected_root = PathBuf::from(
        std::env::var_os("VIZE_TEST_CONTEXT_UNDEFINED_SOURCE_ROOT")
            .expect("actual compiled workspace root required"),
    );
    assert_eq!(
        workspace_root().canonicalize().unwrap(),
        expected_root.canonicalize().unwrap()
    );
    let expected_revision = std::env::var("VIZE_TEST_CONTEXT_UNDEFINED_SOURCE_REVISION")
        .expect("actual compiled source revision required");
    assert_eq!(
        String::from_utf8(revision.stdout.clone()).unwrap().trim(),
        expected_revision
    );
    let observer = std::env::current_exe().unwrap();
    let images: Vec<_> = [
        native.as_path(),
        beta.as_path(),
        Path::new(env!("CARGO_BIN_EXE_vize")),
        observer.as_path(),
    ]
    .into_iter()
    .map(|path| json!({"path":path,"sha256":sha256(&fs::read(path).unwrap())}))
    .collect();
    let identity = json!({"sourceRevision":String::from_utf8(revision.stdout).unwrap().trim(),"images":images});
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/template-context-undefined-10667");
    let cases: Cases =
        serde_json::from_slice(&fs::read(fixture.join("host-cases.json")).unwrap()).unwrap();
    assert_eq!(cases.version, 1);
    assert_eq!(cases.historical_issue, 10667);
    assert_eq!(cases.cases.len(), 15);
    let root = workspace_root().join("target/vize-tests/typed-context-undefined-10667");
    let mut observations = Vec::new();
    let mut cli_observations = Vec::new();
    let mut geometries = Vec::new();
    for (variant, newline, prefix) in [
        ("lf", "\n", ""),
        ("crlf", "\r\n", ""),
        ("unicode-lf", "\n", "<!--🧭-->"),
        ("unicode-crlf", "\r\n", "<!--🧭-->"),
    ] {
        for case in &cases.cases {
            let capture = root.join(variant).join(&case.id);
            fs::create_dir_all(&capture).unwrap();
            let project = tempfile::Builder::new()
                .prefix("project-")
                .tempdir_in(&capture)
                .unwrap();
            let original = fs::read_to_string(fixture.join(&case.source)).unwrap();
            assert_eq!(
                sha256(original.as_bytes()),
                case.sha256,
                "pinned input bytes"
            );
            let source = format!("{prefix}{original}").replace('\n', newline);
            let dep = fs::read_to_string(fixture.join("dep.ts.txt"))
                .unwrap()
                .replace('\n', newline);
            write(project.path(), &case.file, source.as_bytes());
            write(project.path(), "src/dep.ts", dep.as_bytes());
            fs::create_dir(project.path().join("node_modules")).unwrap();
            install_vue_package(project.path());
            install_mapper_manifest(project.path());
            let config = fs::read(fixture.join("tsconfig.json.txt")).unwrap();
            write(project.path(), "tsconfig.native.json", &config);
            let mut mapped_config: Value = serde_json::from_slice(&config).unwrap();
            mapped_config["contentMappers"] = json!([{"package":"vize","extensions":[".vue"]}]);
            write(
                project.path(),
                "tsconfig.json",
                &serde_json::to_vec_pretty(&mapped_config).unwrap(),
            );
            let mut virtual_project = VirtualProject::new(project.path()).unwrap();
            virtual_project.set_tsconfig_path(Some(project.path().join("tsconfig.native.json")));
            virtual_project
                .register_path(&project.path().join("src/dep.ts"))
                .unwrap();
            virtual_project
                .register_path(&project.path().join(&case.file))
                .unwrap();
            virtual_project.materialize().unwrap();
            let virtual_file = format!("{}.ts", case.file);
            let virtual_code =
                fs::read_to_string(virtual_project.virtual_root().join(&virtual_file)).unwrap();
            let transform = generate_vue_content_mapper_transform_with_options(
                Path::new(&case.file),
                &source,
                ContentMapperTransformOptions::default().with_check_javascript(true),
            )
            .unwrap();

            let inputs = json!({"identity":identity,"source":source,"dep":dep,"config":config,"virtualCode":virtual_code,"mapperTransform":transform,"nativeConfig":fs::read(virtual_project.virtual_root().join("tsconfig.json")).unwrap(),"nativeHelpers":fs::read(virtual_project.virtual_root().join("__vize_helpers.d.ts")).unwrap(),"vueManifest":fs::read(project.path().join("node_modules/vue/package.json")).unwrap(),"mapperManifest":fs::read(project.path().join("node_modules/vize/package.json")).unwrap()});
            let expected_native = raw_expected(case, &virtual_code, &virtual_file);
            let expected_mapper = raw_expected(case, &source, &case.file);
            for (host, binary, mapper) in [
                ("native-raw-702", &native, false),
                ("native-raw-beta", &beta, false),
                ("mapper-beta", &beta, true),
            ] {
                let mut command = Command::new(binary);
                if mapper {
                    command.arg("--runExternalCode");
                }
                command.args(["--noEmit", "--pretty", "false", "-p", "tsconfig.json"]);
                let output = observe(
                    &mut command,
                    if mapper {
                        project.path()
                    } else {
                        virtual_project.virtual_root()
                    },
                    &capture,
                    host,
                    &inputs,
                );
                observations.push((
                    format!("{variant}/{}/{host}", case.id),
                    output,
                    if mapper {
                        expected_mapper.clone()
                    } else {
                        expected_native.clone()
                    },
                    if case.expected == "clean" {
                        0
                    } else if host == "native-raw-702" {
                        1
                    } else {
                        2
                    },
                ));
            }
            let mut cli = Command::new(env!("CARGO_BIN_EXE_vize"));
            cli.env("CORSA_PATH", &native).args([
                "check",
                "--format",
                "json",
                "--quiet",
                "--tsconfig",
                "tsconfig.native.json",
            ]);
            let output = observe(&mut cli, project.path(), &capture, "cli", &inputs);
            let errors = diagnostics(case, &source);
            let messages: Vec<_> = errors
                .iter()
                .map(|(line, column, code, message, text)| {
                    let message = format!("error:{line}:{column} [TS{code}] {message}");
                    // JSON retains template expression context; authored script
                    // diagnostics use their concise complete message contract.
                    if case.expected == "member" {
                        format!("{message} (source: {text})")
                    } else {
                        message
                    }
                })
                .collect();
            let mut expected_config: Value = serde_json::from_slice(&config).unwrap();
            let options = expected_config["compilerOptions"].take();
            let mut files = vec![case.file.clone(), "src/dep.ts".to_owned()];
            files.sort();
            let expected_files: Vec<_> = files
                .iter()
                .map(|file| json!({"file":file,"diagnostics":if file == &case.file { messages.clone() } else { Vec::new() }}))
                .collect();
            let expected = json!({"files":expected_files,"programs":[{"root":".","tsconfig":"tsconfig.native.json","compilerOptions":options,"files":files}],"errorCount":errors.len(),"warningCount":0,"fileCount":2});
            cli_observations.push((
                format!("{variant}/{}", case.id),
                output,
                expected,
                i32::from(!errors.is_empty()),
            ));
            geometries.push((case, source, transform));
        }
    }
    assert_eq!(cli_observations.len(), 60);
    for (name, output, expected, status) in cli_observations {
        assert_eq!(output.status.code(), Some(status), "CLI {name}: {output:?}");
        assert!(output.stderr.is_empty(), "CLI {name}: {:?}", output.stderr);
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            expected,
            "CLI {name}"
        );
    }
    assert_eq!(observations.len(), 180);
    for (name, output, expected, status) in observations {
        assert_eq!(output.status.code(), Some(status), "{name}: {output:?}");
        assert!(output.stderr.is_empty(), "{name}: {:?}", output.stderr);
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            expected,
            "{name}: entire stdout"
        );
    }
    assert_eq!(geometries.len(), 60);
    for (case, source, transform) in geometries {
        assert_mapper_geometry(case, &source, &transform);
    }
}

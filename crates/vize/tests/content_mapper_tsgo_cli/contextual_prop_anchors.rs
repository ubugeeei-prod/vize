//! Real complete native programs and the published 7.1 mapper own diagnostics.
use super::{install_mapper_manifest, install_vue_package, workspace_root};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use vize_canon::VirtualProject;
use vize_l0::{String, cstr};

const MATRIX: &str = "VIZE_TEST_CONTEXTUAL_PROP_NATIVE_TSGO";

fn write(root: &Path, name: &str, bytes: &[u8]) {
    let path = root.join(name);
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
        .map(|arg| String::from(arg.to_string_lossy().as_ref()))
        .collect();
    let output = command.current_dir(cwd).output().unwrap();
    write(capture, cstr!("{host}.stdout.raw").as_str(), &output.stdout);
    write(capture, cstr!("{host}.stderr.raw").as_str(), &output.stderr);
    write(capture, cstr!("{host}.json").as_str(), &serde_json::to_vec_pretty(&json!({
        "schema":"vize.contextual-prop-anchor-host","version":1,"host":host,
        "argv":argv,"cwd":cwd,"status":output.status.code(),"stdout":output.stdout,"stderr":output.stderr,"inputs":inputs,
    })).unwrap());
    output
}

fn expected_error(file: &str, source: &str) -> String {
    let offset = source.find("localNmae").unwrap();
    assert_eq!(source.matches("localNmae").count(), 1);
    let before = source.get(..offset).unwrap();
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = before.rsplit('\n').next().unwrap().encode_utf16().count() + 1;
    cstr!(
        "{file}({line},{column}): error TS2552: Cannot find name 'localNmae'. Did you mean 'localName'?\n"
    )
}

struct Observation {
    name: String,
    native: Output,
    beta: Output,
    mapper: Output,
    native_expected: String,
    mapper_expected: String,
    invalid: bool,
}

#[ignore = "requires both actual native providers and the published Vue fixture graph"]
#[test]
fn contextual_prop_anchors_preserve_complete_native_and_mapper_outcomes() {
    let native = PathBuf::from(
        std::env::var_os(MATRIX).expect("the actual 7.0.2 native provider is required"),
    )
    .canonicalize()
    .unwrap();
    let beta = PathBuf::from(
        std::env::var_os(super::TSGO_ENV)
            .expect("the actual published 7.1 beta mapper provider is required"),
    )
    .canonicalize()
    .unwrap();
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/contextual-prop-anchors-10661");
    let ids: Vec<String> =
        serde_json::from_slice(&fs::read(fixture.join("case-ids.json")).unwrap()).unwrap();
    assert_eq!(ids.len(), 10);
    let root = workspace_root().join("target/vize-tests/contextual-prop-anchors-10661");
    let mut observations = Vec::new();
    for (variant, newline, prefix) in [
        ("lf", "\n", ""),
        ("crlf", "\r\n", ""),
        ("unicode-lf", "\n", "<!--🧭-->"),
        ("unicode-crlf", "\r\n", "<!--🧭-->"),
    ] {
        for name in &ids {
            let capture = root.join(variant).join(name.as_str());
            fs::create_dir_all(&capture).unwrap();
            let project = tempfile::Builder::new()
                .prefix("project-")
                .tempdir_in(&capture)
                .unwrap();
            let source =
                fs::read_to_string(fixture.join(name.as_str()).join("src/Route.vue.txt")).unwrap();
            let source = cstr!("{prefix}{source}").replace('\n', newline);
            let dep = fs::read_to_string(fixture.join("dep.ts.txt"))
                .unwrap()
                .replace('\n', newline);
            write(project.path(), "src/Route.vue", source.as_bytes());
            write(project.path(), "src/dep.ts", dep.as_bytes());
            fs::create_dir(project.path().join("node_modules")).unwrap();
            install_vue_package(project.path());
            install_mapper_manifest(project.path());
            let native_config = fs::read(fixture.join("tsconfig.json.txt")).unwrap();
            write(project.path(), "tsconfig.native.json", &native_config);
            let mut mapper_config: Value = serde_json::from_slice(&native_config).unwrap();
            mapper_config["contentMappers"] = json!([{"package":"vize","extensions":[".vue"]}]);
            write(
                project.path(),
                "tsconfig.json",
                &serde_json::to_vec_pretty(&mapper_config).unwrap(),
            );
            let mut virtual_project = VirtualProject::new(project.path()).unwrap();
            virtual_project.set_tsconfig_path(Some(project.path().join("tsconfig.native.json")));
            virtual_project
                .register_path(&project.path().join("src/dep.ts"))
                .unwrap();
            virtual_project
                .register_path(&project.path().join("src/Route.vue"))
                .unwrap();
            virtual_project.materialize().unwrap();
            let virtual_code =
                fs::read_to_string(virtual_project.virtual_root().join("src/Route.vue.ts"))
                    .unwrap();
            let inputs = json!({"source":source,"dep":dep,"config":native_config,"virtualCode":virtual_code,
                "nativeConfig":fs::read(virtual_project.virtual_root().join("tsconfig.json")).unwrap(),
                "nativeHelpers":fs::read(virtual_project.virtual_root().join("__vize_helpers.d.ts")).unwrap(),
                "vueManifest":fs::read(project.path().join("node_modules/vue/package.json")).unwrap(),
                "mapperManifest":fs::read(project.path().join("node_modules/vize/package.json")).unwrap()});
            let mut native_command = Command::new(&native);
            native_command.args(["--noEmit", "--pretty", "false", "-p", "tsconfig.json"]);
            let raw = observe(
                &mut native_command,
                virtual_project.virtual_root(),
                &capture,
                "native-raw-702",
                &inputs,
            );
            let mut beta_command = Command::new(&beta);
            beta_command.args(["--noEmit", "--pretty", "false", "-p", "tsconfig.json"]);
            let raw_beta = observe(
                &mut beta_command,
                virtual_project.virtual_root(),
                &capture,
                "native-raw-beta",
                &inputs,
            );
            let mut mapper = Command::new(&beta);
            mapper.args([
                "--runExternalCode",
                "--noEmit",
                "--pretty",
                "false",
                "-p",
                "tsconfig.json",
            ]);
            let mapped = observe(
                &mut mapper,
                project.path(),
                &capture,
                "mapper-beta",
                &inputs,
            );
            let authored_error = name.as_str() == "authored-ts2552-retained";
            // The original all-84 source remains a separately labelled known
            // producer defect. Require its complete prior output, never classify
            // it as a clean keyword result or discard its unmapped diagnostics.
            let known_undefined_collision = name.as_str() == "all-keyword-props";
            let invalid = authored_error || known_undefined_collision;
            let native_expected = if authored_error {
                expected_error("src/Route.vue.ts", &virtual_code)
            } else if known_undefined_collision {
                fs::read_to_string(fixture.join(name.as_str()).join("known-native.stdout.txt"))
                    .unwrap()
                    .into()
            } else {
                String::default()
            };
            let mapper_expected = if authored_error {
                expected_error("src/Route.vue", &source)
            } else if known_undefined_collision {
                fs::read_to_string(fixture.join(name.as_str()).join("known-mapper.stdout.txt"))
                    .unwrap()
                    .into()
            } else {
                String::default()
            };
            observations.push(Observation {
                name: cstr!("{variant}/{name}"),
                native: raw,
                beta: raw_beta,
                mapper: mapped,
                native_expected,
                mapper_expected,
                invalid,
            });
        }
    }
    assert_eq!(observations.len(), 40);
    for o in observations {
        for (host, output, expected, status) in [
            (
                "native-raw-702",
                &o.native,
                &o.native_expected,
                i32::from(o.invalid),
            ),
            (
                "native-raw-beta",
                &o.beta,
                &o.native_expected,
                if o.invalid { 2 } else { 0 },
            ),
            (
                "mapper-beta",
                &o.mapper,
                &o.mapper_expected,
                if o.invalid { 2 } else { 0 },
            ),
        ] {
            assert_eq!(
                output.status.code(),
                Some(status),
                "{}/{host}: {output:?}",
                o.name
            );
            assert!(
                output.stderr.is_empty(),
                "{}/{host}: complete stderr {:?}",
                o.name,
                output.stderr
            );
            assert_eq!(
                output.stdout,
                expected.as_bytes(),
                "{}/{host}: complete stdout",
                o.name
            );
        }
    }
}

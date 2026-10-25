//! Complete source CLI and official host packets across quoted-generic repairs.

use std::{
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use vize_l0::{String as CompactString, cstr};

use super::{TSGO_ENV, install_mapper_manifest, install_vue_package, output_text, workspace_root};

mod body_error;
mod syntax;

const PACK: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/quoted-generic-parameters-10540/cases.json"
));
const MESSAGE: &str = "Type 'number' is not assignable to type '\"a,b\"'.";
const INPUTS: [&str; 7] = [
    "tsconfig.json",
    "tsconfig.native.json",
    "node_modules/vize/package.json",
    "node_modules/vue/package.json",
    "src/App.vue",
    "src/Comma.vue",
    "src/Edges.vue",
];

struct Observation {
    id: CompactString,
    phase: &'static str,
    invalid: bool,
    prefix: &'static str,
    mapper: Output,
    source: Output,
}

fn write(root: &Path, name: &str, bytes: &[u8]) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

fn materialize(root: &Path, fixture: &Path, case: &Value, newline: &str, prefix: &str) {
    for input in case["inputs"].as_array().unwrap() {
        let name = input["file"].as_str().unwrap();
        if name == "tsconfig.json" {
            continue;
        }
        let mut source =
            std::fs::read_to_string(fixture.join(input["source"].as_str().unwrap())).unwrap();
        if name == "src/App.vue" {
            assert_eq!(source.matches("<Comma ").count(), 1);
            source = source.replace("<Comma ", cstr!("{prefix}<Comma ").as_str());
        }
        write(root, name, source.replace('\n', newline).as_bytes());
    }
}

/// Store every byte before parsing stdout or asserting host behavior.
fn observe(command: &mut Command, root: &Path, artifacts: &Path, host: &str) -> Output {
    let argv = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|argument| CompactString::from(argument.to_string_lossy().as_ref()))
        .collect::<Vec<_>>();
    let inputs = INPUTS
        .iter()
        .map(|name| json!({"file":name,"bytes":std::fs::read(root.join(name)).unwrap()}))
        .collect::<Vec<_>>();
    let output = command.output().unwrap();
    std::fs::create_dir_all(artifacts).unwrap();
    write(
        artifacts,
        cstr!("{host}.stdout.raw").as_str(),
        &output.stdout,
    );
    write(
        artifacts,
        cstr!("{host}.stderr.raw").as_str(),
        &output.stderr,
    );
    let packet = json!({
        "schema":"vize.quoted-generic-parameters-host-observation",
        "version":1,"host":host,"argv":argv,"cwd":root,
        "status":output.status.code(),"success":output.status.success(),
        "statusText":cstr!("{}", output.status),
        "stdout":output.stdout,"stderr":output.stderr,"inputs":inputs,
    });
    write(
        artifacts,
        cstr!("{host}.json").as_str(),
        &serde_json::to_vec_pretty(&packet).unwrap(),
    );
    output
}

fn expected_source(config: &Value, invalid: bool, prefix: &str) -> Value {
    let column = 11 + prefix.encode_utf16().count();
    let diagnostics = if invalid {
        // The existing context scanner ends its first tag at a preceding HTML
        // comment; retain that complete rendering rather than filtering it.
        let binding = if prefix.is_empty() {
            "; binding: 'value'"
        } else {
            ""
        };
        json!([cstr!(
            "error:7:{column} [TS2322] {MESSAGE} (source: {prefix}<Comma :value=\"value\" />{binding})"
        )])
    } else {
        json!([])
    };
    json!({
        "files":[
            {"file":"src/App.vue","diagnostics":diagnostics},
            {"file":"src/Comma.vue","diagnostics":[]},
            {"file":"src/Edges.vue","diagnostics":[]},
        ],
        "programs":[{
            "root":".","tsconfig":"tsconfig.native.json",
            "compilerOptions":config["compilerOptions"],
            "files":["src/App.vue","src/Comma.vue","src/Edges.vue"],
        }],
        "errorCount":usize::from(invalid),"warningCount":0,"fileCount":3,
    })
}

fn executable_sha256(path: &Path) -> CompactString {
    let mut file = std::fs::File::open(path).unwrap();
    let mut hash = Sha256::new();
    let mut bytes = [0; 64 * 1024];
    loop {
        let length = file.read(&mut bytes).unwrap();
        if length == 0 {
            break;
        }
        hash.update(bytes.get(..length).unwrap());
    }
    hash.finalize()
        .iter()
        .map(|byte| cstr!("{byte:02x}"))
        .collect()
}

#[test]
fn quoted_generic_parameters_keep_complete_cli_and_official_host_packets_through_repairs() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact quoted-generic host matrix: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} must name the pinned executable");
    let tsgo = tsgo.canonicalize().unwrap();
    let cli = Path::new(env!("CARGO_BIN_EXE_vize"))
        .canonicalize()
        .unwrap();
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/quoted-generic-parameters-10540");
    let pack: Value = serde_json::from_str(PACK).unwrap();
    assert_eq!(pack["historicalIssue"], 10540);
    assert_eq!(pack["cases"].as_array().unwrap().len(), 3);
    assert_eq!(pack["cases"][0]["id"], "literal-comma-valid");
    assert_eq!(pack["cases"][1]["id"], "numeric-prop-invalid");
    assert_eq!(pack["cases"][2]["id"], "repaired-literal-prop-valid");
    assert_eq!(pack["cases"][0]["diagnostics"], json!([]));
    assert_eq!(pack["cases"][2]["diagnostics"], json!([]));
    assert_eq!(
        pack["cases"][1]["diagnostics"],
        json!([{
            "file":"src/App.vue","line":7,"column":11,"severity":1,"code":2322,
            "message":MESSAGE,
        }])
    );
    let config: Value =
        serde_json::from_slice(&std::fs::read(fixture.join("tsconfig.json.txt")).unwrap()).unwrap();
    let mut mapper_config = config.clone();
    mapper_config["contentMappers"] = json!([{"package":"vize","extensions":[".vue"]}]);
    let artifacts = workspace_root()
        .join("target/vize-tests/content-mapper-quoted-generic-parameters")
        .join(cstr!("{}", std::process::id()).as_str());
    std::fs::create_dir_all(&artifacts).unwrap();
    write(
        &artifacts,
        "provenance.json",
        &serde_json::to_vec_pretty(&json!({
            "sourceCli":cli,"sourceCliSha256":executable_sha256(&cli),
            "officialTsgo":tsgo,"officialTsgoSha256":executable_sha256(&tsgo),
            "pack":pack,"packBytes":PACK.as_bytes(),
        }))
        .unwrap(),
    );
    let mut observations = Vec::new();
    for (id, newline, prefix) in [
        ("lf", "\n", ""),
        ("crlf", "\r\n", ""),
        ("unicode-lf", "\n", "<!--🧭-->"),
        ("unicode-crlf", "\r\n", "<!--🧭-->"),
    ] {
        let variant = artifacts.join(id);
        std::fs::create_dir_all(&variant).unwrap();
        let project = tempfile::Builder::new()
            .prefix("project-")
            .tempdir_in(&variant)
            .unwrap();
        write(
            project.path(),
            "tsconfig.json",
            &serde_json::to_vec_pretty(&mapper_config).unwrap(),
        );
        write(
            project.path(),
            "tsconfig.native.json",
            &serde_json::to_vec_pretty(&config).unwrap(),
        );
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        for (phase, case_index, invalid) in [
            ("valid", 0, false),
            ("invalid", 1, true),
            ("repaired", 2, false),
            ("invalid-again", 1, true),
        ] {
            materialize(
                project.path(),
                &fixture,
                &pack["cases"][case_index],
                newline,
                prefix,
            );
            let capture = variant.join(phase);
            let mut mapper = Command::new(&tsgo);
            mapper.current_dir(project.path()).args([
                "--runExternalCode",
                "--noEmit",
                "-p",
                "tsconfig.json",
                "--pretty",
                "false",
            ]);
            let mapper = observe(&mut mapper, project.path(), &capture, "official-mapper");
            let mut source = Command::new(&cli);
            source
                .current_dir(project.path())
                .args([
                    "check",
                    "--no-config",
                    "--tsconfig",
                    "tsconfig.native.json",
                    "src",
                    "--format",
                    "json",
                    "--quiet",
                    "--corsa-path",
                ])
                .arg(&tsgo);
            let source = observe(&mut source, project.path(), &capture, "source-cli");
            observations.push(Observation {
                id: id.into(),
                phase,
                invalid,
                prefix,
                mapper,
                source,
            });
        }
    }
    // All 32 complete raw packets exist even if the first comparison fails.
    assert_eq!(observations.len(), 16);
    for observation in observations {
        let Observation {
            id,
            phase,
            invalid,
            prefix,
            mapper,
            source,
        } = observation;
        assert_eq!(
            mapper.status.code(),
            Some(if invalid { 2 } else { 0 }),
            "{id}/{phase}: {}",
            output_text(&mapper)
        );
        assert_eq!(mapper.stderr, b"", "{id}/{phase}: {}", output_text(&mapper));
        let expected_mapper = if invalid {
            let column = 11 + prefix.encode_utf16().count();
            cstr!("src/App.vue(7,{column}): error TS2322: {MESSAGE}\n")
        } else {
            CompactString::default()
        };
        assert_eq!(
            std::str::from_utf8(&mapper.stdout).unwrap(),
            expected_mapper,
            "{id}/{phase}"
        );
        assert_eq!(
            source.status.code(),
            Some(i32::from(invalid)),
            "{id}/{phase}: {}",
            output_text(&source)
        );
        assert_eq!(source.stderr, b"", "{id}/{phase}: {}", output_text(&source));
        assert_eq!(
            serde_json::from_slice::<Value>(&source.stdout).unwrap(),
            expected_source(&config, invalid, prefix),
            "{id}/{phase}: {}",
            output_text(&source)
        );
    }
}

//! Independent authored body errors survive the malformed-generic frontier.

use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use serde_json::{Value, json};
use vize_l0::{String as CompactString, cstr};

use super::super::{
    TSGO_ENV, install_mapper_manifest, install_vue_package, output_text, workspace_root,
};
use super::{executable_sha256, write};

const WORKER_ROOT: &str = "VIZE_TEST_GENERIC_BODY_REFERENCE_ROOT";
const WORKER_RESULT: &str = "VIZE_TEST_GENERIC_BODY_REFERENCE_RESULT";
const TEST: &str = "content_mapper_tsgo_cli::quoted_generic_parameters::body_error::independent_authored_body_errors_survive_generic_repairs";
const SEMANTIC: &str = "Type 'string' is not assignable to type 'number'.";
const INPUTS: [&str; 9] = [
    "tsconfig.json",
    "tsconfig.native.json",
    "reference/tsconfig.json",
    "reference/probe.ts",
    "node_modules/vize/package.json",
    "node_modules/vue/package.json",
    "src/App.vue",
    "src/Comma.vue",
    "src/Edges.vue",
];

struct Observation {
    id: &'static str,
    phase: &'static str,
    broken: bool,
    body_error: bool,
    column: usize,
    reference_column: usize,
    cli: Output,
    session: Output,
    mapper: Output,
    source: Output,
    session_result: Vec<u8>,
    reference_root: PathBuf,
}

fn capture(command: &mut Command, root: &Path, target: &Path, host: &str) -> Output {
    let argv = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|argument| CompactString::from(argument.to_string_lossy().as_ref()))
        .collect::<Vec<_>>();
    let environment = command
        .get_envs()
        .map(|(key, value)| {
            (
                CompactString::from(key.to_string_lossy().as_ref()),
                value.map(|value| CompactString::from(value.to_string_lossy().as_ref())),
            )
        })
        .collect::<Vec<_>>();
    let inputs = INPUTS
        .iter()
        .map(|name| json!({"file":name,"bytes":std::fs::read(root.join(name)).unwrap()}))
        .collect::<Vec<_>>();
    let output = command.output().unwrap();
    write(target, cstr!("{host}.stdout.raw").as_str(), &output.stdout);
    write(target, cstr!("{host}.stderr.raw").as_str(), &output.stderr);
    write(
        target,
        cstr!("{host}.json").as_str(),
        &serde_json::to_vec_pretty(&json!({
            "schema":"vize.generic-body-error-host-observation","version":1,"host":host,
            "argv":argv,"environment":environment,"cwd":root,"status":output.status.code(),
            "success":output.status.success(),"statusText":cstr!("{}",output.status),
            "stdout":output.stdout,"stderr":output.stderr,"inputs":inputs,
        }))
        .unwrap(),
    );
    output
}

fn reference_worker(root: &Path) {
    let native = std::env::var(TSGO_ENV).unwrap();
    let root = root.canonicalize().unwrap();
    let uri = url::Url::from_file_path(root.join("probe.ts")).unwrap();
    let client =
        vize_canon::corsa_client::CorsaProjectClient::new_for_workspace(Some(&native), &root);
    let startup = client.as_ref().map(|_| ()).map_err(Clone::clone);
    let (diagnostics, shutdown) = match client {
        Ok(mut client) => {
            let diagnostics = client.request_diagnostics(uri.as_str());
            let shutdown = client.shutdown();
            (json!(diagnostics), json!(shutdown))
        }
        Err(_) => (Value::Null, Value::Null),
    };
    let packet = json!({"worker":TEST,"root":root,"fileURI":uri.as_str(),
        "configBytes":std::fs::read(root.join("tsconfig.json")).unwrap(),
        "startup":startup,"diagnostics":diagnostics,"shutdown":shutdown});
    std::fs::write(
        std::env::var_os(WORKER_RESULT).unwrap(),
        serde_json::to_vec_pretty(&packet).unwrap(),
    )
    .unwrap();
}

fn syntax_oracle(output: &Output, column: usize) -> (u32, &str) {
    let prefix = cstr!("reference/probe.ts(1,{column}): error TS");
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let (code, message) = stdout
        .strip_prefix(prefix.as_str())
        .unwrap()
        .strip_suffix('\n')
        .unwrap()
        .split_once(": ")
        .unwrap();
    let code = code.parse::<u32>().unwrap();
    assert_eq!(code / 1000, 1);
    assert_ne!(message, "");
    assert_eq!(
        message.split(['\r', '\n']).collect::<Vec<_>>(),
        vec![message]
    );
    assert_eq!(stdout, cstr!("{prefix}{code}: {message}\n"));
    (code, message)
}

fn expected_reference(
    broken: bool,
    body_error: bool,
    column: usize,
    syntax: Option<(u32, &str)>,
) -> Value {
    let mut rows = Vec::new();
    if broken {
        let (code, message) = syntax.unwrap();
        rows.push(json!({"range":{"start":{"line":0,"character":column-1},"end":{"line":0,"character":column}},
            "severity":1,"code":code,"source":"ts","message":message}));
    }
    if body_error {
        rows.push(
            json!({"range":{"start":{"line":1,"character":6},"end":{"line":1,"character":17}},
            "severity":1,"code":2322,"source":"ts","message":SEMANTIC}),
        );
    }
    json!(rows)
}

fn expected_source(config: &Value, rows: &Value, syntax_column: usize) -> Value {
    let diagnostics = rows
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let line = row["range"]["start"]["line"].as_u64().unwrap() + 1;
            let column = if line == 1 {
                syntax_column as u64
            } else {
                row["range"]["start"]["character"].as_u64().unwrap() + 1
            };
            cstr!(
                "error:{line}:{column} [TS{}] {}",
                row["code"].as_u64().unwrap(),
                row["message"].as_str().unwrap()
            )
        })
        .collect::<Vec<_>>();
    json!({"files":[{"file":"src/App.vue","diagnostics":diagnostics},
        {"file":"src/Comma.vue","diagnostics":[]},{"file":"src/Edges.vue","diagnostics":[]}],
        "programs":[{"root":".","tsconfig":"tsconfig.native.json","compilerOptions":config["compilerOptions"],
            "files":["src/App.vue","src/Comma.vue","src/Edges.vue"]}],
        "errorCount":rows.as_array().unwrap().len(),"warningCount":0,"fileCount":3})
}

#[test]
fn independent_authored_body_errors_survive_generic_repairs() {
    if let Some(root) = std::env::var_os(WORKER_ROOT) {
        reference_worker(Path::new(&root));
        return;
    }
    let Some(native) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping authored-body-error host matrix: {TSGO_ENV} is not set");
        return;
    };
    let native = native.canonicalize().unwrap();
    let cli = Path::new(env!("CARGO_BIN_EXE_vize"))
        .canonicalize()
        .unwrap();
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/quoted-generic-parameters-10540");
    let config: Value =
        serde_json::from_slice(&std::fs::read(fixture.join("tsconfig.json.txt")).unwrap()).unwrap();
    let mut mapper_config = config.clone();
    mapper_config["contentMappers"] = json!([{"package":"vize","extensions":[".vue"]}]);
    let reference_config = serde_json::to_vec_pretty(
        &json!({"compilerOptions":config["compilerOptions"],"files":["probe.ts"]}),
    )
    .unwrap();
    let artifacts = workspace_root()
        .join("target/vize-tests/content-mapper-quoted-generic-parameters")
        .join(cstr!("{}", std::process::id()).as_str())
        .join("body-error");
    write(&artifacts, "provenance.json", &serde_json::to_vec_pretty(&json!({
        "sourceCli":cli,"sourceCliSha256":executable_sha256(&cli),"native":native,
        "nativeSha256":executable_sha256(&native),"referenceConfigBytes":reference_config,
        "sessionBoundary":"public CorsaProjectClient on handwritten plain TypeScript; whole public DTO, not raw RPC"})).unwrap());
    let mut observations = Vec::new();
    for (id, newline, prefix, reference_prefix) in [
        ("lf", "\n", "", ""),
        ("crlf", "\r\n", "", ""),
        ("unicode-lf", "\n", "<!--🧭-->", "/*🧭*/"),
        ("unicode-crlf", "\r\n", "<!--🧭-->", "/*🧭*/"),
    ] {
        let variant = artifacts.join(id);
        std::fs::create_dir_all(&variant).unwrap();
        let project = tempfile::Builder::new()
            .prefix("project-")
            .tempdir_in(&variant)
            .unwrap();
        for (name, value) in [
            ("tsconfig.json", &mapper_config),
            ("tsconfig.native.json", &config),
        ] {
            write(
                project.path(),
                name,
                &serde_json::to_vec_pretty(value).unwrap(),
            );
        }
        write(project.path(), "reference/tsconfig.json", &reference_config);
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        for name in ["Comma", "Edges"] {
            let source =
                std::fs::read_to_string(fixture.join(cstr!("src/{name}.vue.txt").as_str()))
                    .unwrap();
            write(
                project.path(),
                cstr!("src/{name}.vue").as_str(),
                source.replace('\n', newline).as_bytes(),
            );
        }
        for (phase, broken, body_error) in [
            ("valid-generic-body-error", false, true),
            ("broken-generic-body-error", true, true),
            ("repaired-generic-body-error", false, true),
            ("both-repaired", false, false),
        ] {
            let generic = if broken {
                "T extends"
            } else {
                "T extends string"
            };
            let value = if body_error { "'bad'" } else { "1" };
            let body = cstr!(
                "const independent: number = {value};\nvoid independent;\nconst genericValue: T | undefined = undefined;\nvoid genericValue;\n"
            );
            let authored = cstr!(
                "{prefix}<script setup lang=\"ts\" generic=\"{generic}\">\n{body}</script>\n<template><div /></template>\n"
            );
            let end = authored.find("generic=\"").unwrap() + "generic=\"".len() + generic.len();
            let column = authored.get(..end).unwrap().encode_utf16().count() + 1;
            write(
                project.path(),
                "src/App.vue",
                authored.replace('\n', newline).as_bytes(),
            );
            write(
                project.path(),
                "reference/probe.ts",
                cstr!("{reference_prefix}function probe<{generic}>() {{\n{body}}}\n")
                    .replace('\n', newline)
                    .as_bytes(),
            );
            let reference_column = 25 + reference_prefix.encode_utf16().count();
            let target = variant.join(phase);
            let mut reference = Command::new(&native);
            reference.current_dir(project.path()).args([
                "--noEmit",
                "-p",
                "reference/tsconfig.json",
                "--pretty",
                "false",
            ]);
            let reference_cli = capture(
                &mut reference,
                project.path(),
                &target,
                "plain-typescript-cli",
            );
            let session_result = target.join("reference-session-result.json");
            let reference_root = project.path().join("reference").canonicalize().unwrap();
            let mut session = Command::new(std::env::current_exe().unwrap());
            session
                .current_dir(project.path())
                .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
                .env(WORKER_ROOT, &reference_root)
                .env(WORKER_RESULT, &session_result)
                .env(TSGO_ENV, &native);
            let session = capture(
                &mut session,
                project.path(),
                &target,
                "plain-typescript-session-worker",
            );
            let session_result = std::fs::read(session_result).unwrap_or_default();
            let mut mapper = Command::new(&native);
            mapper.current_dir(project.path()).args([
                "--runExternalCode",
                "--noEmit",
                "-p",
                "tsconfig.json",
                "--pretty",
                "false",
            ]);
            let mapper = capture(&mut mapper, project.path(), &target, "official-mapper");
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
                .arg(&native);
            let source = capture(&mut source, project.path(), &target, "source-cli");
            observations.push(Observation {
                id,
                phase,
                broken,
                body_error,
                column,
                reference_column,
                cli: reference_cli,
                session,
                mapper,
                source,
                session_result,
                reference_root,
            });
        }
    }
    // Each of the 16 physical-project states owns four complete captured hosts.
    assert_eq!(observations.len(), 16);
    for Observation {
        id,
        phase,
        broken,
        body_error,
        column,
        reference_column,
        cli,
        session,
        mapper,
        source,
        session_result,
        reference_root,
    } in observations
    {
        assert_eq!(
            cli.status.code(),
            Some(if broken || body_error { 2 } else { 0 }),
            "{id}/{phase}: {}",
            output_text(&cli)
        );
        assert_eq!(cli.stderr, b"", "{id}/{phase}: {}", output_text(&cli));
        let syntax = broken.then(|| syntax_oracle(&cli, reference_column));
        if !broken {
            let expected = if body_error {
                cstr!("reference/probe.ts(2,7): error TS2322: {SEMANTIC}\n")
            } else {
                CompactString::default()
            };
            assert_eq!(
                std::str::from_utf8(&cli.stdout).unwrap(),
                expected,
                "{id}/{phase}"
            );
        }
        assert_eq!(
            session.status.code(),
            Some(0),
            "{id}/{phase}: {}",
            output_text(&session)
        );
        assert_eq!(
            session.stderr,
            b"",
            "{id}/{phase}: {}",
            output_text(&session)
        );
        let reference_rows = expected_reference(broken, body_error, reference_column, syntax);
        let reference: Value = serde_json::from_slice(&session_result).unwrap();
        assert_eq!(
            reference,
            json!({"worker":TEST,"root":reference_root,
            "fileURI":url::Url::from_file_path(reference_root.join("probe.ts")).unwrap().as_str(),
            "configBytes":reference_config,"startup":{"Ok":null},"diagnostics":{"Ok":reference_rows},"shutdown":{"Ok":null}}),
            "{id}/{phase}"
        );
        assert_eq!(
            mapper.status.code(),
            Some(if broken || body_error { 2 } else { 0 }),
            "{id}/{phase}: {}",
            output_text(&mapper)
        );
        assert_eq!(mapper.stderr, b"", "{id}/{phase}: {}", output_text(&mapper));
        let expected_mapper = if let Some((code, message)) = syntax {
            cstr!("src/App.vue(1,{column}): error TS{code}: {message}\n")
        } else if body_error {
            cstr!("src/App.vue(2,7): error TS2322: {SEMANTIC}\n")
        } else {
            CompactString::default()
        };
        assert_eq!(
            std::str::from_utf8(&mapper.stdout).unwrap(),
            expected_mapper,
            "{id}/{phase}: parser-frontier whole vector"
        );
        assert_eq!(
            source.status.code(),
            Some(i32::from(broken || body_error)),
            "{id}/{phase}: {}",
            output_text(&source)
        );
        assert_eq!(source.stderr, b"", "{id}/{phase}: {}", output_text(&source));
        assert_eq!(
            serde_json::from_slice::<Value>(&source.stdout).unwrap(),
            expected_source(&config, &reference["diagnostics"]["Ok"], column),
            "{id}/{phase}: {}",
            output_text(&source)
        );
    }
}

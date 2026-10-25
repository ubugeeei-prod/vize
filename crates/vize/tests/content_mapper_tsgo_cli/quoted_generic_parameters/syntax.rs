//! A handwritten plain-TS provider oracle owns malformed generic syntax codes.

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
    column: usize,
    reference_column: usize,
    reference: Output,
    mapper: Output,
    source: Output,
}

fn capture(command: &mut Command, root: &Path, target: &Path, host: &str, range: &Value) -> Output {
    let argv = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|argument| CompactString::from(argument.to_string_lossy().as_ref()))
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
            "schema":"vize.quoted-generic-syntax-host-observation","version":1,
            "host":host,"argv":argv,"cwd":root,"status":output.status.code(),
            "success":output.status.success(),"statusText":cstr!("{}",output.status),
            "stdout":output.stdout,"stderr":output.stderr,"inputs":inputs,
            "expectedAuthoredRange":range,
        }))
        .unwrap(),
    );
    output
}

/// Only the independent provider run supplies the numeric syntax code/message.
/// Its whole output must identify one error at the handwritten missing type.
fn reference_error(output: &Output, column: usize) -> (u32, &str) {
    assert_eq!(output.status.code(), Some(2), "{}", output_text(output));
    assert_eq!(output.stderr, b"", "{}", output_text(output));
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let prefix = cstr!("reference/probe.ts(1,{column}): error TS");
    let diagnostic = stdout
        .strip_prefix(prefix.as_str())
        .unwrap()
        .strip_suffix('\n')
        .unwrap();
    let (code, message) = diagnostic.split_once(": ").unwrap();
    assert!(code.bytes().all(|byte| byte.is_ascii_digit()));
    let code = code.parse::<u32>().unwrap();
    assert_eq!(code / 1000, 1, "a provider syntax code is required");
    assert_ne!(message, "");
    assert_eq!(
        message.split(['\r', '\n']).collect::<Vec<_>>(),
        vec![message]
    );
    assert_eq!(stdout, cstr!("{prefix}{code}: {message}\n"));
    (code, message)
}

fn expected_source(config: &Value, column: usize, diagnostic: Option<(u32, &str)>) -> Value {
    let diagnostics = diagnostic.map_or_else(
        || json!([]),
        |(code, message)| json!([cstr!("error:1:{column} [TS{code}] {message}")]),
    );
    json!({
        "files":[
            {"file":"src/App.vue","diagnostics":diagnostics},
            {"file":"src/Comma.vue","diagnostics":[]},
            {"file":"src/Edges.vue","diagnostics":[]},
        ],
        "programs":[{"root":".","tsconfig":"tsconfig.native.json",
            "compilerOptions":config["compilerOptions"],
            "files":["src/App.vue","src/Comma.vue","src/Edges.vue"]}],
        "errorCount":usize::from(diagnostic.is_some()),"warningCount":0,"fileCount":3,
    })
}

#[test]
fn malformed_generic_attributes_keep_provider_syntax_codes_and_authored_ranges_through_repairs() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping reference-led generic syntax matrix: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} must name the pinned executable");
    let tsgo = tsgo.canonicalize().unwrap();
    let cli = Path::new(env!("CARGO_BIN_EXE_vize"))
        .canonicalize()
        .unwrap();
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/quoted-generic-parameters-10540");
    let config: Value =
        serde_json::from_slice(&std::fs::read(fixture.join("tsconfig.json.txt")).unwrap()).unwrap();
    let mut mapper_config = config.clone();
    mapper_config["contentMappers"] = json!([{"package":"vize","extensions":[".vue"]}]);
    let artifacts = workspace_root()
        .join("target/vize-tests/content-mapper-quoted-generic-parameters")
        .join(cstr!("{}", std::process::id()).as_str())
        .join("syntax");
    write(
        &artifacts,
        "provenance.json",
        &serde_json::to_vec_pretty(&json!({
            "sourceCli":cli,"sourceCliSha256":executable_sha256(&cli),
            "officialTsgo":tsgo,"officialTsgoSha256":executable_sha256(&tsgo),
            "oracle":"handwritten function probe<T extends>() {} at its missing constraint",
        }))
        .unwrap(),
    );
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
        write(
            project.path(),
            "reference/tsconfig.json",
            &serde_json::to_vec_pretty(&json!({
                "compilerOptions":config["compilerOptions"],"files":["probe.ts"],
            }))
            .unwrap(),
        );
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
        for (phase, broken) in [
            ("valid", false),
            ("broken", true),
            ("repaired", false),
            ("broken-again", true),
        ] {
            let generic = if broken {
                "T extends"
            } else {
                "T extends string"
            };
            let authored = cstr!(
                "{prefix}<script setup lang=\"ts\" generic=\"{generic}\">\ndefineProps<{{ value?: T }}>()\n</script>\n<template><div /></template>\n"
            );
            let end = authored.find("generic=\"").unwrap() + "generic=\"".len() + generic.len();
            assert_eq!(authored.as_bytes().get(end), Some(&b'"'));
            let column = authored.get(..end).unwrap().encode_utf16().count() + 1;
            write(
                project.path(),
                "src/App.vue",
                authored.replace('\n', newline).as_bytes(),
            );
            let reference = cstr!("{reference_prefix}function probe<{generic}>() {{}}\n");
            write(
                project.path(),
                "reference/probe.ts",
                reference.replace('\n', newline).as_bytes(),
            );
            let reference_column = 25 + reference_prefix.encode_utf16().count();
            let range = json!({"file":"src/App.vue","start":end,"length":1,
                "line":1,"column":column,"token":"\""});
            let target = variant.join(phase);
            let mut reference = Command::new(&tsgo);
            reference.current_dir(project.path()).args([
                "--noEmit",
                "-p",
                "reference/tsconfig.json",
                "--pretty",
                "false",
            ]);
            let reference = capture(
                &mut reference,
                project.path(),
                &target,
                "plain-typescript",
                &range,
            );
            let mut mapper = Command::new(&tsgo);
            mapper.current_dir(project.path()).args([
                "--runExternalCode",
                "--noEmit",
                "-p",
                "tsconfig.json",
                "--pretty",
                "false",
            ]);
            let mapper = capture(
                &mut mapper,
                project.path(),
                &target,
                "official-mapper",
                &range,
            );
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
            let source = capture(&mut source, project.path(), &target, "source-cli", &range);
            observations.push(Observation {
                id,
                phase,
                broken,
                column,
                reference_column,
                reference,
                mapper,
                source,
            });
        }
    }
    // Preserve all 48 independent raw packets before comparing any host result.
    assert_eq!(observations.len(), 16);
    for Observation {
        id,
        phase,
        broken,
        column,
        reference_column,
        reference,
        mapper,
        source,
    } in observations
    {
        let diagnostic = if broken {
            Some(reference_error(&reference, reference_column))
        } else {
            assert_eq!(
                reference.status.code(),
                Some(0),
                "{id}/{phase}: {}",
                output_text(&reference)
            );
            assert_eq!(
                reference.stdout,
                b"",
                "{id}/{phase}: {}",
                output_text(&reference)
            );
            assert_eq!(
                reference.stderr,
                b"",
                "{id}/{phase}: {}",
                output_text(&reference)
            );
            None
        };
        assert_eq!(
            mapper.status.code(),
            Some(if broken { 2 } else { 0 }),
            "{id}/{phase}: {}",
            output_text(&mapper)
        );
        assert_eq!(mapper.stderr, b"", "{id}/{phase}: {}", output_text(&mapper));
        let expected_mapper = diagnostic.map_or_else(CompactString::default, |(code, message)| {
            cstr!("src/App.vue(1,{column}): error TS{code}: {message}\n")
        });
        assert_eq!(
            std::str::from_utf8(&mapper.stdout).unwrap(),
            expected_mapper,
            "{id}/{phase}"
        );
        assert_eq!(
            source.status.code(),
            Some(i32::from(broken)),
            "{id}/{phase}: {}",
            output_text(&source)
        );
        assert_eq!(source.stderr, b"", "{id}/{phase}: {}", output_text(&source));
        assert_eq!(
            serde_json::from_slice::<Value>(&source.stdout).unwrap(),
            expected_source(&config, column, diagnostic),
            "{id}/{phase}: {}",
            output_text(&source)
        );
    }
}

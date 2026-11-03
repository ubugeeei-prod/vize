//! Consume physically emitted declarations with the actual workspace vue-tsc.

use super::{inputs, process, project, providers, retain, write};
use crate::content_mapper_tsgo_cli::{TSGO_ENV, workspace_root};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use vize_l0::String;

fn outputs(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    if root.is_dir() {
        for entry in fs::read_dir(root).unwrap() {
            let entry = entry.unwrap();
            assert!(entry.file_type().unwrap().is_file());
            entries.push((
                entry.file_name().into_string().unwrap().into(),
                fs::read(entry.path()).unwrap(),
            ));
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

fn sites(bytes: &[u8]) -> Vec<(u32, u32, u32)> {
    let mut result = Vec::new();
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if line.starts_with("  ") {
            continue;
        }
        let (site, message) = line
            .split_once("): error TS")
            .expect("every non-continuation line is a diagnostic");
        let coordinates = site
            .strip_prefix("src/Probe.vue(")
            .expect("diagnostic belongs to authored consumer");
        let (line, column) = coordinates.split_once(',').unwrap();
        let (code, _) = message.split_once(':').unwrap();
        result.push((
            line.parse().unwrap(),
            column.parse().unwrap(),
            code.parse().unwrap(),
        ));
    }
    result
}

#[test]
fn emitted_declarations_accept_fallthrough_without_erasing_owned_inputs() {
    let Some(host) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        return;
    };
    assert!(host.is_file());
    let vue_tsc = workspace_root().join("tests/node_modules/.bin/vue-tsc");
    assert!(vue_tsc.is_file(), "existing workspace vue-tsc is required");
    let input = inputs();
    let package = input["declarationPackage"].as_str().unwrap();
    let config = input["declarationConfig"].as_str().unwrap();
    let original = input["declarationSource"].as_str().unwrap();
    let consumer = input["declarationConsumer"].as_str().unwrap();
    let two_props = original
        .replace("bar: number", "bar: number; baz: number")
        .replace("defineEmits<{ change: [] }>();\n", "");
    let cases = vec![
        (
            "original-emits",
            original.to_owned(),
            consumer.to_owned(),
            vec![],
        ),
        (
            "one-prop-no-emits",
            original.replace("defineEmits<{ change: [] }>();\n", ""),
            consumer.to_owned(),
            vec![],
        ),
        (
            "two-required",
            two_props.clone(),
            consumer.replace(":bar=\"1\"", ":bar=\"1\" :baz=\"2\""),
            vec![],
        ),
        (
            "wrong-owned-prop",
            original.to_owned(),
            consumer.replace(":bar=\"1\" disabled", "bar=\"baz\""),
            vec![(6, 8, 2322)],
        ),
        (
            "missing-bar",
            original.to_owned(),
            consumer.replace(":bar=\"1\" ", ""),
            vec![(6, 4, 2345)],
        ),
        (
            "missing-baz",
            two_props,
            consumer.to_owned(),
            vec![(6, 4, 2345)],
        ),
        (
            "wrong-owned-listener",
            original.to_owned(),
            consumer.replace(" disabled", " onChange=\"wrong\" disabled"),
            vec![(6, 17, 2322)],
        ),
        (
            "repaired-original",
            original.to_owned(),
            consumer.to_owned(),
            vec![],
        ),
    ];
    let mut observations = Vec::new();
    let mut verdicts = Vec::new();
    for (id, source, consumer_source, expected) in cases {
        let dir = tempfile::Builder::new()
            .prefix("reported-fallthrough-11454-")
            .tempdir()
            .unwrap();
        let producer = dir.path().join("producer");
        let consumer = dir.path().join("consumer");
        project(&producer, package, config);
        project(&consumer, package, config);
        write(&producer.join("src/Foo.vue"), source.as_bytes());
        write(&consumer.join("src/Probe.vue"), consumer_source.as_bytes());
        let emitted = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(&producer)
            .env("CORSA_PATH", &host)
            .env("VIZE_TEST_REQUIRE_TSGO", "1")
            .env_remove("VIZE_TEST_DISABLE_TSGO")
            .args([
                "check",
                "--declaration",
                "--declaration-dir",
                "../out",
                "src/Foo.vue",
            ])
            .output()
            .unwrap();
        let actual_files = outputs(&dir.path().join("out"));
        for (name, bytes) in &actual_files {
            write(&consumer.join("src/vendor").join(name), bytes);
        }
        let actual = Command::new(&vue_tsc)
            .current_dir(&consumer)
            .args(["--noEmit", "-p", "tsconfig.json", "--pretty", "false"])
            .output()
            .unwrap();
        let reference_emit = Command::new(&vue_tsc)
            .current_dir(&producer)
            .args([
                "--declaration",
                "--emitDeclarationOnly",
                "--noEmit",
                "false",
                "--rootDir",
                "src",
                "--outDir",
                "../reference-out",
                "-p",
                "tsconfig.json",
                "--pretty",
                "false",
            ])
            .output()
            .unwrap();
        let reference_files = outputs(&dir.path().join("reference-out"));
        for (name, bytes) in &reference_files {
            write(&consumer.join("src/reference").join(name), bytes);
        }
        let reference_source = consumer_source.replace("./vendor/Foo.vue", "./reference/Foo.vue");
        write(&consumer.join("src/Probe.vue"), reference_source.as_bytes());
        let reference = Command::new(&vue_tsc)
            .current_dir(&consumer)
            .args(["--noEmit", "-p", "tsconfig.json", "--pretty", "false"])
            .output()
            .unwrap();
        observations.push(json!({"id":id,"source":source,"consumerSource":consumer_source,"referenceConsumerSource":reference_source,"package":package,"config":config,"cwd":dir.path(),"providers":providers(&producer),"vizeExecutable":env!("CARGO_BIN_EXE_vize"),"vueTscExecutable":vue_tsc,"sourceEmit":process(&emitted),"actualFiles":actual_files,"actualConsumer":process(&actual),"referenceEmit":process(&reference_emit),"referenceFiles":reference_files,"referenceConsumer":process(&reference)}));
        verdicts.push((
            id,
            emitted,
            actual_files,
            actual,
            reference_emit,
            reference_files,
            reference,
            expected,
        ));
    }
    retain("fallthrough-11454", &host, json!(observations));
    for (id, emitted, files, actual, reference_emit, reference_files, reference, expected) in
        verdicts
    {
        assert_eq!(emitted.status.code(), Some(0), "{id}");
        assert_eq!(
            files
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            vec!["Foo.vue.d.ts", "__vize_helpers.d.ts"],
            "{id}"
        );
        assert_eq!(reference_emit.status.code(), Some(0), "{id}");
        assert_eq!(
            reference_files
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            vec!["Foo.vue.d.ts"],
            "{id}"
        );
        for output in [&actual, &reference] {
            assert_eq!(
                output.status.code(),
                Some(if expected.is_empty() { 0 } else { 2 }),
                "{id}"
            );
            assert_eq!(output.stderr.as_slice(), b"", "{id}");
            assert_eq!(sites(&output.stdout), expected, "{id}");
            if expected.is_empty() {
                assert_eq!(output.stdout.as_slice(), b"", "{id}");
            }
        }
    }
}

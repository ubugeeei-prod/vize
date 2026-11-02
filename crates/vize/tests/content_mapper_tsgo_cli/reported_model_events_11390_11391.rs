//! Original model-event reports and source-owned boundary/payload controls.

use std::{path::PathBuf, time::Instant};

use serde_json::{Value, json};

use super::TSGO_ENV;
use super::support::{check_project, install_mapper_manifest, install_vue_package, workspace_root};

fn input() -> Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/reported-model-events-11390-11391/input.json"
    )))
    .unwrap()
}

fn listener(key: &str, payload: &str) -> String {
    format!(
        "import Probe from './Probe.vue';\nconst handler = (value: {payload}) => {{ void value; }};\nconst listener: InstanceType<typeof Probe>[\"$props\"][\"{key}\"] = handler;\nvoid listener;\n"
    )
}

fn scaled_component(emits: usize, models: usize) -> String {
    let mut source = String::from("<script setup lang=\"ts\">\ndefineEmits<{\n");
    for index in 0..emits {
        source.push_str(&format!("e{index}: [value: number];\n"));
    }
    source.push_str("}>();\n");
    for index in 0..models {
        source.push_str(&format!("defineModel<number>(\"m{index}\");\n"));
    }
    source.push_str("</script>\n<template><div /></template>\n");
    source
}

#[test]
fn original_model_event_maps_preserve_boundaries_and_payload_errors() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping original model-event controls: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_absolute() && tsgo.is_file());
    let fixture = input();
    let typed = fixture["typedSeventeen"].as_str().unwrap();
    let runtime = fixture["runtimeArray"].as_str().unwrap();
    let numbered_runtime = runtime.replace(
        "defineModel();",
        "defineModel<number>(\"count\", { required: true });",
    );
    let typed_sentinel = typed.replace(
        "defineEmits<{ e0: [value: number] }>();\n",
        "defineEmits<{ e0: [value: number] }>();\nconst probeText: number = \"a\"; void probeText;\n",
    );
    let runtime_sentinel = runtime.replace(
        "defineModel();\n",
        "defineModel();\nconst probeText: number = \"a\"; void probeText;\n",
    );
    let mut runtime_seventeen = runtime.replace("defineModel();\n", "");
    let models = (0..16)
        .map(|index| format!("defineModel<number>(\"m{index}\");\n"))
        .collect::<String>();
    runtime_seventeen = runtime_seventeen.replace("</script>", &format!("{models}</script>"));
    let cases = [
        ("typed-original-17", typed.to_owned(), None, None),
        (
            "typed-boundary-16",
            typed.replace("defineModel<number>(\"m15\");\n", ""),
            None,
            None,
        ),
        ("typed-no-model-24", scaled_component(24, 0), None, None),
        (
            "typed-generic-17",
            typed
                .replace("lang=\"ts\"", "lang=\"ts\" generic=\"T\"")
                .replace("value: number", "value: T")
                .replace("defineModel<number>", "defineModel<T>"),
            None,
            None,
        ),
        (
            "typed-payload-right",
            typed.to_owned(),
            Some(listener("onE0", "number")),
            None,
        ),
        (
            "typed-payload-wrong",
            typed.to_owned(),
            Some(listener("onE0", "string")),
            Some("src/Listener.ts(3,7): error TS2322"),
        ),
        (
            "typed-payload-repaired",
            typed.to_owned(),
            Some(listener("onE0", "number")),
            None,
        ),
        ("runtime-original-array", runtime.to_owned(), None, None),
        (
            "runtime-object",
            numbered_runtime.replace(
                "defineEmits([\"a\"])",
                "defineEmits({ a: (value: number) => typeof value === 'number' })",
            ),
            None,
            None,
        ),
        (
            "runtime-no-model",
            runtime.replace("defineModel();\n", ""),
            None,
            None,
        ),
        (
            "runtime-generic",
            runtime
                .replace("lang=\"ts\"", "lang=\"ts\" generic=\"T\"")
                .replace("defineModel()", "defineModel<T>()"),
            None,
            None,
        ),
        (
            "runtime-payload-right",
            numbered_runtime.clone(),
            Some(listener("onUpdate:count", "number")),
            None,
        ),
        (
            "runtime-payload-wrong",
            numbered_runtime.clone(),
            Some(listener("onUpdate:count", "string")),
            Some("src/Listener.ts(3,7): error TS2322"),
        ),
        (
            "runtime-payload-repaired",
            numbered_runtime,
            Some(listener("onUpdate:count", "number")),
            None,
        ),
        (
            "typed-authored-error",
            typed_sentinel,
            None,
            Some("src/Probe.vue(3,7): error TS2322"),
        ),
        (
            "runtime-authored-error",
            runtime_sentinel,
            None,
            Some("src/Probe.vue(4,7): error TS2322"),
        ),
        (
            "typed-representative-100",
            scaled_component(50, 50),
            None,
            None,
        ),
        ("runtime-boundary-17", runtime_seventeen, None, None),
        ("restored-typed-original", typed.to_owned(), None, None),
        ("restored-runtime-original", runtime.to_owned(), None, None),
    ];
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let mut observations = Vec::new();
    let mut verdicts = Vec::new();
    for (id, source, listener, expected_header) in cases {
        let project = tempfile::Builder::new()
            .prefix("reported-model-events-11390-11391-")
            .tempdir_in(&cases_root)
            .unwrap();
        let root = project.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("package.json"),
            fixture["packageJson"].as_str().unwrap(),
        )
        .unwrap();
        let mut config = fixture["tsconfig"].as_str().unwrap().to_owned();
        if let Some(listener) = &listener {
            std::fs::write(root.join("src/Listener.ts"), listener).unwrap();
            config = config.replace(
                "[\"src/Probe.vue\"]",
                "[\"src/Probe.vue\", \"src/Listener.ts\"]",
            );
        }
        std::fs::write(root.join("tsconfig.json"), &config).unwrap();
        std::fs::write(root.join("src/Probe.vue"), &source).unwrap();
        install_mapper_manifest(root);
        install_vue_package(root);
        let manifest = std::fs::canonicalize(root.join("node_modules/vue/package.json")).unwrap();
        let started = Instant::now();
        let output = check_project(&tsgo, root, "tsconfig.json");
        let elapsed = started.elapsed();
        observations.push(json!({
            "id": id, "cwd": root, "source": source, "listener": listener,
            "package": fixture["packageJson"], "config": config,
            "host": tsgo,
            "argv": ["--runExternalCode", "--noEmit", "-p", "tsconfig.json", "--pretty", "false"],
            "sourceRoleDeviations": ["upstream workflow host instead of reported tsrs --fast", "installed workflow Vue provider; manifest retained"],
            "vueManifestPath": manifest, "vueManifest": std::fs::read(&manifest).unwrap(),
            "mapperManifest": std::fs::read(root.join("node_modules/vize/package.json")).unwrap(),
            "status": output.status.code(), "osStatus": output.status.to_string(),
            "stdout": output.stdout, "stderr": output.stderr,
            "elapsedNanos": elapsed.as_nanos().to_string(),
            "expectedDiagnosticHeader": expected_header,
        }));
        verdicts.push((id, output, expected_header));
    }
    // Always retain all original, negative, repair and scale results before asserting.
    eprintln!(
        "reported-model-events-11390-11391: {}",
        serde_json::to_string(&observations).unwrap()
    );
    for (id, output, expected_header) in verdicts {
        assert_eq!(output.stderr, b"", "{id}");
        if let Some(expected) = expected_header {
            assert_eq!(output.status.code(), Some(2), "{id}");
            let stdout = std::str::from_utf8(&output.stdout).unwrap();
            let actual = stdout
                .lines()
                .filter_map(|line| {
                    let (position, diagnostic) = line.split_once(": error TS")?;
                    let (code, _) = diagnostic.split_once(':')?;
                    Some(format!("{position}: error TS{code}"))
                })
                .collect::<Vec<_>>();
            assert_eq!(actual, vec![expected], "{id}: {stdout}");
        } else {
            assert_eq!(output.status.code(), Some(0), "{id}");
            assert_eq!(output.stdout, b"", "{id}");
        }
    }
}

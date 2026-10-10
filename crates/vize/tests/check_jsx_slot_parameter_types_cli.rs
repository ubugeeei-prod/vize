#![cfg(test)]
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

#[test]
fn required_slot_callback_types_preserve_complete_native_cli_packets() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let vue = [
        "node_modules/vue",
        "tests/node_modules/vue",
        "examples/jsx-tsx/node_modules/vue",
    ]
    .map(|candidate| workspace.join(candidate))
    .into_iter()
    .find(|candidate| candidate.exists())
    .expect("required original Vue installation");
    let dependencies = vue.parent().unwrap();
    let fixtures = workspace
        .join("tests/_fixtures/differential/typecheck/jsx-slot-parameter-annotations-8419");
    let cases: Value =
        serde_json::from_str(&std::fs::read_to_string(fixtures.join("cases.json")).unwrap())
            .unwrap();
    let output_root = std::env::var_os("VIZE_JSX_SLOT_CAPTURE")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace.join("target/vize-tests/jsx-slot-parameter-types"))
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&output_root).unwrap();
    let options = json!({
        "jsx": "preserve", "module": "ESNext", "moduleResolution": "Bundler",
        "skipLibCheck": true, "strict": true, "target": "ES2022", "types": ["vue/jsx"]
    });
    for case in cases["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let project = output_root.join(id);
        std::fs::create_dir(&project).unwrap();
        link_dir(dependencies, &project.join("node_modules"));
        let child = std::fs::read(fixtures.join("Child.vue.txt")).unwrap();
        let consumer = std::fs::read(fixtures.join(format!("{id}.tsx.txt"))).unwrap();
        std::fs::write(project.join("Child.vue"), &child).unwrap();
        std::fs::write(project.join("Consumer.tsx"), &consumer).unwrap();
        std::fs::write(
            project.join("vize.config.json"),
            r#"{"typeChecker":{"jsxTypecheck":true}}"#,
        )
        .unwrap();
        std::fs::write(
            project.join("tsconfig.json"),
            json!({
                "compilerOptions": options, "files": ["Child.vue", "Consumer.tsx"]
            })
            .to_string(),
        )
        .unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(&project)
            .args([
                "check",
                "Consumer.tsx",
                "Child.vue",
                "--quiet",
                "--format",
                "json",
                "--corsa-path",
            ])
            .arg(&corsa)
            .output()
            .unwrap();
        std::fs::write(project.join("stdout.raw"), &result.stdout).unwrap();
        std::fs::write(project.join("stderr.raw"), &result.stderr).unwrap();
        std::fs::write(
            project.join("process.json"),
            json!({
                "exitCode": result.status.code(), "success": result.status.success(),
                "sourceCli": env!("CARGO_BIN_EXE_vize"), "corsa": corsa
            })
            .to_string(),
        )
        .unwrap();
        assert_eq!(
            result.status.code(),
            case["exitCode"].as_i64().map(|code| code as i32),
            "{id}"
        );
        assert!(result.stderr.is_empty(), "{id}: {:?}", result.stderr);
        let expected = json!({
            "files": [{"file":"Child.vue","diagnostics":[]}, {"file":"Consumer.tsx","diagnostics":case["diagnostics"]}],
            "programs": [{"root":".","tsconfig":"tsconfig.json","compilerOptions":options,
                "files":["Child.vue","Consumer.tsx","node_modules/vue/jsx.d.ts"]}],
            "errorCount":case["diagnostics"].as_array().unwrap().len(), "warningCount":0, "fileCount":2
        });
        assert_eq!(
            serde_json::from_slice::<Value>(&result.stdout).unwrap(),
            expected,
            "{id}"
        );
        assert_eq!(std::fs::read(project.join("Child.vue")).unwrap(), child);
        assert_eq!(
            std::fs::read(project.join("Consumer.tsx")).unwrap(),
            consumer
        );
    }
}

fn link_dir(source: &Path, target: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}

#[test]
fn unchanged_inferred_slot_oracle_passes_with_explicit_jsx() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    // This original public-contract fixture also imports vue-component-type-helpers.
    let dependencies = workspace.join("tests/node_modules");
    assert!(dependencies.join("vue-component-type-helpers").exists());
    let fixture = workspace.join("tests/_fixtures/differential/typecheck/jsx-slot-parameter-annotations-8419/inferred-slot-public-contract");
    let project = std::env::var_os("VIZE_JSX_SLOT_CAPTURE")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace.join("target/vize-tests/jsx-slot-parameter-types"))
        .join(format!("inferred-{}", std::process::id()));
    std::fs::create_dir_all(&project).unwrap();
    link_dir(&dependencies, &project.join("node_modules"));
    let files: Vec<String> =
        serde_json::from_slice(&std::fs::read(fixture.join("files.json")).unwrap()).unwrap();
    for name in &files {
        std::fs::copy(fixture.join(format!("{name}.txt")), project.join(name)).unwrap();
    }
    for name in ["tsconfig.json", "vize.config.json"] {
        std::fs::copy(fixture.join(name), project.join(name)).unwrap();
    }
    let result = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "--quiet",
            "--format",
            "json",
            "--corsa-path",
        ])
        .arg(&corsa)
        .output()
        .unwrap();
    std::fs::write(project.join("stdout.raw"), &result.stdout).unwrap();
    std::fs::write(project.join("stderr.raw"), &result.stderr).unwrap();
    std::fs::write(project.join("process.json"), json!({"exitCode":result.status.code(),"sourceCli":env!("CARGO_BIN_EXE_vize"),"corsa":corsa}).to_string()).unwrap();
    assert_eq!(result.status.code(), Some(0));
    assert!(result.stderr.is_empty());
    let expected: Value =
        serde_json::from_slice(&std::fs::read(fixture.join("expected-report.json")).unwrap())
            .unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stdout).unwrap(),
        expected
    );
    for name in &files {
        assert_eq!(
            std::fs::read(project.join(name)).unwrap(),
            std::fs::read(fixture.join(format!("{name}.txt"))).unwrap()
        );
    }
}

#[test]
fn unchanged_nested_slot_body_preserves_complete_native_cli_packets() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let fixture =
        workspace.join("tests/_fixtures/differential/typecheck/jsx-nested-native-expressions-8441");
    let source = std::fs::read(fixture.join("AfsStepperDialog.stories.tsx.txt")).unwrap();
    let original =
        std::fs::read_to_string(workspace.join("crates/vize/tests/check_tsx_sfc_attrs_cli.rs"))
            .unwrap();
    assert!(
        original.contains(std::str::from_utf8(&source).unwrap()),
        "unchanged authored slot fixture"
    );
    let root = std::env::var_os("VIZE_JSX_SLOT_CAPTURE")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace.join("target/vize-tests/jsx-slot-parameter-types"))
        .join(format!("nested-{}", std::process::id()));
    for (name, broken) in [("original", false), ("callback-member", true)] {
        let project = root.join(name);
        std::fs::create_dir_all(project.join("src")).unwrap();
        link_dir(
            &workspace.join("tests/node_modules"),
            &project.join("node_modules"),
        );
        let input = if broken {
            String::from_utf8(source.clone())
                .unwrap()
                .replace("{i + 1}", "{i.toUpperCase()}")
                .into_bytes()
        } else {
            source.clone()
        };
        let input_path = project.join("src/AfsStepperDialog.stories.tsx");
        std::fs::write(&input_path, &input).unwrap();
        for file in ["tsconfig.json", "vize.config.json"] {
            std::fs::copy(fixture.join(file), project.join(file)).unwrap();
        }
        let result = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(&project)
            .args([
                "check",
                "--tsconfig",
                "tsconfig.json",
                "src/AfsStepperDialog.stories.tsx",
                "--quiet",
                "--format",
                "json",
                "--corsa-path",
            ])
            .arg(&corsa)
            .output()
            .unwrap();
        std::fs::write(project.join("stdout.raw"), &result.stdout).unwrap();
        std::fs::write(project.join("stderr.raw"), &result.stderr).unwrap();
        std::fs::write(project.join("process.json"), json!({"exitCode":result.status.code(),"sourceCli":env!("CARGO_BIN_EXE_vize"),"corsa":corsa}).to_string()).unwrap();
        let mut expected: Value =
            serde_json::from_slice(&std::fs::read(fixture.join("expected-report.json")).unwrap())
                .unwrap();
        if broken {
            expected["files"][0]["diagnostics"] = json!([
                "error:28:34 [TS2339] Property 'toUpperCase' does not exist on type 'number'."
            ]);
            expected["errorCount"] = json!(1);
        }
        assert_eq!(result.status.code(), Some(i32::from(broken)), "{name}");
        assert!(result.stderr.is_empty(), "{name}");
        assert_eq!(
            serde_json::from_slice::<Value>(&result.stdout).unwrap(),
            expected,
            "{name}"
        );
        assert_eq!(std::fs::read(input_path).unwrap(), input);
    }
}

#[test]
fn nested_raw_and_structural_native_callbacks_keep_whole_packets() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let fixture =
        workspace.join("tests/_fixtures/differential/typecheck/jsx-nested-native-expressions-8441");
    let cases: Vec<String> =
        serde_json::from_slice(&std::fs::read(fixture.join("cases.json")).unwrap()).unwrap();
    for name in cases {
        let source = fixture.join(&name);
        let project = std::env::var_os("VIZE_JSX_SLOT_CAPTURE")
            .map(PathBuf::from)
            .unwrap_or_else(|| workspace.join("target/vize-tests/jsx-slot-parameter-types"))
            .join(format!("{name}-{}", std::process::id()));
        std::fs::create_dir_all(&project).unwrap();
        link_dir(
            &workspace.join("tests/node_modules"),
            &project.join("node_modules"),
        );
        let input = std::fs::read(source.join("App.tsx.txt")).unwrap();
        std::fs::write(project.join("App.tsx"), &input).unwrap();
        for file in ["tsconfig.json", "vize.config.json"] {
            std::fs::copy(source.join(file), project.join(file)).unwrap();
        }
        let result = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(&project)
            .args([
                "check",
                "--tsconfig",
                "tsconfig.json",
                "App.tsx",
                "--quiet",
                "--format",
                "json",
                "--corsa-path",
            ])
            .arg(&corsa)
            .output()
            .unwrap();
        std::fs::write(project.join("stdout.raw"), &result.stdout).unwrap();
        std::fs::write(project.join("stderr.raw"), &result.stderr).unwrap();
        std::fs::write(project.join("process.json"), json!({"exitCode":result.status.code(),"sourceCli":env!("CARGO_BIN_EXE_vize"),"corsa":corsa}).to_string()).unwrap();
        let expected: Value =
            serde_json::from_slice(&std::fs::read(source.join("expected-report.json")).unwrap())
                .unwrap();
        assert_eq!(result.status.code(), Some(0), "{name}");
        assert!(result.stderr.is_empty(), "{name}");
        assert_eq!(
            serde_json::from_slice::<Value>(&result.stdout).unwrap(),
            expected,
            "{name}"
        );
        assert_eq!(std::fs::read(project.join("App.tsx")).unwrap(), input);
    }
}

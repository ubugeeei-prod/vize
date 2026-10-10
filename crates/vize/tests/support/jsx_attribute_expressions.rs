//! Required native full CLI packets for authored JSX attribute AST roots.
use super::{corsa_requirement, link_dir};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn authored_attribute_roots_preserve_complete_native_cli_packets() {
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
    .find(|candidate| candidate.join("jsx.d.ts").exists())
    .expect("required actual Vue installation");
    let dependencies = vue.parent().unwrap();
    let fixtures = workspace
        .join("tests/_fixtures/differential/typecheck/jsx-attribute-native-expressions-8371");
    let output = workspace
        .join("target/vize-tests/jsx-attributes")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&output).unwrap();
    for (id, input, semantic_probe, jsx) in [
        ("valid", "valid", false, true),
        ("member", "invalid", false, true),
        ("semantic-options", "valid", true, true),
        ("semantic-options-explicit-false", "valid", true, false),
    ] {
        let project = output.join(id);
        std::fs::create_dir(&project).unwrap();
        link_dir(dependencies, &project.join("node_modules"));
        let host = std::fs::read(fixtures.join("Host.vue.txt")).unwrap();
        let consumer = std::fs::read(fixtures.join(format!("{input}.tsx.txt"))).unwrap();
        let probe = std::fs::read(fixtures.join("Probe.vue.txt")).unwrap();
        std::fs::write(project.join("Host.vue"), &host).unwrap();
        std::fs::write(project.join("Consumer.tsx"), &consumer).unwrap();
        let mut files = vec!["Consumer.tsx", "Host.vue"];
        if semantic_probe {
            std::fs::write(project.join("Probe.vue"), &probe).unwrap();
            files.push("Probe.vue");
        }
        let mut options = json!({"jsx":"preserve", "module":"ESNext", "moduleResolution":"Bundler",
            "skipLibCheck":true, "strict":true, "target":"ES2022", "types":["vue/jsx"]});
        if semantic_probe {
            options["importsNotUsedAsValues"] = json!("preserve");
        }
        std::fs::write(
            project.join("tsconfig.json"),
            json!({"compilerOptions":options,"files":files}).to_string(),
        )
        .unwrap();
        std::fs::write(
            project.join("vize.config.json"),
            json!({"typeChecker":{"jsxTypecheck":jsx}}).to_string(),
        )
        .unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(&project)
            .arg("check")
            .args(&files)
            .args(["--quiet", "--format", "json", "--corsa-path"])
            .arg(&corsa)
            .output()
            .unwrap();
        std::fs::write(project.join("stdout.raw"), &result.stdout).unwrap();
        std::fs::write(project.join("stderr.raw"), &result.stderr).unwrap();
        std::fs::write(
            project.join("process.json"),
            json!({"exitCode":result.status.code(),
            "sourceCli":env!("CARGO_BIN_EXE_vize"),"corsa":corsa,"jsx":jsx})
            .to_string(),
        )
        .unwrap();
        let member = if input == "invalid" {
            json!([r#"error:5:54 [TS2339] Property 'missing' does not exist on type '"ok"'."#])
        } else {
            json!([])
        };
        let mut expected_files = vec![
            json!({"file":"Consumer.tsx","diagnostics":member}),
            json!({"file":"Host.vue","diagnostics":[]}),
        ];
        if semantic_probe {
            expected_files.push(json!({"file":"Probe.vue","diagnostics":["error:2:7 [TS2322] Type 'number' is not assignable to type 'string'."]}));
            expected_files.push(json!({"file":"tsconfig.json","diagnostics":["error:1:1 [TS5023] Unknown compiler option 'importsNotUsedAsValues'."]}));
        }
        let errors = if semantic_probe {
            2
        } else {
            usize::from(input == "invalid")
        };
        let mut program_files = files.clone();
        program_files.push("node_modules/vue/jsx.d.ts");
        let expected = json!({"files":expected_files,"programs":[{"root":".","tsconfig":"tsconfig.json",
            "compilerOptions":options,"files":program_files}],"errorCount":errors,"warningCount":0,"fileCount":files.len()});
        assert_eq!(result.status.code(), Some(i32::from(errors > 0)), "{id}");
        assert!(result.stderr.is_empty(), "{id}: {:?}", result.stderr);
        assert_eq!(
            serde_json::from_slice::<Value>(&result.stdout).unwrap(),
            expected,
            "{id}"
        );
        assert_eq!(std::fs::read(project.join("Host.vue")).unwrap(), host);
        assert_eq!(
            std::fs::read(project.join("Consumer.tsx")).unwrap(),
            consumer
        );
        if semantic_probe {
            assert_eq!(std::fs::read(project.join("Probe.vue")).unwrap(), probe);
        }
    }
}

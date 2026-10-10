//! Required native full CLI packets for authored JSX attribute AST roots.
use super::{corsa_requirement, link_dir};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
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
    let output = std::env::var_os("VIZE_JSX_SLOT_CAPTURE")
        .map(PathBuf::from)
        .map(|root| root.join("attribute-expressions"))
        .unwrap_or_else(|| workspace.join("target/vize-tests/jsx-attributes"))
        .join(std::process::id().to_string());
    let revision = Command::new("git")
        .current_dir(workspace)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(revision.status.success());
    let source_sha = std::str::from_utf8(&revision.stdout).unwrap().trim();
    if let Ok(expected) = std::env::var("SOURCE_SHA") {
        assert_eq!(source_sha, expected);
    }
    let cli = Path::new(env!("CARGO_BIN_EXE_vize"))
        .canonicalize()
        .unwrap();
    let cli_sha256 = executable_hash(&cli);
    let corsa_sha256 = executable_hash(&corsa);
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
        let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
        command
            .current_dir(&project)
            .arg("check")
            .args(&files)
            .args(["--quiet", "--format", "json", "--corsa-path"])
            .arg(&corsa)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let arguments: Vec<_> = command
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect();
        let child = command.spawn().unwrap();
        let pid = child.id();
        let result = child.wait_with_output().unwrap();
        std::fs::write(project.join("stdout.raw"), &result.stdout).unwrap();
        std::fs::write(project.join("stderr.raw"), &result.stderr).unwrap();
        std::fs::write(
            project.join("process.json"),
            json!({"exitCode":result.status.code(),
            "sourceCli":cli,"sourceCliSha256":cli_sha256,
            "corsa":corsa,"corsaSha256":corsa_sha256,"jsx":jsx,
            "sourceSha":source_sha,"pid":pid,"harnessPid":std::process::id(),
            "arguments":arguments,"cwd":project})
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

fn executable_hash(path: &Path) -> vize_l0::String {
    let mut file = std::fs::File::open(path).unwrap();
    let mut hash = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let length = file.read(&mut buffer).unwrap();
        if length == 0 {
            break;
        }
        hash.update(&buffer[..length]);
    }
    vize_l0::cstr!("{:x}", hash.finalize())
}

//! Required native full CLI packets for authored JSX attribute AST roots.
use super::{corsa_requirement, link_dir};
use crate::support::vite_config;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
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
    std::fs::create_dir_all(&output).unwrap();
    let packets = std::env::var_os("VIZE_JSX_ATTRIBUTE_PACKET_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(std::process::id().to_string()))
        .unwrap_or_else(|| output.join("packets"));
    std::fs::create_dir_all(&packets).unwrap();
    let mut image_hashes = json!({"sourceCliSha256":null,"corsaSha256":null});
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
            project.join("vite.config.mjs"),
            vite_config::source(json!({"typeChecker":{"jsxTypecheck":jsx}}).to_string()),
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
        let inputs_before = owned_inputs(&project);
        let mut process = json!({"exitCode":null,"sourceCli":cli,
            "sourceCliSha256":null,"corsa":corsa,"corsaSha256":null,"jsx":jsx,
            "sourceSha":source_sha,"pid":null,"harnessPid":std::process::id(),
            "nativeChildPid":null,"nativeImageLoaded":"unobserved",
            "imageHashScope":"explicit candidate file bytes; not native loaded-image evidence",
            "stdin":"inherited","arguments":arguments,"cwd":project,
            "githubRunId":std::env::var("GITHUB_RUN_ID").ok(),
            "githubRunAttempt":std::env::var("GITHUB_RUN_ATTEMPT").ok()});
        for (field, candidate) in [
            ("sourceCliSha256", cli.as_path()),
            ("corsaSha256", corsa.as_path()),
        ] {
            if image_hashes[field].is_null() {
                match executable_hash(candidate) {
                    Ok(digest) => image_hashes[field] = json!(digest),
                    Err(error) => {
                        retain_packet(
                            &packets,
                            id,
                            &process,
                            &inputs_before,
                            field,
                            Some(json!(error.to_string())),
                            None,
                        );
                        panic!("{id}: {field}: {error}");
                    }
                }
            }
            process[field] = image_hashes[field].clone();
        }
        let child = command.spawn().unwrap_or_else(|error| {
            retain_packet(
                &packets,
                id,
                &process,
                &inputs_before,
                "spawn",
                Some(json!(error.to_string())),
                None,
            );
            panic!("{id}: spawn: {error}");
        });
        let pid = child.id();
        process["pid"] = json!(pid);
        let result = child.wait_with_output().unwrap_or_else(|error| {
            retain_packet(
                &packets,
                id,
                &process,
                &inputs_before,
                "wait",
                Some(json!(error.to_string())),
                None,
            );
            panic!("{id}: wait: {error}");
        });
        process["exitCode"] = json!(result.status.code());
        retain_packet(
            &packets,
            id,
            &process,
            &inputs_before,
            "completed",
            None,
            Some(&result),
        );
        std::fs::write(project.join("stdout.raw"), &result.stdout).unwrap();
        std::fs::write(project.join("stderr.raw"), &result.stderr).unwrap();
        std::fs::write(project.join("process.json"), process.to_string()).unwrap();
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

fn executable_hash(path: &Path) -> io::Result<vize_l0::String> {
    let mut file = std::fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let length = file.read(&mut buffer)?;
        if length == 0 {
            break;
        }
        hash.update(&buffer[..length]);
    }
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| vize_l0::cstr!("{byte:02x}"))
        .collect())
}

fn data_bytes(bytes: &[u8]) -> Value {
    let digest: vize_l0::String = Sha256::digest(bytes)
        .iter()
        .map(|byte| vize_l0::cstr!("{byte:02x}"))
        .collect();
    json!({"bytes":bytes,"length":bytes.len(),"sha256":digest})
}

// A closed owned-file list: never copy a project tree or traverse node_modules.
fn owned_inputs(project: &Path) -> Vec<Value> {
    [
        "Host.vue",
        "Consumer.tsx",
        "Probe.vue",
        "tsconfig.json",
        "vite.config.mjs",
    ]
    .into_iter()
    .map(|name| {
        match std::fs::symlink_metadata(project.join(name)).and_then(|metadata| {
            if !metadata.file_type().is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "owned input is not a regular file",
                ));
            }
            std::fs::read(project.join(name))
        }) {
            Ok(bytes) => json!({"name":name,"state":"present","data":data_bytes(&bytes)}),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                json!({"name":name,"state":"absent","data":null})
            }
            Err(error) => {
                json!({"name":name,"state":"unavailable","data":null,"error":error.to_string()})
            }
        }
    })
    .collect()
}

// Data only, saved before every outcome assertion. No missing stream becomes empty success.
fn retain_packet(
    packets: &Path,
    id: &str,
    process: &Value,
    inputs_before: &[Value],
    stage: &str,
    error: Option<Value>,
    result: Option<&Output>,
) {
    let project = Path::new(process["cwd"].as_str().unwrap());
    let signal = result.and_then(|output| {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            output.status.signal()
        }
        #[cfg(not(unix))]
        {
            let _ = output;
            None::<i32>
        }
    });
    let packet = json!({"schema":"vize.jsx-attribute-native-cli-packet","version":1,
        "case":id,"stage":stage,"error":error,"process":process,
        "inputsBefore":inputs_before,"inputsAfter":owned_inputs(project),
        "stdout":result.map(|output| data_bytes(&output.stdout)),
        "stderr":result.map(|output| data_bytes(&output.stderr)),
        "status":result.map(|output| json!({"code":output.status.code(),
            "success":output.status.success(),"signal":signal,"debug":format!("{:?}",output.status)}))});
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(packets.join(format!("{id}.json")))
        .unwrap();
    serde_json::to_writer(&mut file, &packet).unwrap();
    file.sync_all().unwrap();
}

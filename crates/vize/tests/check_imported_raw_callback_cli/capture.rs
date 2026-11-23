//! Test-only complete packet custody; provider discovery resolves paths without loading packages.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

pub(super) fn digest(bytes: &[u8]) -> vize_l0::String {
    use std::fmt::Write;
    let mut value = vize_l0::String::default();
    for byte in Sha256::digest(bytes) {
        write!(&mut value, "{byte:02x}").unwrap();
    }
    value
}

pub(super) fn save(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

pub(super) fn retain(directory: &Path, output: &Output, source: &str) {
    fs::create_dir_all(directory).unwrap();
    fs::write(directory.join("stdout.bin"), &output.stdout).unwrap();
    fs::write(directory.join("stderr.bin"), &output.stderr).unwrap();
    fs::write(directory.join("input-source.txt"), source).unwrap();
    let value = json!({"status":output.status.code(),"success":output.status.success(),
        "stdoutBytes":output.stdout.len(),"stderrBytes":output.stderr.len(),"source":source,
        "stdoutSha256":digest(&output.stdout),"stderrSha256":digest(&output.stderr)});
    save(&directory.join("process.json"), &value);
    println!("imported-raw-callback-process {}", value);
}

pub(super) fn providers(workspace: &Path, evidence: &Path, runtime: &Path) -> Value {
    let output = Command::new("node")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/programs/discover-callback-providers.js"
        ))
        .arg(workspace)
        .output()
        .unwrap();
    retain(&evidence.join("provider-discovery"), &output, "");
    assert!(output.status.success(), "provider discovery: {:?}", output);
    let mut value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let bytes = fs::read(runtime).unwrap();
    value["native"] = json!({"path":fs::canonicalize(runtime).unwrap(),"bytes":bytes.len(),"sha256":digest(&bytes),"sourceOwner":"UNJOINED"});
    let manifest_path = runtime
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("package.json");
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    value["native"]["manifest"] = json!({"path":manifest_path,"body":manifest,"sha256":digest(manifest.as_bytes()),"package":serde_json::from_str::<Value>(&manifest).unwrap()});
    let cli = Path::new(env!("CARGO_BIN_EXE_vize"));
    let cli_bytes = fs::read(cli).unwrap();
    value["sourceCLI"] = json!({"path":fs::canonicalize(cli).unwrap(),"bytes":cli_bytes.len(),"sha256":digest(&cli_bytes),"crateVersion":env!("CARGO_PKG_VERSION")});
    save(&evidence.join("providers.json"), &value);
    println!(
        "imported-raw-callback-providers {}",
        json!({"file":evidence.join("providers.json"),"sha256":digest(&fs::read(evidence.join("providers.json")).unwrap())})
    );
    value
}

pub(super) fn forwarder(evidence: &Path) -> PathBuf {
    let directory = evidence.join("observer/.bin");
    fs::create_dir_all(&directory).unwrap();
    let source = include_str!("../programs/capture-raw-checker.js");
    let executable = directory.join("native-forward.mjs");
    fs::write(&executable, source).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    }
    executable
}

pub(super) fn native_calls(directory: &Path) -> Value {
    let mut paths = fs::read_dir(directory)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    paths.sort();
    let rows = paths.into_iter().map(|path| {
        let read=|name|serde_json::from_slice::<Value>(&fs::read(path.join(name)).unwrap()).unwrap();
        let request=read("request.json");
        let mut included=Vec::new();
        if request["kind"]=="main-cli" {
            let config_path=Path::new(request["config"].as_str().unwrap());
            let config=read("config.bytes");
            let roots=config.get("files").unwrap_or(&config["include"]);
            for entry in roots.as_array().unwrap() {
                let physical=fs::canonicalize(config_path.parent().unwrap().join(entry.as_str().unwrap())).unwrap();
                let body=fs::read_to_string(&physical).unwrap();
                included.push(json!({"literal":entry,"physical":physical,"sha256":digest(body.as_bytes()),"body":body}));
            }
        }
        json!({"directory":path,"request":request,"status":read("native-replay.status.json"),
            "stdout":fs::read(path.join("native-replay.stdout.bin")).unwrap(),
            "stderr":fs::read(path.join("native-replay.stderr.bin")).unwrap(),"includedSourceBodies":included})
    }).collect::<Vec<_>>();
    let value = json!(rows);
    save(&directory.with_extension("json"), &value);
    println!(
        "imported-raw-callback-native-custody {}",
        json!({"file":directory.with_extension("json"),"sha256":digest(&fs::read(directory.with_extension("json")).unwrap())})
    );
    value
}

pub(super) fn geometry(evidence: &Path, providers: &Value, inputs: &Value) -> Value {
    let input = evidence.join("geometry-input.json");
    save(&input, inputs);
    let output = Command::new("node")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/programs/observe-callback-membership.js"
        ))
        .arg(providers["typescript"]["entry"].as_str().unwrap())
        .arg(&input)
        .output()
        .unwrap();
    retain(&evidence.join("geometry-process"), &output, "");
    assert!(output.status.success(), "{:?}", output);
    let value = serde_json::from_slice(&output.stdout).unwrap();
    save(&evidence.join("geometry.json"), &value);
    value
}

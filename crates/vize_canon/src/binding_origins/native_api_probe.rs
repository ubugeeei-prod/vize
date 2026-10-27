//! Test-only direct-stdio API carrier. This does not qualify editor socket IPC.
use corsa::{
    api::{ApiMode, ApiSpawnConfig, ProjectSession},
    runtime::block_on,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    process::Command,
};
use vize_carton::{String, append, cstr};

fn hash(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(64);
    for byte in Sha256::digest(bytes).iter() {
        append!(output, "{byte:02x}");
    }
    output
}
fn write(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
fn required(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).expect(name))
}
#[test]
#[ignore = "requires exact public native providers; mandatory in source-law Actions"]
fn standalone_native_api_retains_one_project_symbol_owner() {
    let provider = required("VIZE_ORIGIN_API_PROVIDER").canonicalize().unwrap();
    let backend = required("VIZE_ORIGIN_API_BACKEND").canonicalize().unwrap();
    let output = required("VIZE_ORIGIN_API_CAPTURE");
    let version = std::env::var("VIZE_ORIGIN_API_VERSION").unwrap();
    assert!(!output.exists(), "never replace earlier evidence");
    fs::create_dir(&output).unwrap();
    let probe = Command::new(&backend).arg("--version").output().unwrap();
    write(
        &output.join("backend-version.json"),
        &json!({"status":probe.status.code(),"stdout":probe.stdout,"stderr":probe.stderr}),
    );
    assert!(probe.status.success());
    assert_eq!(probe.stdout, cstr!("Version {version}\n").as_bytes());
    assert!(probe.stderr.is_empty());
    let root = output.join("project");
    fs::create_dir(&root).unwrap();
    symlink(provider.join("node_modules"), root.join("node_modules")).unwrap();
    let config = root.join("tsconfig.json");
    fs::write(
        &config,
        r#"{"compilerOptions":{"strict":true,"noEmit":true,"skipLibCheck":true,"module":"ESNext","moduleResolution":"Bundler","target":"ES2024","types":[]},"files":["probe.ts"]}"#,
    )
    .unwrap();
    let source = "import { useTemplateRef as actual } from 'vue';\n/*😀*/ void actual;\n";
    let file = root.join("probe.ts");
    fs::write(&file, source).unwrap();
    let launcher = output.join("api-exec.py");
    fs::write(&launcher, include_str!("native_api_probe.py")).unwrap();
    fs::set_permissions(&launcher, fs::Permissions::from_mode(0o755)).unwrap();
    let spawn = ApiSpawnConfig::new(&launcher)
        .with_cwd(&root)
        .with_mode(ApiMode::AsyncJsonRpcStdio)
        .with_env(
            "VIZE_ORIGIN_API_BACKEND",
            backend.to_string_lossy().as_ref(),
        )
        .with_env("VIZE_ORIGIN_API_CAPTURE", output.to_string_lossy().as_ref());
    let session = block_on(ProjectSession::spawn(
        spawn,
        config.to_string_lossy().as_ref(),
        None,
    ))
    .expect("direct API stdio carrier must actually open the configured project");
    let lease = session.snapshot_lease();
    let snapshot = lease.handle.clone();
    let project = session.project().clone();
    assert_eq!(Path::new(&project.config_file_name), config);
    let launch: Value =
        serde_json::from_slice(&fs::read(output.join("actual-launch.json")).unwrap()).unwrap();
    let pid = launch.get("pid").and_then(Value::as_u64).unwrap();
    let executable = fs::read_link(cstr!("/proc/{pid}/exe").as_str()).unwrap();
    assert_eq!(executable, backend);
    let stdin = fs::read_link(cstr!("/proc/{pid}/fd/0").as_str()).unwrap();
    let stdout = fs::read_link(cstr!("/proc/{pid}/fd/1").as_str()).unwrap();
    assert!(stdin.to_string_lossy().starts_with("pipe:["));
    assert!(stdout.to_string_lossy().starts_with("pipe:["));
    let offset = source.rfind("actual").unwrap();
    let position = u32::try_from(source.get(..offset).unwrap().encode_utf16().count()).unwrap();
    let selected =
        block_on(session.get_symbol_at_position(file.to_string_lossy().as_ref(), position))
            .unwrap()
            .expect("authored import alias must resolve");
    assert_ne!(selected.flags & (1 << 21), 0);
    let target = block_on(session.client().get_aliased_symbol(
        snapshot.clone(),
        project.id.clone(),
        selected.id.clone(),
    ))
    .unwrap()
    .expect("actual Vue alias must resolve");
    assert_eq!(target.name, "useTemplateRef");
    assert!(!target.declarations.is_empty());
    let again = block_on(session.get_symbol_at_position(file.to_string_lossy().as_ref(), position))
        .unwrap()
        .unwrap();
    assert_eq!(selected, again);
    let normalized = block_on(session.client().raw_json_request(
        "getSymbolAtPosition",
        json!({"snapshot":snapshot,"project":project.id,"file":file,"position":position}),
    ))
    .unwrap();
    write(&output.join("adapter-normalized-symbol.json"), &normalized);
    assert_eq!(
        normalized.get("name").and_then(Value::as_str),
        Some("actual")
    );
    let names = block_on(session.client().raw_json_request(
        "getSourceFileNames",
        json!({"snapshot":snapshot,"project":project.id}),
    ))
    .unwrap();
    write(&output.join("source-file-names.json"), &names);
    let names: Vec<String> = serde_json::from_value(names).unwrap();
    assert!(names.iter().any(|name| Path::new(name.as_str()) == file));
    let manifest = names
        .iter()
        .map(|name| {
            let real = Path::new(name.as_str()).canonicalize().unwrap();
            let bytes = fs::read(&real).unwrap();
            json!({"nativeName":name,"actualPath":real,"bytes":bytes.len(),"sha256":hash(&bytes)})
        })
        .collect::<Vec<_>>();
    write(&output.join("source-input-manifest.json"), &json!(manifest));
    let declaration = target
        .declarations
        .first()
        .unwrap()
        .declaring_path()
        .unwrap();
    let declaration = Path::new(declaration.as_str()).canonicalize().unwrap();
    let core = provider
        .join("node_modules/@vue/runtime-core")
        .canonicalize()
        .unwrap();
    assert!(declaration.starts_with(&core));
    let payload = block_on(session.client().get_source_file(
        snapshot.clone(),
        project.id.clone(),
        declaration.to_string_lossy().as_ref(),
    ))
    .unwrap()
    .expect("the selected declaration owner must exist in this same snapshot");
    fs::write(
        output.join("declaration-owner.encoded.bin"),
        payload.as_bytes(),
    )
    .unwrap();
    let dialect = session.client().dialect().unwrap().as_str();
    assert_eq!(
        dialect,
        if version == "7.0.2" {
            "session-snapshots"
        } else {
            assert_eq!(version, "7.1.0-beta");
            "derived-snapshots"
        }
    );
    assert_eq!(session.snapshot().handle, snapshot);
    assert_eq!(session.project(), &project);
    let receipt = json!({
        "schema":"vize.binding-origin.native-api-carrier.v1",
        "scope":"test-only standalone direct stdio; editor Unix-socket IPC and origin host wiring are unqualified",
        "version":version,"backend":backend,"backendSha256":hash(&fs::read(&backend).unwrap()),
        "provider":provider,"launch":launch,"actualExecutable":executable,
        "actualStdin":stdin,"actualStdout":stdout,"dialect":dialect,
        "snapshot":snapshot,"project":project,"sourceInputManifestEntries":manifest.len(),
        "selected":selected,"target":target,"repeat":again,"adapterNormalized":normalized,
        "fullWireCaptured":false,"actualWireRequestCounts":Value::Null,
        "explicitHostCalls":{"getSymbolAtPosition":3,"getAliasedSymbol":1,"getSourceFileNames":1,"getSourceFile":1},
        "managedSnapshotLeaseBytes":Value::Null,
        "retainedEncodedSourcePayloadBytes":payload.as_bytes().len(),
        "retainedEncodedSourcePayloadSha256":hash(payload.as_bytes()),
        "nativeProgramCreationCount":Value::Null,"nativeBindCount":Value::Null,
        "hostProjectSessionSpawns":1,"hostSnapshotRefreshes":0,"snapshotHandleTransfers":0,
        "hostWire":false,"passed":true
    });
    write(&output.join("carrier-receipt.json"), &receipt);
    println!("NATIVE_ORIGIN_API_RECEIPT {receipt}");
    drop(lease);
    block_on(session.snapshot().release()).unwrap();
    block_on(session.close()).unwrap();
    assert!(!Path::new(cstr!("/proc/{pid}").as_str()).exists());
}

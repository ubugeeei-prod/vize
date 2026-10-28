//! Test-only raw metadata capability; no production origin inference is wired.
use corsa::{
    api::{ApiClient, FileChanges, ProjectSession},
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
mod cases;
mod coverage;
mod fixture;
mod mutation;
mod transport;
use transport::NativeTap;

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

fn pipe_inode(path: &Path) -> u64 {
    path.to_str()
        .and_then(|text| text.strip_prefix("pipe:["))
        .and_then(|text| text.strip_suffix(']'))
        .expect("the actual descriptor must be a pipe inode")
        .parse()
        .expect("the complete pipe inode must be numeric")
}

#[test]
#[ignore = "requires exact public native providers; mandatory in source-law Actions"]
fn standalone_native_api_retains_one_project_symbol_owner() {
    assert_hash_vectors();
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
    fixture::write(&root);
    let config = root.join("tsconfig.json");
    let launcher = output.join("api-exec.mts");
    fs::write(&launcher, include_str!("native_api_probe.mts")).unwrap();
    fs::set_permissions(&launcher, fs::Permissions::from_mode(0o755)).unwrap();
    let mut tap = NativeTap::start(&launcher, &backend, &root, &output);
    let client = block_on(ApiClient::connect_pipe(tap.socket())).unwrap();
    let mut session = block_on(ProjectSession::open(
        client,
        config.to_string_lossy().as_ref(),
        None,
    ))
    .unwrap();
    let launch: Value =
        serde_json::from_slice(&fs::read(output.join("actual-launch.json")).unwrap()).unwrap();
    let pid = launch.get("pid").and_then(Value::as_u64).unwrap();
    assert_eq!(pid, u64::from(tap.pid()));
    assert_eq!(
        launch.get("ppid").and_then(Value::as_u64),
        Some(u64::from(std::process::id()))
    );
    let executable = fs::read_link(cstr!("/proc/{pid}/exe").as_str()).unwrap();
    assert_eq!(executable, backend);
    let stdin = fs::read_link(cstr!("/proc/{pid}/fd/0").as_str()).unwrap();
    let stdout = fs::read_link(cstr!("/proc/{pid}/fd/1").as_str()).unwrap();
    assert_ne!(pipe_inode(&stdin), 0);
    assert_ne!(pipe_inode(&stdout), 0);
    let mut phases = Vec::new();
    for (index, (name, partial)) in [("valid", false), ("partial", true), ("repaired", false)]
        .into_iter()
        .enumerate()
    {
        if index > 0 {
            if partial {
                fixture::partial(&root);
            } else {
                fixture::repair(&root);
            }
            block_on(session.refresh(Some(FileChanges::InvalidateAll {
                invalidate_all: true,
            })))
            .unwrap();
        }
        let lease = session.snapshot_lease();
        let handle = lease.handle.clone();
        assert_eq!(Path::new(&session.project().config_file_name), config);
        let phase = cases::run(
            &session,
            &provider,
            &root,
            &output.join(name),
            &version,
            partial,
        );
        assert_eq!(session.snapshot().handle, handle);
        phases.push(phase);
        drop(lease);
    }
    let semantic_mutations = mutation::run(&mut session, &provider, &root, &output, &version);
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
    block_on(session.snapshot().release()).unwrap();
    let (forwarded, received) = tap.finish().unwrap();
    block_on(session.close()).unwrap();
    assert!(!Path::new(cstr!("/proc/{pid}").as_str()).exists());
    assert_eq!(
        fs::metadata(output.join("client-to-native.wire.bin"))
            .unwrap()
            .len(),
        forwarded
    );
    assert_eq!(
        fs::metadata(output.join("native-to-client.wire.bin"))
            .unwrap()
            .len(),
        received
    );
    let receipt = json!({
        "schema":"vize.binding-origin.native-api-carrier.v3","version":version,
        "scope":"test Unix relay to standalone native stdio; product editor IPC and origin classifier are unqualified",
        "backend":backend,"backendSha256":hash(&fs::read(&backend).unwrap()),
        "provider":provider,"launch":launch,"actualExecutable":executable,
        "actualStdin":stdin,"actualStdout":stdout,"dialect":dialect,
        "phases":phases,"semanticMutationPhases":semantic_mutations,"hashVectors":{"count":4,"oracle":"Node26 crypto and known SHA256 vectors"},
        "forwardedClientBytes":forwarded,"observedNativeStdoutBytes":received,"fullWireCaptured":true,
        "managedSnapshotLeaseBytes":Value::Null,"nativeProgramCreationCount":Value::Null,"nativeBindCount":Value::Null,
        "hostProjectSessionSpawns":1,"hostSnapshotRefreshes":5,"snapshotHandleTransfers":0,
        "currentBaseRelease":"observed wire only; derived current-base ownership may defer release",
        "originClassifierQualified":false,"hostWire":false,"passed":true
    });
    write(&output.join("carrier-receipt.json"), &receipt);
    println!("NATIVE_ORIGIN_API_RECEIPT {receipt}");
}

fn assert_hash_vectors() {
    // Independent Node crypto output; empty and abc also match known SHA-256 vectors.
    let inputs: [&[u8]; 4] = [
        b"",
        b"abc",
        "Vue🦀\r\n漢字".as_bytes(),
        &[0, 255, 128, 1, 0, 127],
    ];
    let expected = [
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        "bd9e2a542de34728433281b5de7eb8d8ac508415f4e8affef0d072a7451ddea1",
        "39ed0ab046e79efb308e0da701c01d10a00c181aabd802522b5b03db74787b21",
    ];
    for (bytes, expected) in inputs.into_iter().zip(expected) {
        assert_eq!(hash(bytes), expected);
    }
}

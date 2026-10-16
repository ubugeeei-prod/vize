//! Actual source CLI replays complete frozen parser and ordinary semantic packets.
#![cfg(unix)]
#[path = "check_canon_native_completion_cli/control.rs"]
mod control;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
use control::assert_control;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};
use vize_l0::{String, cstr};

const CASE: &str = include_str!(
    "../../../tests/_fixtures/differential/typechecker/native-semantic-completion/case.json"
);
const EXPECTED: &[u8] = include_bytes!(
    "../../../tests/_fixtures/differential/typechecker/native-semantic-completion/original.stdout.json"
);
const WRONG: &str = "<script setup lang=\"ts\">\nconst failure: number = \"bad\";\n</script>\n<template>{{ failure }}</template>\n";
const FIXED: &str = "<script setup lang=\"ts\">\nconst failure: number = 1;\n</script>\n<template>{{ failure }}</template>\n";
const EOF: &[u8] = b"error handling method 'exit': EOF\n";

fn digest(bytes: &[u8]) -> String {
    const HEX: &[u8] = b"0123456789abcdef";
    let mut result = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        result.push(HEX[usize::from(byte >> 4)] as char);
        result.push(HEX[usize::from(byte & 15)] as char)
    }
    result
}
fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/typechecker/native-semantic-completion")
}
fn populate(root: &Path, case: &Value) {
    for row in case["inputFiles"].as_array().unwrap() {
        let bytes = fs::read(fixture().join(row["fixture"].as_str().unwrap())).unwrap();
        assert_eq!(digest(&bytes).as_str(), row["sha256"].as_str().unwrap());
        assert_eq!(bytes.len(), row["bytes"].as_u64().unwrap() as usize);
        let path = root.join(row["file"].as_str().unwrap());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, &bytes).unwrap();
    }
}
fn inputs(root: &Path, case: &Value) -> Value {
    json!(
        case["inputFiles"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                let bytes = fs::read(root.join(row["file"].as_str().unwrap())).unwrap();
                json!({"file":row["file"],"bytes":bytes,"sha256":digest(&bytes)})
            })
            .collect::<Vec<_>>()
    )
}
fn run(cli: &Path, native: &Path, root: &Path, dir: &Path, name: &str) -> Output {
    let args = [
        "check",
        "--no-config",
        "--quiet",
        "--format",
        "json",
        "--tsconfig",
        "tsconfig.json",
        "--corsa-path",
        native.to_str().unwrap(),
        "--servers",
        "1",
    ];
    let child = Command::new(cli)
        .current_dir(root)
        .args(args)
        .env("RAYON_NUM_THREADS", "1")
        .env_remove("GOMAXPROCS")
        .env("VIZE_BENCH", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let cli_pid = child.id();
    let output = child.wait_with_output().unwrap();
    fs::write(
        dir.join(cstr!("{name}.stdout.bin").as_str()),
        &output.stdout,
    )
    .unwrap();
    fs::write(
        dir.join(cstr!("{name}.stderr.bin").as_str()),
        &output.stderr,
    )
    .unwrap();
    fs::write(dir.join(cstr!("{name}.receipt.json").as_str()),serde_json::to_vec_pretty(&json!({"cli":cli,"cliPid":cli_pid,"cliReaped":true,"nativeCommand":native,"cwd":root,"argv":args,"exitCode":output.status.code(),"status":cstr!("{:?}",output.status),"stdoutSha256":digest(&output.stdout),"stderrSha256":digest(&output.stderr),"wholePhysicalInputs":physical(root)})).unwrap()).unwrap();
    output
}
fn physical(root: &Path) -> Value {
    fn visit(root: &Path, path: &Path, out: &mut Vec<Value>) {
        let mut entries: Vec<_> = fs::read_dir(path).unwrap().map(|e| e.unwrap()).collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let p = e.path();
            let kind = e.file_type().unwrap();
            if kind.is_dir() {
                visit(root, &p, out)
            } else if kind.is_file() {
                let bytes = fs::read(&p).unwrap();
                out.push(json!({"file":p.strip_prefix(root).unwrap(),"bytes":bytes,"sha256":digest(&bytes)}))
            } else {
                out.push(json!({"file":p.strip_prefix(root).unwrap(),"symlink":fs::read_link(&p).unwrap()}))
            }
        }
    }
    let mut out = Vec::new();
    visit(root, root, &mut out);
    json!(out)
}
fn quote(value: &Path) -> String {
    cstr!("'{}'", value.to_str().unwrap().replace('\'', "'\\''"))
}
fn wrapper(native: &Path, root: &Path, mode: &str) -> PathBuf {
    let path = root.join("native-wrapper.sh");
    let driver = root.join("native-control.py");
    fs::write(
        &driver,
        include_bytes!("../../../tests/_fixtures/corsa-process-receipt/native-control.py"),
    )
    .unwrap();
    assert!(matches!(
        mode,
        "nonzero" | "nonutf8" | "unexpected" | "sigsegv" | "timeout"
    ));
    fs::write(&path, cstr!(
        "#!/bin/sh\nif [ \"$1\" = '--lsp' ]; then\nexec python3 {driver} --native {native} --mode '{mode}' --control-dir {root} -- \"$@\"\nfi\nexec {native} \"$@\"\n",
        driver=quote(&driver), native=quote(native), root=quote(root),
    )).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

#[test]
fn native_completion_cli_preserves_frozen_full_outputs_and_observable_failures() {
    let Some(native) = corsa_requirement::required_or_skip(None::<PathBuf>) else {
        return;
    };
    let native = native.canonicalize().unwrap();
    let cli = Path::new(env!("CARGO_BIN_EXE_vize"))
        .canonicalize()
        .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let dir = std::env::var_os("VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| temp.path().to_path_buf());
    fs::create_dir_all(&dir).unwrap();
    let case: Value = serde_json::from_str(CASE).unwrap();
    let root = dir.join("cli-frozen");
    fs::create_dir_all(&root).unwrap();
    populate(&root, &case);
    let before = inputs(&root, &case);
    let version = Command::new(&native).arg("--version").output().unwrap();
    fs::write(dir.join("source-native-identity.json"),serde_json::to_vec_pretty(&json!({"cli":cli,"cliSha256":digest(&fs::read(&cli).unwrap()),"native":native,"nativeSha256":digest(&fs::read(&native).unwrap()),"versionStdout":version.stdout,"versionStderr":version.stderr,"versionStatus":version.status.code(),"fixture":case,"wholeOriginalInputBytes":before})).unwrap()).unwrap();
    let frozen = run(&cli, &native, &root, &dir, "frozen");
    let mut controls = Vec::new();
    for mode in ["unexpected", "nonutf8", "nonzero", "sigsegv", "timeout"] {
        let control = dir.join(cstr!("cli-{mode}").as_str());
        fs::create_dir_all(&control).unwrap();
        populate(&control, &case);
        let wrapped = wrapper(&native, &control, mode);
        let output = run(&cli, &wrapped, &control, &dir, mode);
        controls.push((mode, control, output));
    }
    let ordinary = dir.join("cli-ordinary");
    fs::create_dir_all(&ordinary).unwrap();
    let config = fs::read(fixture().join("inputs/tsconfig.json.txt")).unwrap();
    fs::write(ordinary.join("tsconfig.json"), config).unwrap();
    fs::write(ordinary.join("Wrong.vue"), WRONG).unwrap();
    let wrong = run(&cli, &native, &ordinary, &dir, "wrong");
    fs::write(ordinary.join("Wrong.vue"), FIXED).unwrap();
    let repaired = run(&cli, &native, &ordinary, &dir, "repaired");
    // Retain every original/native packet before judging any complete output.
    assert_eq!(version.status.code(), Some(0));
    assert!(
        version.stdout.starts_with(b"Version "),
        "native version identity"
    );
    assert_eq!(version.stderr, b"");
    assert_eq!(inputs(&root, &case), before);
    assert_eq!(frozen.status.code(), Some(1));
    assert_eq!(frozen.stdout, EXPECTED);
    assert_eq!(frozen.stderr, b"");
    for (mode, control, output) in controls {
        assert_eq!(inputs(&control, &case), before);
        assert_control(&control, &native, mode);
        if matches!(mode, "nonzero" | "sigsegv" | "timeout") {
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(output.stdout, b"");
            assert_eq!(
                output
                    .stderr
                    .split_inclusive(|b| *b == b'\n')
                    .next()
                    .unwrap(),
                EOF
            );
        } else {
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(output.stdout, EXPECTED);
            let prefix = if mode == "nonutf8" {
                b"wrapper\0\xffcontrol\n".as_slice()
            } else {
                b"unexpected native stderr\n".as_slice()
            };
            let mut expected = prefix.to_vec();
            expected.extend_from_slice(EOF);
            assert_eq!(output.stderr, expected);
        }
    }
    assert_eq!(wrong.status.code(), Some(1));
    assert_eq!(wrong.stderr, b"");
    assert_eq!(repaired.status.code(), Some(0));
    assert_eq!(repaired.stderr, b"");
    let wrong: Value = serde_json::from_slice(&wrong.stdout).unwrap();
    let repaired: Value = serde_json::from_slice(&repaired.stdout).unwrap();
    let options = json!({"lib":["ESNext","DOM"],"module":"ESNext","moduleResolution":"Bundler","skipLibCheck":true,"strict":true,"target":"ESNext"});
    for (actual, diagnostics, errors) in [
        (
            &wrong,
            json!(["error:2:7 [TS2322] Type 'string' is not assignable to type 'number'."]),
            1,
        ),
        (&repaired, json!([]), 0),
    ] {
        assert_eq!(
            *actual,
            json!({"files":[{"file":"Wrong.vue","diagnostics":diagnostics}],"programs":[{"root":".","tsconfig":"tsconfig.json","compilerOptions":options,"files":["Wrong.vue"]}],"errorCount":errors,"warningCount":0,"fileCount":1})
        );
    }
}

#[test]
fn native_completion_cli_refuses_denied_transport_instead_of_reporting_clean() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true},"include":["*.vue"]}"#,
    )
    .unwrap();
    fs::write(root.join("Wrong.vue"), FIXED).unwrap();
    let denied = root.join("denied-native.sh");
    fs::write(
        &denied,
        "#!/bin/sh\nprintf 'native transport denied\\n' >&2\nexit 29\n",
    )
    .unwrap();
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o755)).unwrap();
    let output = run(
        Path::new(env!("CARGO_BIN_EXE_vize")),
        &denied,
        root,
        root,
        "denied",
    );
    assert!(
        !output.status.success(),
        "transport refusal must fail closed"
    );
    assert!(
        output.stdout.is_empty(),
        "no successful diagnostic vector from a denied owner"
    );
    let refusal = b"native transport denied";
    assert!(
        output
            .stderr
            .windows(refusal.len())
            .any(|bytes| bytes == refusal),
        "raw refusal must remain observable: {:?}",
        output.stderr
    );
}

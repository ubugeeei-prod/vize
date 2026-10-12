//! Whole bytes, genuine installed providers, and the ordinary configured checker.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fmt::Write,
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

pub(super) fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub(super) fn corpus() -> PathBuf {
    repository().join("tests/_fixtures/differential/typechecker/empty-vue-star-exports")
}

pub(super) fn relative(value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    assert!(!path.as_os_str().is_empty());
    assert!(
        path.components()
            .all(|part| matches!(part, Component::Normal(_)))
    );
    path
}

pub(super) fn value(path: &str) -> Value {
    serde_json::from_slice(&fs::read(corpus().join(relative(path))).unwrap()).unwrap()
}

pub(super) fn hash(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(64);
    for byte in Sha256::digest(bytes).iter() {
        write!(&mut result, "{byte:02x}").unwrap();
    }
    result
}

pub(super) fn blob(record: &Value) -> Value {
    let body = value(record["relativeBlob"].as_str().unwrap());
    let text = body["wholeUtf8"].as_str().unwrap();
    assert_eq!(body["bytes"], json!(text.len()));
    assert_eq!(body["sha256"], json!(hash(text.as_bytes())));
    assert_eq!(record["bytes"], body["bytes"]);
    assert_eq!(record["sha256"], body["sha256"]);
    body
}

pub(super) fn body(record: &Value) -> String {
    blob(record)["wholeUtf8"].as_str().unwrap().to_owned()
}

fn installed(name: &str) -> Option<PathBuf> {
    [
        "tests/node_modules",
        "node_modules",
        "npm/builder/vite/node_modules",
    ]
    .into_iter()
    .find_map(|directory| repository().join(directory).join(name).canonicalize().ok())
}

pub(super) struct Runtime {
    pub(super) native: PathBuf,
    pub(super) native_custody: Value,
    pub(super) vue: PathBuf,
    pub(super) vite: PathBuf,
}

pub(super) fn runtime() -> Option<Runtime> {
    let required = std::env::var("VIZE_TEST_REQUIRE_TSGO").as_deref() == Ok("1");
    let disabled = std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some();
    let native = (!disabled)
        .then(|| {
            vize_carton::corsa_resolver::resolve_corsa_executable(
                vize_carton::corsa_resolver::CorsaResolveRequest {
                    explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
                    project_root: Some(&repository()),
                },
            )
            .ok()
            .and_then(|path| path.canonicalize().ok())
        })
        .flatten();
    match (native, installed("vue"), installed("vite")) {
        (Some(native), Some(vue), Some(vite)) => Some(Runtime {
            native_custody: super::qualification::native(&native),
            native,
            vue,
            vite,
        }),
        _ => {
            assert!(
                !required,
                "required empty-Vue-star native/provider qualification is unavailable or disabled"
            );
            eprintln!(
                "{}",
                json!({"law":"empty-vue-star-exports", "nativeAcceptance":"unobserved", "reason":"native runtime or genuine installed Vue/Vite provider unavailable"})
            );
            None
        }
    }
}

pub(super) fn link(source: &Path, target: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}

pub(super) fn observed_custody(path: &Path) -> Value {
    match fs::read(path) {
        Ok(bytes) => match String::from_utf8(bytes.clone()) {
            Ok(text) => {
                json!({"path":path,"bytes":bytes.len(),"sha256":hash(&bytes),"wholeUtf8":text})
            }
            Err(error) => json!({"path":path,"bytes":bytes.len(),"sha256":hash(&bytes),
                "wholeBytes":bytes,"observedReadError":error.to_string()}),
        },
        Err(error) => json!({"path":path,"observedReadError":error.to_string()}),
    }
}

pub(super) fn cli(runtime: &Runtime, config: &Path, arguments: &[&str]) -> Value {
    let cwd = config.parent().unwrap();
    let output = Command::new(&runtime.native)
        .args(arguments)
        .arg("--project")
        .arg(config)
        .current_dir(cwd)
        .output()
        .unwrap();
    json!({"nativePath":runtime.native, "nativeSHA256":runtime.native_custody["wholeExecutableSHA256"],
        "arguments":arguments,"project":config,"cwd":cwd,"exitCode":output.status.code(),
        "wholeStdoutBytes":output.stdout,"wholeStdoutUtf8":String::from_utf8(output.stdout.clone()).ok(),
        "wholeStderrBytes":output.stderr,"stdoutSha256":hash(&output.stdout),"stderrSha256":hash(&output.stderr)})
}

pub(super) fn capture(id: &str, packet: &Value) {
    if let Some(directory) = std::env::var_os("VIZE_TEST_FIX_HISTORY_CAPTURE_DIR")
        .or_else(|| std::env::var_os("VIZE_NATIVE_SYNTAX_CAPTURE_DIR"))
    {
        let directory = PathBuf::from(directory).join("empty-vue-star-exports");
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join(format!("{id}.json")),
            serde_json::to_vec_pretty(packet).unwrap(),
        )
        .unwrap();
    }
}

pub(super) fn raw_rows(stdout: &str) -> Value {
    let mut rows = Vec::new();
    for line in stdout.lines() {
        let (position, detail) = line
            .split_once("): error TS")
            .expect("every raw native output line must be retained as a diagnostic");
        let (file, position) = position.rsplit_once('(').unwrap();
        let (line, column) = position.split_once(',').unwrap();
        let (code, message) = detail.split_once(": ").unwrap();
        rows.push(
            json!({"file":Path::new(file).file_name().unwrap().to_str().unwrap(),
            "line":line.parse::<u32>().unwrap(),"column":column.parse::<u32>().unwrap(),
            "severity":1,"code":code.parse::<u32>().unwrap(),"message":message}),
        );
    }
    json!(rows)
}

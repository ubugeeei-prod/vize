//! Linux physical native process controls; no wrapper/counter/zombie credit.

use std::{
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessLife {
    pub pid: u32,
    pub birth: u64,
    pub executable: PathBuf,
}

pub fn original_fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/differential/typechecker/original-diagnosing-process-7698/input.json"
    ))
    .unwrap()
}

pub fn fixture() -> serde_json::Value {
    original_fixture()["admitted"].clone()
}

pub fn write_fixture(root: &Path, fixture: &serde_json::Value) {
    std::fs::write(root.join("source.ts"), fixture["source"].as_str().unwrap()).unwrap();
    std::fs::write(root.join("shared.js"), fixture["shared"].as_str().unwrap()).unwrap();
    std::fs::write(
        root.join("tsconfig.json"),
        serde_json::to_vec(&fixture["config"]).unwrap(),
    )
    .unwrap();
}

pub fn enabled() -> bool {
    let required = std::env::var("VIZE_TEST_REQUIRE_TSGO").as_deref() == Ok("1");
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        assert!(
            !required,
            "required physical native laws cannot be disabled"
        );
        return false;
    }
    true
}

pub fn runtime() -> PathBuf {
    assert!(
        std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_none(),
        "physical native laws cannot be disabled"
    );
    let explicit = std::env::var_os("CORSA_PATH");
    vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            explicit_path: explicit.as_deref().map(Path::new),
            project_root: Some(Path::new(env!("CARGO_MANIFEST_DIR"))),
        },
    )
    .unwrap()
    .canonicalize()
    .unwrap()
}

pub fn native_lsp(root: &Path, executable: &Path) -> ProcessLife {
    native_process(root, executable, b"--lsp")
}

pub fn native_process(root: &Path, executable: &Path, role: &[u8]) -> ProcessLife {
    let root = root.canonicalize().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let lives: Vec<_> = std::fs::read_dir("/proc")
            .unwrap()
            .filter_map(Result::ok)
            .filter_map(|entry| entry.file_name().to_str()?.parse::<u32>().ok())
            .filter_map(|pid| {
                let base = PathBuf::from(vize_l0::cstr!("/proc/{pid}").as_str());
                let stat = std::fs::read_to_string(base.join("stat")).ok()?;
                let (_, tail) = stat.rsplit_once(") ")?;
                let mut fields = tail.split_whitespace();
                if fields.next()? == "Z"
                    || fields.next()?.parse::<u32>().ok()? != std::process::id()
                {
                    return None;
                }
                let birth = fields.nth(17)?.parse().ok()?;
                let physical = std::fs::read_link(base.join("exe")).ok()?;
                let cwd = std::fs::read_link(base.join("cwd")).ok()?;
                let command = std::fs::read(base.join("cmdline")).ok()?;
                if physical != executable
                    || cwd != root
                    || !command.split(|byte| *byte == 0).any(|arg| arg == role)
                {
                    return None;
                }
                Some(ProcessLife {
                    pid,
                    birth,
                    executable: physical,
                })
            })
            .collect();
        if lives.len() == 1 {
            return lives.into_iter().next().unwrap();
        }
        assert!(
            Instant::now() < deadline,
            "exactly one owned physical native process required: {lives:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

pub fn assert_reaped(life: &ProcessLife) {
    let path = PathBuf::from(vize_l0::cstr!("/proc/{}/stat", life.pid).as_str());
    let deadline = Instant::now() + Duration::from_secs(5);
    while path.exists() {
        let stat = match std::fs::read_to_string(&path) {
            Ok(stat) => stat,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            Err(error) => panic!("cannot observe owned native life: {error}"),
        };
        let birth = stat
            .rsplit_once(") ")
            .and_then(|(_, tail)| tail.split_whitespace().nth(19))
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap();
        if birth != life.birth {
            // Kernel PID reuse requires retirement of the recorded old life.
            return;
        }
        assert!(
            Instant::now() < deadline,
            "native process was not reaped: {life:?}"
        );
        // A zombie still has /proc state and receives no retirement credit.
        thread::sleep(Duration::from_millis(10));
    }
}

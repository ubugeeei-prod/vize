//! Test-only decoded RPC and terminal custody; these are not raw wire frames.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "integration process capture uses standard filesystem and buffers"
)]

use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    sync::{Arc, Mutex},
};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub(super) struct Capture {
    root: PathBuf,
    protocol: Arc<Mutex<File>>,
}

impl Capture {
    pub(super) fn new(pid: u32, command: &Command) -> Option<Self> {
        let root = PathBuf::from(std::env::var_os("VIZE_INLAY_HINT_CAPTURE")?)
            .join(format!("process-{pid}"));
        std::fs::create_dir_all(&root).unwrap();
        let executable = Path::new(command.get_program()).canonicalize().unwrap();
        let mut binary = File::open(&executable).unwrap();
        let mut digest = Sha256::new();
        let mut buffer = [0; 64 * 1024];
        loop {
            let length = binary.read(&mut buffer).unwrap();
            if length == 0 {
                break;
            }
            digest.update(&buffer[..length]);
        }
        // Identity probes run only when hosted tests request artifact custody.
        let version = Command::new(&executable).arg("--version").output().unwrap();
        let metadata = json!({
            "pid":pid,"executable":executable,
            "executableSha256":hex(&digest.finalize()),
            "arguments":command.get_args().map(|arg|arg.to_string_lossy()).collect::<Vec<_>>(),
            "cwd":command.get_current_dir(),"sourceSha":std::env::var("SOURCE_SHA").ok(),
            "testName":std::thread::current().name(),
            "requiredCorsa":std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_some(),
            "disabledNative":std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some(),
            "harnessPackageVersion":env!("CARGO_PKG_VERSION"),
            "versionProbe":{"arguments":["--version"],"success":version.status.success(),
                "exitCode":version.status.code(),"stdout":version.stdout,"stderr":version.stderr},
            "protocolRepresentation":"decoded JSON messages; stdout errors retained separately"
        });
        std::fs::write(
            root.join("process.json"),
            serde_json::to_vec_pretty(&metadata).unwrap(),
        )
        .unwrap();
        let protocol = Arc::new(Mutex::new(
            File::create(root.join("protocol.jsonl")).unwrap(),
        ));
        Some(Self { root, protocol })
    }

    pub(super) fn root(&self) -> &Path {
        &self.root
    }

    pub(super) fn record(&self, direction: &str, message: &Value) {
        let mut file = self
            .protocol
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        writeln!(file, "{}", json!({"direction":direction,"message":message})).unwrap();
        file.flush().unwrap();
    }

    pub(super) fn error(&self, direction: &str, error: &std::io::Error) {
        self.record(
            direction,
            &json!({
                "kind":format!("{:?}",error.kind()),"message":error.to_string()
            }),
        );
    }

    pub(super) fn terminal(
        &self,
        status: Option<ExitStatus>,
        stderr: &[u8],
        stdout_joined: Option<bool>,
        stderr_joined: Option<bool>,
    ) {
        std::fs::write(self.root.join("stderr.bin"), stderr).unwrap();
        let terminal = json!({
            "success":status.map(|status|status.success()),
            "exitCode":status.and_then(|status|status.code()),
            "exitStatus":status.map(|status|status.to_string()),
            "stdoutReaderJoined":stdout_joined,"stderrReaderJoined":stderr_joined,
            "stderrBytes":stderr.len(),"stderrSha256":hex(&Sha256::digest(stderr))
        });
        std::fs::write(
            self.root.join("terminal.json"),
            serde_json::to_vec_pretty(&terminal).unwrap(),
        )
        .unwrap();
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .concat()
}

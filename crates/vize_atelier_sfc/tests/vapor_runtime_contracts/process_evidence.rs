//! Preserve the complete SFC child result before enforcing its original exit law.

use std::{
    io,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::{
        OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

static ORDINAL: AtomicUsize = AtomicUsize::new(0);
static SOURCE: OnceLock<Value> = OnceLock::new();

fn signal(output: &Output) -> Option<i32> {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        output.status.signal()
    }
    #[cfg(not(unix))]
    {
        None
    }
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn source() -> Value {
    SOURCE
        .get_or_init(|| {
            let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            let result = Command::new("git")
                .current_dir(workspace)
                .args(["rev-parse", "HEAD", "HEAD^{tree}"])
                .output();
            match result {
                Ok(output) => {
                    let text = String::from_utf8_lossy(&output.stdout);
                    let mut lines = text.lines();
                    json!({
                        "sourceSha": lines.next(), "sourceTree": lines.next(),
                        "gitStatus": output.status.to_string(),
                        "gitStdoutBytes": output.stdout, "gitStderrBytes": output.stderr,
                        "githubSha": std::env::var("GITHUB_SHA").ok()
                    })
                }
                Err(error) => json!({"sourceIdentityError": error.to_string()}),
            }
        })
        .clone()
}

pub(crate) fn capture(
    backend: &str,
    runner: &Path,
    child_pid: u32,
    sources: (&str, &str),
    input: &[u8],
    output: &Output,
    stdin_error: Option<&io::Error>,
) -> io::Result<PathBuf> {
    let profile = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
        "full"
    } else {
        "pr"
    };
    let destination = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/nextest")
        .join(profile)
        .join("vapor-sfc")
        .join(format!(
            "{}-{}",
            std::process::id(),
            ORDINAL.fetch_add(1, Ordering::Relaxed)
        ));
    std::fs::create_dir_all(&destination)?;
    for (filename, bytes) in [
        ("input.bin", input),
        ("stdout.bin", output.stdout.as_slice()),
        ("stderr.bin", output.stderr.as_slice()),
        ("source.vue", sources.0.as_bytes()),
        ("child.vue", sources.1.as_bytes()),
    ] {
        std::fs::write(destination.join(filename), bytes)?;
    }
    let receipt = json!({
        "schema": "vize-sfc-runtime-process/v1", "source": source(),
        "backend": backend, "rustPid": std::process::id(), "childPid": child_pid,
        "runtimeArgv": ["node", runner.to_str()],
        "actualNodeIdentity": null,
        "status": output.status.to_string(), "exitCode": output.status.code(),
        "signal": signal(output), "success": output.status.success(),
        "stdinWriteError": stdin_error.map(ToString::to_string),
        "streams": {
            "input": {"bytes": input.len(), "sha256": digest(input)},
            "stdout": {"bytes": output.stdout.len(), "sha256": digest(&output.stdout)},
            "stderr": {"bytes": output.stderr.len(), "sha256": digest(&output.stderr)}
        },
        "authoredSources": {
            "source.vue": {"bytes": sources.0.len(), "sha256": digest(sources.0.as_bytes())},
            "child.vue": {"bytes": sources.1.len(), "sha256": digest(sources.1.as_bytes())}
        }
    });
    std::fs::write(
        destination.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    Ok(destination)
}

pub(crate) fn failure(
    backend: &str,
    input: &[u8],
    output: &Output,
    capture: &io::Result<PathBuf>,
) -> String {
    format!(
        "{backend}: runtime child failed\nstatus: {}\nexit_code: {:?}\nsignal: {:?}\nevidence: {capture:?}\ninput:\n{}\ninput_bytes: {input:?}\nstdout:\n{}\nstdout_bytes: {:?}\nstderr:\n{}\nstderr_bytes: {:?}",
        output.status,
        output.status.code(),
        signal(output),
        String::from_utf8_lossy(input),
        String::from_utf8_lossy(&output.stdout),
        output.stdout,
        String::from_utf8_lossy(&output.stderr),
        output.stderr,
    )
}

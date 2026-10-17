//! A single Node child pauses before each actual configuration load phase.
//!
//! Only phase path lists are bounded. The final native JSON payload retains
//! the ordinary loader's size semantics. This blocking transport runs solely
//! on the caller's configuration worker, never on an async executor thread.

use std::{
    fmt,
    io::{self, BufReader, ErrorKind, Read, Write},
    path::PathBuf,
    process::{Child, Command, ExitStatus, Output, Stdio},
    thread::{self, JoinHandle},
};

const MAX_PHASE_BYTES: usize = 64 * 1024;

struct ObservedChild {
    child: Child,
    stderr: Option<JoinHandle<io::Result<Vec<u8>>>>,
}

/// Keep the original process evidence even when its final JSON never arrived.
#[derive(Debug)]
struct ObservationFailure {
    cause: io::Error,
    status: io::Result<ExitStatus>,
    stderr: io::Result<Vec<u8>>,
}

impl fmt::Display for ObservationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "node failed to load config: ")?;
        match &self.stderr {
            Ok(stderr) => write!(formatter, "{}", String::from_utf8_lossy(stderr).trim())?,
            Err(error) => write!(formatter, "stderr drain failed: {error}")?,
        }
        match &self.status {
            Ok(status) => write!(
                formatter,
                " ({status}; observation transport: {})",
                self.cause
            ),
            Err(error) => write!(
                formatter,
                " (reap failed: {error}; observation transport: {})",
                self.cause
            ),
        }
    }
}

impl std::error::Error for ObservationFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}

impl ObservedChild {
    fn drain_stderr(&mut self) -> io::Result<Vec<u8>> {
        self.stderr
            .take()
            .ok_or_else(|| io::Error::other("Missing config stderr drain"))?
            .join()
            .map_err(|_| io::Error::other("Config stderr drain panicked"))?
    }

    fn stop(&mut self) -> io::Result<ExitStatus> {
        match self.child.try_wait() {
            Ok(Some(status)) => Ok(status),
            _ => {
                let _ = self.child.kill();
                self.child.wait()
            }
        }
    }

    fn failure(&mut self, cause: io::Error) -> io::Error {
        // Collect both results before returning either error. Stderr remains
        // joined even if reaping fails, and the cause never replaces its bytes.
        let status = self.stop();
        let stderr = self.drain_stderr();
        io::Error::new(
            cause.kind(),
            ObservationFailure {
                cause,
                status,
                stderr,
            },
        )
    }
}

impl Drop for ObservedChild {
    fn drop(&mut self) {
        // Callback unwinding also owns kill/reap and the stderr join.
        let _ = self.stop();
        if let Some(stderr) = self.stderr.take() {
            let _ = stderr.join();
        }
    }
}

pub(super) fn evaluate_observed(
    command: &mut Command,
    before_read: &mut dyn FnMut(&[PathBuf]),
) -> io::Result<Output> {
    let child = command
        .arg("--observe-config-inputs")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut process = ObservedChild {
        child,
        stderr: None,
    };
    let mut stderr = process
        .child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("Missing piped config stderr"))?;
    process.stderr = Some(thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes)?;
        Ok(bytes)
    }));
    let mut stdout = BufReader::new(
        process
            .child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("Missing piped config stdout"))?,
    );
    let mut stdin = process
        .child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("Missing piped config stdin"))?;
    let phases = read_phases(&mut stdout, &mut stdin, before_read);
    drop(stdin);
    let payload = phases.and_then(|()| {
        let mut payload = Vec::new();
        stdout.read_to_end(&mut payload)?;
        Ok(payload)
    });
    drop(stdout);
    let payload = payload.map_err(|cause| process.failure(cause))?;
    let status = process
        .child
        .wait()
        .map_err(|cause| process.failure(cause))?;
    let stderr = process.drain_stderr()?;
    Ok(Output {
        status,
        stdout: payload,
        stderr,
    })
}

fn read_phases(
    stdout: &mut impl Read,
    stdin: &mut impl Write,
    before_read: &mut dyn FnMut(&[PathBuf]),
) -> io::Result<()> {
    loop {
        let mut header = Vec::with_capacity(16);
        loop {
            let mut byte = [0];
            stdout.read_exact(&mut byte)?;
            if byte[0] == b'\n' {
                break;
            }
            if header.len() == 16 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "Invalid config phase header",
                ));
            }
            header.push(byte[0]);
        }
        if header == b"R" {
            break;
        }
        let length = header
            .strip_prefix(b"B")
            .and_then(|value| std::str::from_utf8(value).ok())
            .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|length| *length <= MAX_PHASE_BYTES)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "Invalid config phase length"))?;
        let mut bytes = vec![0; length];
        stdout.read_exact(&mut bytes)?;
        let paths: Vec<PathBuf> = serde_json::from_slice(&bytes)
            .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
        if paths.iter().any(|path| !path.is_absolute()) {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Config phase path must be absolute",
            ));
        }
        before_read(&paths);
        stdin.write_all(&[6])?;
        stdin.flush()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ObservationFailure, evaluate_observed};
    use std::{path::PathBuf, process::Command};

    #[test]
    fn drains_stderr_while_waiting_for_the_phase_ack_and_preserves_large_final_payload() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("policy.mjs");
        let encoded = serde_json::to_string(&[&path]).unwrap();
        let script = format!(
            r#"
const frame = Buffer.from({encoded:?});
process.stderr.write('e'.repeat(100000));
process.stdout.write(`B${{frame.length}}\n`);
process.stdout.write(frame);
process.stdin.once('data', ack => {{
  if (ack.length !== 1 || ack[0] !== 6) process.exit(2);
  process.stdout.end('R\n' + 'x'.repeat(100000));
  process.stdin.destroy();
}});
"#
        );
        let mut command = Command::new("node");
        command
            .arg("-e")
            .arg(script)
            .arg("--")
            .arg("config-fixture");
        let mut observed = Vec::new();
        let output = evaluate_observed(&mut command, &mut |paths| {
            observed.extend_from_slice(paths);
        })
        .unwrap();
        assert!(output.status.success());
        assert_eq!(observed, vec![path]);
        assert_eq!(output.stdout, vec![b'x'; 100000]);
        assert_eq!(output.stderr, vec![b'e'; 100000]);
    }

    #[test]
    fn rejects_truncated_or_oversized_phases_and_reaps_the_child() {
        for frame in [
            "B65537\n",
            "B-1\n",
            "Bwat\n",
            "B10\n[]",
            "B1",
            "",
            "B5\n[\".\"]",
        ] {
            let mut command = Command::new("node");
            command
                .arg("-e")
                .arg(format!("process.stderr.write(\"malformed-specificreason\\n\"); process.stdout.write({frame:?});"))
                .arg("--")
                .arg("config-fixture");
            let mut observed: Vec<PathBuf> = Vec::new();
            let error = evaluate_observed(&mut command, &mut |paths| {
                observed.extend_from_slice(paths);
            })
            .expect_err("accepted invalid config phase");
            let failure = error
                .get_ref()
                .unwrap()
                .downcast_ref::<ObservationFailure>()
                .unwrap();
            assert_eq!(
                failure.stderr.as_ref().unwrap().as_slice(),
                b"malformed-specificreason\n",
                "lost stderr for {frame:?}"
            );
            assert!(failure.status.is_ok());
            assert!(observed.is_empty());
        }
    }

    #[test]
    fn exit_before_final_payload_retains_actual_status_and_stderr() {
        let mut command = Command::new("node");
        command
            .arg("-e")
            .arg("console.error('specificreason 日本語'); process.exit(17);")
            .arg("--")
            .arg("config-fixture");
        let error =
            evaluate_observed(&mut command, &mut |_| panic!("no phase emitted")).unwrap_err();
        let failure = error
            .get_ref()
            .unwrap()
            .downcast_ref::<ObservationFailure>()
            .unwrap();
        assert_eq!(failure.status.as_ref().unwrap().code(), Some(17));
        assert_eq!(
            failure.stderr.as_ref().unwrap().as_slice(),
            "specificreason 日本語\n".as_bytes()
        );
        assert_eq!(failure.cause.kind(), std::io::ErrorKind::UnexpectedEof);
        let status = if cfg!(windows) {
            "exit code: 17"
        } else {
            "exit status: 17"
        };
        assert_eq!(
            crate::cstr!("{error}"),
            format!(
                "node failed to load config: specificreason 日本語 ({status}; observation transport: failed to fill whole buffer)"
            )
        );
    }

    #[test]
    fn startup_exception_before_finally_retains_actual_diagnostic_and_status() {
        // Independent Node execution owns the complete default exception packet,
        // including runtime-specific internal stack frames. Keep the actual throw
        // and argv identical; this oracle never calls the observation transport.
        let mut direct = Command::new("node");
        direct
            .arg("-e")
            .arg("throw new Error('startup-specificreason');")
            .arg("--")
            .arg("config-fixture")
            .arg("--observe-config-inputs")
            .stdin(std::process::Stdio::piped());
        let baseline = direct.output().unwrap();
        assert_eq!(baseline.status.code(), Some(1));
        assert_eq!(baseline.stdout.as_slice(), b"");
        let stderr = std::str::from_utf8(&baseline.stderr).unwrap();
        let (source, exception) = stderr.split_once("\n\n").unwrap();
        assert_eq!(
            source,
            "[eval]:1\nthrow new Error('startup-specificreason');\n^"
        );
        assert_eq!(
            exception.lines().next(),
            Some("Error: startup-specificreason")
        );

        let mut command = Command::new("node");
        command
            .arg("-e")
            .arg("throw new Error('startup-specificreason');")
            .arg("--")
            .arg("config-fixture");
        let error =
            evaluate_observed(&mut command, &mut |_| panic!("no phase emitted")).unwrap_err();
        let failure = error
            .get_ref()
            .unwrap()
            .downcast_ref::<ObservationFailure>()
            .unwrap();
        assert_eq!(command.get_program(), direct.get_program());
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            direct.get_args().collect::<Vec<_>>()
        );
        assert_eq!(command.get_current_dir(), direct.get_current_dir());
        assert_eq!(
            command.get_envs().collect::<Vec<_>>(),
            direct.get_envs().collect::<Vec<_>>()
        );
        assert_eq!(failure.status.as_ref().unwrap(), &baseline.status);
        assert_eq!(failure.stderr.as_ref().unwrap(), &baseline.stderr);
        assert_eq!(failure.cause.kind(), std::io::ErrorKind::UnexpectedEof);
        assert_eq!(
            crate::cstr!("{}", failure.cause),
            "failed to fill whole buffer"
        );
        assert_eq!(
            crate::cstr!("{error}"),
            format!(
                "node failed to load config: {} ({}; observation transport: failed to fill whole buffer)",
                stderr.strip_suffix('\n').unwrap(),
                baseline.status
            )
        );
    }

    #[test]
    fn callback_panic_kills_and_reaps_the_child_and_joins_its_stderr_drain() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("child.pid");
        let encoded = serde_json::to_string(&[&path]).unwrap();
        let mut command = Command::new("node");
        command
            .arg("-e")
            .arg(format!(
                r#"
const fs = require('node:fs');
const paths = {encoded};
fs.writeFileSync(paths[0], String(process.pid));
process.stderr.write('e'.repeat(100000));
const frame = Buffer.from(JSON.stringify(paths));
process.stdout.write(`B${{frame.length}}\n`);
process.stdout.write(frame);
setInterval(() => {{}}, 1000);
"#
            ))
            .arg("--")
            .arg("config-fixture");
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = evaluate_observed(&mut command, &mut |paths| {
                assert_eq!(paths, std::slice::from_ref(&path));
                panic!("configuration observer panic");
            });
        }));
        assert!(panic.is_err());
        let pid = std::fs::read_to_string(path)
            .unwrap()
            .parse::<u32>()
            .unwrap();
        let status = Command::new("node")
            .arg("-e")
            .arg(format!(
                r#"
try {{ process.kill({pid}, 0); process.exit(1); }}
catch (error) {{ process.exit(error.code === 'ESRCH' ? 0 : 2); }}
"#
            ))
            .status()
            .unwrap();
        assert!(
            status.success(),
            "configuration child {pid} survived or was not reaped"
        );
    }
}

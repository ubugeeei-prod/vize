//! The Node child that evaluates a config, read up to its result line.
//!
//! The child writes its result as one line of JSON (`JSON.stringify` never
//! emits a raw newline) and exits. The host stops reading at that line and
//! gives the child a bounded grace to exit before it kills it, so a timer,
//! server or process the config leaves behind never keeps the host waiting.
//! An observing child first pauses before each actual load phase. Only phase
//! path lists are bounded; the result line keeps the ordinary loader's size
//! semantics. This blocking transport runs solely on the caller's
//! configuration worker, never on an async executor thread.

use std::{
    fmt,
    io::{self, BufRead, BufReader, ErrorKind, Read, Write},
    path::PathBuf,
    process::{Child, Command, ExitStatus, Output, Stdio},
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};

const MAX_PHASE_BYTES: usize = 64 * 1024;
/// How long a child may take to exit, or its stderr to end, once its result is in.
pub(super) const EXIT_GRACE: Duration = Duration::from_secs(2);
/// Sees the paths of each load phase before the child reads them.
type PhaseObserver<'a, 'b> = Option<&'a mut (dyn FnMut(&[PathBuf]) + 'b)>;

/// One evaluation child after its result line or its stdout EOF.
pub(super) struct ChildOutput {
    pub(super) status: ExitStatus,
    /// The result line without its newline, or every byte before EOF.
    pub(super) stdout: Vec<u8>,
    /// The child kept running after a complete result line, so the host killed it.
    pub(super) stopped_after_result: bool,
    stderr: StderrDrain,
}

impl ChildOutput {
    /// The complete process evidence. Stderr is collected only when a caller
    /// needs it, for at most the exit grace, so a stderr handle that a
    /// grandchild inherited cannot block the host.
    pub(super) fn into_output(self) -> io::Result<Output> {
        Ok(Output {
            status: self.status,
            stdout: self.stdout,
            stderr: self.stderr.collect()?,
        })
    }
}

/// Stderr read on its own thread so the child never blocks on it.
pub(super) struct StderrDrain {
    chunks: Receiver<io::Result<Vec<u8>>>,
    bytes: Vec<u8>,
    error: Option<io::Error>,
    ended: bool,
}

impl StderrDrain {
    pub(super) fn start(mut stderr: impl Read + Send + 'static) -> Self {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            loop {
                let mut chunk = vec![0; 8192];
                let read = match stderr.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(read) => read,
                    Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                    Err(error) => {
                        let _ = sender.send(Err(error));
                        break;
                    }
                };
                chunk.truncate(read);
                // A host that stopped listening releases the pipe.
                if sender.send(Ok(chunk)).is_err() {
                    break;
                }
            }
        });
        Self {
            chunks: receiver,
            bytes: Vec::new(),
            error: None,
            ended: false,
        }
    }

    /// Keep what arrives until EOF or `deadline`.
    fn read_until(&mut self, deadline: Instant) {
        while !self.ended {
            match self
                .chunks
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            {
                Ok(Ok(chunk)) => self.bytes.extend_from_slice(&chunk),
                Ok(Err(error)) => {
                    self.error.get_or_insert(error);
                }
                Err(RecvTimeoutError::Disconnected) => self.ended = true,
                Err(RecvTimeoutError::Timeout) => return,
            }
        }
    }

    /// Every byte up to EOF, or up to the exit grace.
    pub(super) fn collect(mut self) -> io::Result<Vec<u8>> {
        self.read_until(Instant::now() + EXIT_GRACE);
        match self.error {
            Some(error) => Err(error),
            None => Ok(self.bytes),
        }
    }
}

struct EvaluationChild {
    child: Child,
    stderr: Option<StderrDrain>,
}

/// Keep the original process evidence even when its result never arrived.
#[derive(Debug)]
pub(super) struct ObservationFailure {
    pub(super) cause: io::Error,
    pub(super) status: io::Result<ExitStatus>,
    pub(super) stderr: io::Result<Vec<u8>>,
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

impl EvaluationChild {
    fn take_stderr(&mut self) -> io::Result<StderrDrain> {
        self.stderr
            .take()
            .ok_or_else(|| io::Error::other("Missing config stderr drain"))
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

    /// Wait at most the exit grace, then kill. Reports whether it had to kill.
    fn exit(&mut self) -> io::Result<(ExitStatus, bool)> {
        let deadline = Instant::now() + EXIT_GRACE;
        let mut pause = Duration::from_micros(20);
        loop {
            if let Some(status) = self.child.try_wait()? {
                return Ok((status, false));
            }
            let now = Instant::now();
            if now >= deadline {
                let _ = self.child.kill();
                return Ok((self.child.wait()?, true));
            }
            match self.stderr.as_mut() {
                // Stderr ends as the child exits, unless another process
                // inherited it; check the child at least every millisecond.
                Some(stderr) if !stderr.ended => {
                    stderr.read_until((now + Duration::from_millis(1)).min(deadline));
                }
                // Reaping follows the closed descriptors within moments.
                _ => {
                    thread::sleep(pause.min(deadline - now));
                    pause = pause.saturating_mul(2).min(Duration::from_millis(1));
                }
            }
        }
    }

    fn failure(&mut self, cause: io::Error) -> io::Error {
        // Collect both results before returning either error. Stderr remains
        // drained even if reaping fails, and the cause never replaces its bytes.
        let status = self.stop();
        let stderr = self.take_stderr().and_then(StderrDrain::collect);
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

impl Drop for EvaluationChild {
    fn drop(&mut self) {
        // Callback unwinding also owns kill/reap and the stderr drain.
        let _ = self.stop();
        if let Some(stderr) = self.stderr.take() {
            let _ = stderr.collect();
        }
    }
}

/// Run one evaluation child up to its result line. With `before_read`, the
/// child pauses before each load phase until the callback has seen its paths.
pub(super) fn evaluate(
    command: &mut Command,
    before_read: PhaseObserver<'_, '_>,
) -> io::Result<ChildOutput> {
    match before_read {
        Some(_) => command.arg("--observe-config-inputs").stdin(Stdio::piped()),
        None => command.stdin(Stdio::null()),
    };
    let child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut process = EvaluationChild {
        child,
        stderr: None,
    };
    let stderr = process
        .child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("Missing piped config stderr"))?;
    process.stderr = Some(StderrDrain::start(stderr));
    let mut stdout = BufReader::new(
        process
            .child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("Missing piped config stdout"))?,
    );
    if let Some(before_read) = before_read {
        let phases = match process.child.stdin.take() {
            Some(mut stdin) => read_phases(&mut stdout, &mut stdin, before_read),
            None => Err(io::Error::other("Missing piped config stdin")),
        };
        phases.map_err(|cause| process.failure(cause))?;
    }
    let mut result = Vec::new();
    let read = stdout.read_until(b'\n', &mut result);
    // Bytes after the result line, and stdout's EOF, are never awaited.
    drop(stdout);
    read.map_err(|cause| process.failure(cause))?;
    let delivered = result.last() == Some(&b'\n');
    if delivered {
        result.pop();
    }
    let (status, killed) = process.exit().map_err(|cause| process.failure(cause))?;
    Ok(ChildOutput {
        status,
        stdout: result,
        stopped_after_result: killed && delivered,
        stderr: process.take_stderr()?,
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

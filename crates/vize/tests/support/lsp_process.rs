#![cfg(test)]
// Process pipes require std's growable UTF-8 buffer and the reader threads
// require shared ownership; both are intentional at this test boundary.
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![allow(
    dead_code,
    reason = "shared helpers serve different integration targets"
)]

use std::{
    io::{Read, Write},
    path::Path,
    process::{Child, ChildStdin, Command, ExitStatus, Stdio},
    sync::{Arc, Mutex, mpsc},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use serde_json::Value;
use vize_l0::{String as CompactString, cstr, path::canonicalize_non_verbatim};

#[path = "lsp_process/capture.rs"]
mod capture;
use capture::Capture;

#[path = "lsp_process/evidence.rs"]
mod evidence;
use evidence::Evidence;

#[path = "lsp_process/protocol.rs"]
mod protocol;
use protocol::read_message;

const MESSAGE_TIMEOUT: Duration = Duration::from_secs(20);
const PROCESS_EXIT_TIMEOUT: Duration = Duration::from_secs(5);
pub struct LspProcess {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    messages: mpsc::Receiver<Result<Value, CompactString>>,
    published_diagnostics: Arc<Mutex<Vec<Value>>>,
    stdout_reader: Option<JoinHandle<()>>,
    stderr_reader: Option<JoinHandle<()>>,
    stderr: Arc<Mutex<Vec<u8>>>,
    status: Option<ExitStatus>,
    capture: Option<Capture>,
    evidence: Evidence,
    stdout_joined: Option<bool>,
    stderr_joined: Option<bool>,
}
impl LspProcess {
    pub fn spawn(project_root: &Path) -> Self {
        Self::spawn_with_stderr(project_root, Stdio::piped())
    }

    /// Spawns with a caller-chosen stderr (e.g. a pipe whose reader an editor
    /// already closed); only a piped stderr is captured for failure reports.
    pub fn spawn_with_stderr(project_root: &Path, stderr_target: Stdio) -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
        command.current_dir(project_root).arg("lsp");
        Self::spawn_command(command, stderr_target)
    }

    pub fn spawn_native(project_root: &Path, executable: &Path) -> Self {
        let mut command = Command::new(executable);
        command.current_dir(project_root).args(["--lsp", "--stdio"]);
        Self::spawn_command(command, Stdio::piped())
    }

    fn spawn_command(mut command: Command, stderr_target: Stdio) -> Self {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(stderr_target)
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stderr_pipe = child.stderr.take();

        let (messages_tx, messages) = mpsc::channel();
        let stderr = Arc::new(Mutex::new(Vec::new()));
        let published_diagnostics = Arc::new(Mutex::new(Vec::new()));
        // Own the child before spawning either reader. If thread creation
        // panics, unwinding drops this partial guard and still reaps the LSP.
        let mut process = Self {
            child: Some(child),
            stdin: Some(stdin),
            messages,
            published_diagnostics: Arc::clone(&published_diagnostics),
            stdout_reader: None,
            stderr_reader: None,
            stderr: Arc::clone(&stderr),
            status: None,
            capture: None,
            evidence: Evidence::default(),
            stdout_joined: None,
            stderr_joined: None,
        };
        process.capture = Capture::new(process.child.as_ref().unwrap().id(), &command);

        let capture = process.capture.clone();
        let evidence = process.evidence.clone();
        let stdout_reader = std::thread::spawn(move || {
            let mut reader = std::io::BufReader::new(stdout);
            loop {
                match read_message(&mut reader) {
                    Ok(message) => {
                        evidence.record("response", &message);
                        if let Some(capture) = &capture {
                            capture.record("response", &message);
                        }
                        if message["method"] == "textDocument/publishDiagnostics" {
                            published_diagnostics
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner)
                                .push(message.clone());
                        }
                        let _ = messages_tx.send(Ok(message));
                    }
                    Err(error) => {
                        if let Some(capture) = &capture {
                            capture.error("stdoutError", &error);
                        }
                        let _ = messages_tx.send(Err(cstr!("LSP stdout closed: {error}")));
                        break;
                    }
                }
            }
        });
        process.stdout_reader = Some(stdout_reader);

        if let Some(stderr_pipe) = stderr_pipe {
            let stderr_buffer = Arc::clone(&stderr);
            let capture = process.capture.clone();
            let stderr_reader = std::thread::spawn(move || {
                let mut reader = std::io::BufReader::new(stderr_pipe);
                let mut buffer = Vec::new();
                if let Err(error) = reader.read_to_end(&mut buffer) {
                    if let Some(capture) = &capture {
                        capture.error("stderrError", &error);
                    }
                }
                *stderr_buffer
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = buffer;
            });
            process.stderr_reader = Some(stderr_reader);
        }

        process
    }

    pub fn send(&mut self, message: Value) {
        self.evidence.record("request", &message);
        self.trace("request", &message);
        let body = cstr!("{message}");
        let result = self
            .stdin
            .as_mut()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "LSP stdin closed"))
            .and_then(|stdin| {
                write!(stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
                stdin.flush()
            });
        if let Err(error) = result {
            self.fail(cstr!("failed to send LSP message {message}: {error}"));
        }
    }
    pub fn recv_response(&mut self, id: i64) -> Value {
        self.recv_matching(|message| message["id"].as_i64() == Some(id))
    }

    pub fn recv_matching(&mut self, mut matches: impl FnMut(&Value) -> bool) -> Value {
        let deadline = Instant::now() + MESSAGE_TIMEOUT;
        let mut seen = Vec::new();
        loop {
            let now = Instant::now();
            if now >= deadline {
                self.fail(cstr!("timed out waiting for LSP message; seen: {seen:#?}"));
            }
            let remaining = deadline.saturating_duration_since(now);
            match self.messages.recv_timeout(remaining) {
                Ok(Ok(message)) => {
                    if matches(&message) {
                        return message;
                    }
                    seen.push(message);
                }
                Ok(Err(error)) => self.fail(cstr!(
                    "failed while waiting for LSP message: {error}; seen: {seen:#?}"
                )),
                Err(error) => self.fail(cstr!(
                    "timed out waiting for LSP message: {error}; seen: {seen:#?}"
                )),
            }
        }
    }

    /// Whole publication envelopes survive response matching and reader shutdown.
    pub fn published_diagnostics(&self) -> Vec<Value> {
        self.published_diagnostics
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn trace_dir(&self) -> Option<&Path> {
        self.capture.as_ref().map(Capture::root)
    }

    fn trace(&self, direction: &str, message: &Value) {
        if let Some(capture) = &self.capture {
            capture.record(direction, message);
        }
    }

    /// Wait for the server to terminate without closing its stdin. This models
    /// editor clients, which keep the pipe alive while waiting for the LSP
    /// `exit` notification to end the child process.
    pub fn wait_for_exit(&mut self) -> ExitStatus {
        assert!(
            self.stdin.is_some(),
            "LSP stdin must remain open while waiting for process exit"
        );
        self.poll_exit()
    }

    /// End the stock native transport after its successful shutdown response.
    pub fn wait_for_transport_eof(&mut self) -> ExitStatus {
        assert!(self.stdin.take().is_some(), "LSP stdin already closed");
        self.poll_exit()
    }

    fn poll_exit(&mut self) -> ExitStatus {
        let deadline = Instant::now() + PROCESS_EXIT_TIMEOUT;
        loop {
            let status = self
                .child
                .as_mut()
                .expect("LSP child is unavailable")
                .try_wait()
                .unwrap_or_else(|error| self.fail(cstr!("failed to poll LSP process: {error}")));
            if let Some(status) = status {
                self.status = Some(status);
                self.finish_readers();
                assert_eq!(self.stdout_joined, Some(true), "stdout reader panicked");
                assert_ne!(self.stderr_joined, Some(false), "stderr reader panicked");
                return status;
            }
            if Instant::now() >= deadline {
                let transport = if self.stdin.is_some() {
                    " while stdin remained open"
                } else {
                    " after stdin EOF"
                };
                self.fail(cstr!(
                    "LSP process did not exit within {PROCESS_EXIT_TIMEOUT:?}{transport}"
                ));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn fail(&mut self, message: CompactString) -> ! {
        self.shutdown();
        let stderr = self
            .stderr
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let stderr = String::from_utf8_lossy(&stderr);
        panic!(
            "{message}\nLSP process status: {:?}\nLSP stderr:\n{stderr}",
            self.status
        );
    }

    fn shutdown(&mut self) {
        self.stdin.take();
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            self.status = child.wait().ok().or(self.status);
        }
        self.finish_readers();
    }

    fn finish_readers(&mut self) {
        if let Some(reader) = self.stdout_reader.take() {
            self.stdout_joined = Some(reader.join().is_ok());
        }
        if let Some(reader) = self.stderr_reader.take() {
            self.stderr_joined = Some(reader.join().is_ok());
        }
        if let Some(capture) = &self.capture {
            let stderr = self
                .stderr
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            capture.terminal(self.status, &stderr, self.stdout_joined, self.stderr_joined);
        }
    }
}

impl Drop for LspProcess {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub fn file_uri(path: &Path) -> CompactString {
    let path = canonicalize_non_verbatim(path);
    let path = path.to_string_lossy().replace('\\', "/");
    let prefix = if path.starts_with('/') {
        "file://"
    } else {
        "file:///"
    };
    cstr!("{prefix}{}", percent_encode_path(&path))
}

fn percent_encode_path(path: &str) -> CompactString {
    let mut encoded = CompactString::with_capacity(path.len());
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                encoded.push(byte as char)
            }
            _ => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                encoded.push('%');
                encoded.push(HEX[(byte >> 4) as usize] as char);
                encoded.push(HEX[(byte & 0x0f) as usize] as char);
            }
        }
    }
    encoded
}

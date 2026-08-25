// Process pipes require std's growable UTF-8 buffer and the reader threads
// require shared ownership; both are intentional at this test boundary.
#![allow(clippy::disallowed_types)]

use std::{
    io::{BufRead, Read, Write},
    path::Path,
    process::{Child, ChildStdin, Command, ExitStatus, Stdio},
    sync::{Arc, Mutex, mpsc},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use serde_json::Value;
use vize_s0::{String as CompactString, cstr, path::canonicalize_non_verbatim};

const MESSAGE_TIMEOUT: Duration = Duration::from_secs(20);
const PROCESS_EXIT_TIMEOUT: Duration = Duration::from_secs(5);

pub struct LspProcess {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    messages: mpsc::Receiver<Result<Value, CompactString>>,
    stdout_reader: Option<JoinHandle<()>>,
    stderr_reader: Option<JoinHandle<()>>,
    stderr: Arc<Mutex<Vec<u8>>>,
    status: Option<ExitStatus>,
}

impl LspProcess {
    pub fn spawn(project_root: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project_root)
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stderr_pipe = child.stderr.take().unwrap();

        let (messages_tx, messages) = mpsc::channel();
        let stderr = Arc::new(Mutex::new(Vec::new()));
        // Own the child before spawning either reader. If thread creation
        // panics, unwinding drops this partial guard and still reaps the LSP.
        let mut process = Self {
            child: Some(child),
            stdin: Some(stdin),
            messages,
            stdout_reader: None,
            stderr_reader: None,
            stderr: Arc::clone(&stderr),
            status: None,
        };

        let stdout_reader = std::thread::spawn(move || {
            let mut reader = std::io::BufReader::new(stdout);
            loop {
                match read_message(&mut reader) {
                    Ok(message) => {
                        if messages_tx.send(Ok(message)).is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        let _ = messages_tx.send(Err(cstr!("LSP stdout closed: {error}")));
                        break;
                    }
                }
            }
        });
        process.stdout_reader = Some(stdout_reader);

        let stderr_buffer = Arc::clone(&stderr);
        let stderr_reader = std::thread::spawn(move || {
            let mut reader = std::io::BufReader::new(stderr_pipe);
            let mut buffer = Vec::new();
            let _ = reader.read_to_end(&mut buffer);
            *stderr_buffer
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = buffer;
        });
        process.stderr_reader = Some(stderr_reader);

        process
    }

    pub fn send(&mut self, message: Value) {
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
                Ok(Ok(message)) if matches(&message) => return message,
                Ok(Ok(message)) => seen.push(message),
                Ok(Err(error)) => self.fail(cstr!(
                    "failed while waiting for LSP message: {error}; seen: {seen:#?}"
                )),
                Err(error) => self.fail(cstr!(
                    "timed out waiting for LSP message: {error}; seen: {seen:#?}"
                )),
            }
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
                return status;
            }
            if Instant::now() >= deadline {
                self.fail(cstr!(
                    "LSP process did not exit within {PROCESS_EXIT_TIMEOUT:?} while stdin remained open"
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
            self.status = child.wait().ok();
        }
        if let Some(reader) = self.stdout_reader.take() {
            let _ = reader.join();
        }
        if let Some(reader) = self.stderr_reader.take() {
            let _ = reader.join();
        }
    }
}

impl Drop for LspProcess {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn read_message(reader: &mut impl BufRead) -> std::io::Result<Value> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "LSP stdout reached EOF",
            ));
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        if let Some(value) = line.strip_prefix("Content-Length:") {
            content_length =
                Some(value.trim().parse::<usize>().map_err(|error| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, error)
                })?);
        }
    }

    let Some(content_length) = content_length else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "missing Content-Length header",
        ));
    };
    let mut body = vec![0; content_length];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
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

#[cfg(test)]
mod tests {
    use super::read_message;
    use std::io::{BufReader, Cursor, ErrorKind};

    #[test]
    fn read_message_accepts_an_exact_content_length() {
        let mut reader = BufReader::new(Cursor::new(b"Content-Length: 7\r\n\r\n{\"x\":1}"));

        assert_eq!(
            read_message(&mut reader).unwrap(),
            serde_json::json!({ "x": 1 })
        );
    }

    #[test]
    fn read_message_rejects_a_missing_content_length() {
        let mut reader = BufReader::new(Cursor::new(b"Content-Type: application/json\r\n\r\n{}"));

        let error = read_message(&mut reader).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert!(vize_s0::cstr!("{error}").contains("missing Content-Length"));
    }

    #[test]
    fn read_message_rejects_a_body_shorter_than_content_length() {
        let mut reader = BufReader::new(Cursor::new(b"Content-Length: 8\r\n\r\n{\"x\":1}"));

        assert_eq!(
            read_message(&mut reader).unwrap_err().kind(),
            ErrorKind::UnexpectedEof
        );
    }
}

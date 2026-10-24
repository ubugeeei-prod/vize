//! Measurement mechanics for `tools/commands/davinci/resource-budgets.rs`
//! (Davinci P5-11a, TS-44): a timestamped stdio LSP session and a Linux
//! `/proc` sampler for process-tree RSS and CPU time.
#![allow(dead_code)]

use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};

/// One server message and the instant the reader thread finished parsing it.
pub struct Received {
    pub at: Instant,
    pub message: Value,
}

pub struct Session {
    pub child: Child,
    pub spawned_at: Instant,
    stdin: ChildStdin,
    messages: Receiver<Received>,
    next_id: i64,
}

impl Session {
    pub fn spawn(server: &Path, workspace: &Path) -> Result<Self, String> {
        let spawned_at = Instant::now();
        let mut child = Command::new(server)
            .arg("lsp")
            .current_dir(workspace)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("cannot spawn {}: {error}", server.display()))?;
        let stdin = child.stdin.take().ok_or("server stdin unavailable")?;
        let mut stdout = child.stdout.take().ok_or("server stdout unavailable")?;
        let (sender, messages) = mpsc::channel();
        thread::spawn(move || {
            let mut buffer = Vec::new();
            let mut chunk = [0u8; 65536];
            loop {
                while let Some((message, used)) = take_frame(&buffer) {
                    buffer.drain(..used);
                    if sender.send(Received { at: Instant::now(), message }).is_err() {
                        return;
                    }
                }
                match stdout.read(&mut chunk) {
                    Ok(0) | Err(_) => return,
                    Ok(read) => buffer.extend_from_slice(&chunk[..read]),
                }
            }
        });
        Ok(Self { child, spawned_at, stdin, messages, next_id: 0 })
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn send(&mut self, message: Value) -> Result<Instant, String> {
        let body = serde_json::to_vec(&message).map_err(|error| error.to_string())?;
        let at = Instant::now();
        write!(self.stdin, "Content-Length: {}\r\n\r\n", body.len())
            .and_then(|()| self.stdin.write_all(&body))
            .and_then(|()| self.stdin.flush())
            .map_err(|error| format!("cannot write to the server: {error}"))?;
        Ok(at)
    }

    pub fn request(&mut self, method: &str, params: Value) -> Result<(Instant, Received), String> {
        self.next_id += 1;
        let id = self.next_id;
        let sent = self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }))?;
        let reply = self.wait(&format!("{method} response"), Duration::from_secs(60), |message| {
            message.get("id").and_then(Value::as_i64) == Some(id) && message.get("method").is_none()
        })?;
        Ok((sent, reply))
    }

    pub fn notify(&mut self, method: &str, params: Value) -> Result<Instant, String> {
        self.send(json!({ "jsonrpc": "2.0", "method": method, "params": params }))
    }

    /// Waits for the first message matching `predicate`, answering server requests on the way.
    pub fn wait(
        &mut self,
        label: &str,
        timeout: Duration,
        predicate: impl Fn(&Value) -> bool,
    ) -> Result<Received, String> {
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let received = match self.messages.recv_timeout(remaining) {
                Ok(received) => received,
                Err(RecvTimeoutError::Timeout) => return Err(format!("timed out waiting for {label}")),
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(format!("the server closed stdout while waiting for {label}"));
                }
            };
            if predicate(&received.message) {
                return Ok(received);
            }
            if let (Some(id), Some(method)) = (received.message.get("id"), received.message.get("method")) {
                let result = if method == "workspace/configuration" {
                    let items = received.message.pointer("/params/items").and_then(Value::as_array);
                    Value::Array(vec![Value::Null; items.map_or(0, Vec::len)])
                } else {
                    Value::Null
                };
                self.send(json!({ "jsonrpc": "2.0", "id": id, "result": result }))?;
            }
        }
    }

    /// `shutdown` (no params: JSON-RPC forbids `null` there) + `exit`, then the exit code.
    pub fn close(mut self) -> Result<i32, String> {
        self.next_id += 1;
        let id = self.next_id;
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": "shutdown" }))?;
        self.wait("shutdown response", Duration::from_secs(30), |message| {
            message.get("id").and_then(Value::as_i64) == Some(id) && message.get("method").is_none()
        })?;
        self.send(json!({ "jsonrpc": "2.0", "method": "exit" }))?;
        let status = self.child.wait().map_err(|error| error.to_string())?;
        Ok(status.code().unwrap_or(-1))
    }
}

fn take_frame(buffer: &[u8]) -> Option<(Value, usize)> {
    let header_end = buffer.windows(4).position(|window| window == b"\r\n\r\n")?;
    let header = std::str::from_utf8(&buffer[..header_end]).ok()?;
    let length = header.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("content-length").then(|| value.trim().parse::<usize>().ok())?
    })?;
    let end = header_end + 4 + length;
    if buffer.len() < end {
        return None;
    }
    let message = serde_json::from_slice(&buffer[header_end + 4..end]).unwrap_or(Value::Null);
    Some((message, end))
}

/// Resident set and cumulative CPU time of a process and all its descendants.
#[derive(Clone, Copy, Debug, Default)]
pub struct TreeSample {
    pub rss_bytes: u64,
    pub cpu_seconds: f64,
    pub processes: usize,
}

/// Linux `/proc` sampler. `ticks` and `page` come from `getconf CLK_TCK` / `PAGESIZE`.
pub struct ProcSampler {
    ticks_per_second: f64,
    page_bytes: u64,
}

impl ProcSampler {
    /// `None` off Linux: the sampler reads `/proc/<pid>/stat` and nothing else.
    pub fn new() -> Option<Self> {
        if !Path::new("/proc/self/stat").exists() {
            return None;
        }
        let getconf = |name: &str| -> Option<f64> {
            let output = Command::new("getconf").arg(name).output().ok()?;
            String::from_utf8_lossy(&output.stdout).trim().parse().ok()
        };
        Some(Self { ticks_per_second: getconf("CLK_TCK")?, page_bytes: getconf("PAGESIZE")? as u64 })
    }

    /// Sums RSS (field 24, pages) and utime+stime+cutime+cstime (fields 14–17,
    /// ticks) over `root` and every live descendant found through field 4 (ppid).
    pub fn sample(&self, root: u32) -> Option<TreeSample> {
        let mut parent_of = HashMap::new();
        let mut stats = HashMap::new();
        for entry in fs::read_dir("/proc").ok()?.flatten() {
            let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else { continue };
            let Ok(stat) = fs::read_to_string(entry.path().join("stat")) else { continue };
            let Some(fields) = stat.rsplit_once(") ").map(|(_, rest)| rest.split(' ').collect::<Vec<_>>()) else {
                continue;
            };
            // `fields[0]` is field 3 (state), so field N is `fields[N - 3]`.
            let field = |n: usize| fields.get(n - 3).and_then(|value| value.parse::<u64>().ok()).unwrap_or(0);
            parent_of.insert(pid, field(4) as u32);
            stats.insert(pid, (field(24), field(14) + field(15) + field(16) + field(17)));
        }
        stats.get(&root)?;
        let mut sample = TreeSample::default();
        let mut ticks = 0;
        for (&pid, &(rss_pages, cpu_ticks)) in &stats {
            let mut cursor = pid;
            let in_tree = loop {
                if cursor == root {
                    break true;
                }
                match parent_of.get(&cursor) {
                    Some(&parent) if parent != 0 && parent != cursor => cursor = parent,
                    _ => break false,
                }
            };
            if in_tree {
                sample.rss_bytes += rss_pages * self.page_bytes;
                ticks += cpu_ticks;
                sample.processes += 1;
            }
        }
        sample.cpu_seconds = ticks as f64 / self.ticks_per_second;
        Some(sample)
    }
}

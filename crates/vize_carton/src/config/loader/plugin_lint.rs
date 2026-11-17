//! The trusted JS plugin lint sidecar.
//!
//! One Node process per lint run or editor config evaluates `vite.config.*`
//! with the shared projector and resolves its plan once. It refuses a plan
//! whose digest differs from the one the native host verified, and it answers
//! each request (one line of JSON) with one line of JSON, in order, after one
//! ready line. A plan refusal or rule failure is an answer too, so one failing
//! document leaves the process running. The CLI starts the sidecar as soon as a plugin rule is
//! enabled, so Node startup and config evaluation overlap the built-in lint
//! pass. Later passes (`--fix`) and editor edits reuse the process.

use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Child, ChildStdin, Stdio},
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    time::{Duration, Instant},
};

use super::{js::runtime_file::NodeRuntimeFile, js_observation::StderrDrain};

const SIDECAR: &str = r#"
const configPath = process.argv[1];
const READY = '{"ready":true}';
console.log = (...args) => console.error(...args);
const namespace = await importConfigModule(configPath);
const lint = await openViteConfigPluginLint(namespace.default ?? namespace, configPath);
process.stdout.write(READY + "\n");
const answer = async (request) => {
  try {
    return await lint(request);
  } catch (error) {
    if (error?.name !== "VizePluginPlanError") throw error;
    return { error: String(error.message).split("\n")[0].trimEnd() };
  }
};
let pending = [];
for await (const chunk of process.stdin) {
  let start = 0;
  for (let end = chunk.indexOf(10); end !== -1; end = chunk.indexOf(10, start)) {
    pending.push(chunk.subarray(start, end));
    start = end + 1;
    const request = JSON.parse(Buffer.concat(pending).toString("utf8"));
    pending = [];
    process.stdout.write(JSON.stringify(await answer(request)) + "\n");
  }
  if (start < chunk.length) pending.push(chunk.subarray(start));
}
// The host closed stdin: it is done or gone, so timers the config left must
// not keep this process alive after it.
process.exit(0);
"#;

/// How often a waiting request checks its cancellation.
const CANCEL_CHECK: Duration = Duration::from_millis(5);
/// The sidecar's first line, once its config and plan are resolved.
const READY: &[u8] = br#"{"ready":true}"#;

/// Bounds for one request: config evaluation, then the answer itself.
#[derive(Clone, Copy, Debug)]
pub struct SidecarBudget {
    /// From process start until the plan is resolved.
    pub startup: Duration,
    /// From the later of readiness and the request until its answer.
    pub answer: Duration,
}

/// How a bounded request ended.
#[derive(Debug, PartialEq, Eq)]
pub enum SidecarReply {
    /// The response JSON.
    Answer(Vec<u8>),
    /// The caller stopped waiting; the late answer is skipped.
    Cancelled,
    /// No answer before the deadline; the sidecar was stopped.
    TimedOut,
}

/// A running plugin lint sidecar for one `vite.config.*`.
pub struct PluginLintSidecar {
    child: Child,
    stdin: Option<ChildStdin>,
    /// Complete answer lines, in request order; closed when stdout ends.
    answers: Receiver<Vec<u8>>,
    started: Instant,
    ready: Option<Instant>,
    sent: u64,
    received: u64,
    stderr: Option<StderrDrain>,
    _runtime: NodeRuntimeFile,
}

impl PluginLintSidecar {
    /// Start the sidecar; it evaluates the config while the caller keeps working.
    pub fn start(config_path: &Path) -> Result<Self, String> {
        let runtime = super::js::runtime_source::vite_tools_runtime().map_err(String::from)?;
        let script = [
            runtime.as_str(),
            include_str!("module-runtime.mjs"),
            SIDECAR,
        ]
        .concat();
        let runtime = NodeRuntimeFile::new(&script).map_err(|error| {
            crate::cstr!("failed to prepare the plugin lint sidecar: {error}").into_string()
        })?;
        let mut child = runtime
            .command(config_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| crate::cstr!("failed to start node: {error}").into_string())?;
        let (Some(stdin), Some(stdout), Some(stderr)) =
            (child.stdin.take(), child.stdout.take(), child.stderr.take())
        else {
            let _ = child.kill();
            let _ = child.wait();
            return Err("failed to connect to the plugin lint sidecar".into());
        };
        let (sender, answers) = mpsc::channel();
        std::thread::spawn(move || {
            let mut stdout = BufReader::new(stdout);
            loop {
                let mut line = Vec::new();
                match stdout.read_until(b'\n', &mut line) {
                    Ok(_) if line.pop() == Some(b'\n') => {
                        if sender.send(line).is_err() {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        });
        Ok(Self {
            child,
            stdin: Some(stdin),
            answers,
            started: Instant::now(),
            ready: None,
            sent: 0,
            received: 0,
            // Plugin output goes to stderr; drain it so the process never blocks on it.
            stderr: Some(StderrDrain::start(stderr)),
            _runtime: runtime,
        })
    }

    /// Send one request and wait for its answer.
    pub fn request(&mut self, request: &[u8]) -> Result<Vec<u8>, String> {
        match self.request_within(request, None, &|| false)? {
            SidecarReply::Answer(answer) => Ok(answer),
            SidecarReply::Cancelled | SidecarReply::TimedOut => {
                Err("the plugin lint sidecar stopped answering".into())
            }
        }
    }

    /// Send one request and wait for its answer within `budget`, unless
    /// `cancelled`. Answers to requests the caller stopped waiting for are
    /// skipped.
    pub fn request_within(
        &mut self,
        request: &[u8],
        budget: Option<SidecarBudget>,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<SidecarReply, String> {
        let sent = self
            .stdin
            .as_mut()
            .ok_or_else(|| std::io::Error::other("the sidecar has stopped"))
            .and_then(|stdin| {
                stdin.write_all(request)?;
                stdin.write_all(b"\n")?;
                stdin.flush()
            });
        if sent.is_err() {
            return Err(self.failure());
        }
        self.sent += 1;
        let requested = Instant::now();
        loop {
            let deadline = budget.map(|budget| match self.ready {
                Some(ready) => ready.max(requested) + budget.answer,
                None => self.started + budget.startup,
            });
            let wait = deadline.map_or(CANCEL_CHECK, |deadline| {
                deadline
                    .saturating_duration_since(Instant::now())
                    .min(CANCEL_CHECK)
            });
            match self.answers.recv_timeout(wait) {
                Ok(line) if self.ready.is_none() && line == READY => {
                    self.ready = Some(Instant::now());
                }
                Ok(answer) => {
                    self.received += 1;
                    if self.received == self.sent {
                        return Ok(SidecarReply::Answer(answer));
                    }
                }
                Err(RecvTimeoutError::Disconnected) => return Err(self.failure()),
                Err(RecvTimeoutError::Timeout) => {
                    if cancelled() {
                        return Ok(SidecarReply::Cancelled);
                    }
                    if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                        self.stop();
                        return Ok(SidecarReply::TimedOut);
                    }
                }
            }
        }
    }

    /// Whether the process can still answer.
    pub fn is_running(&self) -> bool {
        self.stdin.is_some()
    }

    /// How long config evaluation took, once the plan is resolved.
    pub fn startup(&self) -> Option<Duration> {
        self.ready.map(|ready| ready - self.started)
    }

    fn stop(&mut self) {
        self.stdin = None;
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    /// The sidecar stopped answering: its plugin refusal or its stderr.
    fn failure(&mut self) -> String {
        self.stop();
        let stderr = self
            .stderr
            .take()
            .and_then(|stderr| stderr.collect().ok())
            .unwrap_or_default();
        let stderr = String::from_utf8_lossy(&stderr);
        // Surface the plugin refusal itself, not the Node stack around it.
        stderr
            .lines()
            .find_map(|line| line.trim().strip_prefix("VizePluginPlanError: "))
            .map_or_else(
                || {
                    let trimmed = stderr.trim_end();
                    if trimmed.is_empty() {
                        "the plugin lint sidecar exited without answering".to_owned()
                    } else {
                        trimmed.to_owned()
                    }
                },
                str::to_owned,
            )
    }
}

impl Drop for PluginLintSidecar {
    fn drop(&mut self) {
        // Every answer has arrived; a plugin's pending timers must not keep
        // the lint run waiting.
        self.stop();
    }
}

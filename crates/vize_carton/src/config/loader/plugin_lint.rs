//! The trusted JS plugin lint sidecar.
//!
//! One Node process per lint run evaluates `vite.config.*` with the shared
//! projector and resolves its plan once. It refuses a plan whose digest
//! differs from the one the native host verified, and it answers each request
//! (one line of JSON) with one line of JSON. The host starts it as soon as a
//! plugin rule is enabled, so Node startup and config evaluation overlap the
//! built-in lint pass. Later passes (`--fix`) reuse the process.

use std::{
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Stdio},
    thread::JoinHandle,
};

use super::js::runtime_file::NodeRuntimeFile;

const SIDECAR: &str = r#"
const configPath = process.argv[1];
console.log = (...args) => console.error(...args);
const namespace = await importConfigModule(configPath);
const lint = await openViteConfigPluginLint(namespace.default ?? namespace, configPath);
let pending = [];
for await (const chunk of process.stdin) {
  let start = 0;
  for (let end = chunk.indexOf(10); end !== -1; end = chunk.indexOf(10, start)) {
    pending.push(chunk.subarray(start, end));
    start = end + 1;
    const request = JSON.parse(Buffer.concat(pending).toString("utf8"));
    pending = [];
    process.stdout.write(JSON.stringify(await lint(request)) + "\n");
  }
  if (start < chunk.length) pending.push(chunk.subarray(start));
}
"#;

/// A running plugin lint sidecar for one `vite.config.*`.
pub struct PluginLintSidecar {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    stderr: Option<JoinHandle<Vec<u8>>>,
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
        let (Some(stdin), Some(stdout), Some(mut stderr)) =
            (child.stdin.take(), child.stdout.take(), child.stderr.take())
        else {
            let _ = child.kill();
            let _ = child.wait();
            return Err("failed to connect to the plugin lint sidecar".into());
        };
        // Plugin output goes to stderr; drain it so the process never blocks on it.
        let stderr = std::thread::spawn(move || {
            let mut output = Vec::new();
            let _ = stderr.read_to_end(&mut output);
            output
        });
        Ok(Self {
            child,
            stdin: Some(stdin),
            stdout: BufReader::new(stdout),
            stderr: Some(stderr),
            _runtime: runtime,
        })
    }

    /// Send one request; returns the response JSON.
    pub fn request(&mut self, request: &[u8]) -> Result<Vec<u8>, String> {
        let sent = self
            .stdin
            .as_mut()
            .ok_or_else(|| std::io::Error::other("the sidecar has stopped"))
            .and_then(|stdin| {
                stdin.write_all(request)?;
                stdin.write_all(b"\n")?;
                stdin.flush()
            });
        let mut response = Vec::new();
        match sent.and_then(|()| self.stdout.read_until(b'\n', &mut response)) {
            Ok(_) if response.pop() == Some(b'\n') => Ok(response),
            _ => Err(self.failure()),
        }
    }

    /// The sidecar stopped answering: its plugin refusal or its stderr.
    fn failure(&mut self) -> String {
        self.stdin = None;
        let _ = self.child.kill();
        let _ = self.child.wait();
        let stderr = self
            .stderr
            .take()
            .and_then(|stderr| stderr.join().ok())
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
        self.stdin = None;
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

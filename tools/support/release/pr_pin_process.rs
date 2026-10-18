//! Bounded compensation processes, with byte custody independent of Node's lifetime.
use super::Retain;
use serde_json::{Value, json};
#[cfg(unix)]
use std::os::unix::{
    fs::{DirBuilderExt, OpenOptionsExt},
    process::ExitStatusExt,
};
use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const MODULE_TYPE: &str = "--input-type=module-typescript";
const STREAM_CAP: usize = 2 * 1024 * 1024;
pub(in super::super) const COMPENSATION_RUNNER: &str = r#"
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawn } from 'node:child_process';
const request = JSON.parse(process.argv[1]);
const cap = 2 * 1024 * 1024;
const owned = request.custody ?? fs.mkdtempSync(path.join(os.tmpdir(), 'vize-compensation-direct-'));
fs.chmodSync(owned, 0o700);
const commands = request.commands ?? [{ program: request.program, args: request.args }];
if (commands.length < 1 || commands.length > 13) throw Error('Invalid finite process count');
const results = await Promise.all(commands.map((command, index) => new Promise((resolve) => {
  const prefix = path.join(owned, String(index));
  const stdout = fs.openSync(prefix + '.stdout', 'wx', 0o600);
  const stderr = fs.openSync(prefix + '.stderr', 'wx', 0o600);
  const packet = { deadline: request.deadline, started: Date.now(), finished: null,
    pid: null, status: null, signal: null, timeout: false, error: null,
    outputComplete: false, completion: 'prepared', exitObserved: null, stdoutBytes: 0, stderrBytes: 0 };
  const record = () => {
    fs.writeFileSync(prefix + '.state-next', JSON.stringify(packet), { mode: 0o600 });
    fs.renameSync(prefix + '.state-next', prefix + '.state');
  };
  const finish = () => {
    fs.closeSync(stdout); fs.closeSync(stderr);
    if (!request.custody) {
      packet.stdout = Array.from(fs.readFileSync(prefix + '.stdout'));
      packet.stderr = Array.from(fs.readFileSync(prefix + '.stderr'));
    }
    record(); resolve(packet);
  };
  record();
  const remaining = request.deadline - Date.now();
  if (remaining <= 0) {
    packet.pid = 0; packet.completion = 'not-started';
    packet.finished = Date.now(); packet.timeout = true;
    packet.error = { code: 'ETIMEDOUT', message: 'Compensation deadline expired before spawn' };
    finish(); return;
  }
  const child = spawn(command.program, command.args, { cwd: request.root, stdio: ['ignore', 'pipe', 'pipe'] });
  packet.pid = child.pid ?? 0;
  packet.completion = child.pid ? 'running' : 'spawn-pending'; record();
  const timer = setTimeout(() => {
    packet.timeout = true; packet.error = { code: 'ETIMEDOUT', message: 'Compensation deadline expired' };
    record(); child.kill('SIGKILL');
  }, Math.max(1, request.deadline - Date.now()));
  const capture = (stream, descriptor) => (chunk) => {
    const key = stream + 'Bytes';
    const available = Math.max(0, cap - packet[key]);
    const bytes = chunk.subarray(0, available);
    for (let offset = 0; offset < bytes.length;) offset += fs.writeSync(descriptor, bytes, offset);
    packet[key] += bytes.length;
    if (chunk.length > available) {
      packet.error = { code: 'ENOBUFS', message: 'Compensation stream exceeds its 2 MiB cap' };
      packet.completion = 'overflow'; record(); child.kill('SIGKILL');
    }
  };
  child.stdout.on('data', capture('stdout', stdout));
  child.stderr.on('data', capture('stderr', stderr));
  child.on('error', (error) => {
    packet.error = { code: error.code ?? 'SPAWN_ERROR', message: error.message }; record();
  });
  child.on('exit', (status, signal) => {
    packet.exitObserved = { status, signal, finished: Date.now() }; record();
  });
  child.on('close', (status, signal) => {
    clearTimeout(timer); packet.status = status; packet.signal = signal;
    packet.finished = Date.now(); packet.completion = 'closed';
    packet.outputComplete = !packet.error || packet.error.code === 'ETIMEDOUT'; finish();
  });
})));
process.stdout.write(JSON.stringify(request.commands ? results : results[0]));
if (!request.custody) fs.rmSync(owned, { recursive: true });
"#;

/// One absolute 15-second compensation bound, never a publication or Q-wait grant.
pub(in super::super) struct Compensation {
    deadline: u64,
}
impl Compensation {
    pub(in super::super) fn new() -> Result<Self, String> {
        Ok(Self {
            deadline: Self::now()?
                .checked_add(15_000)
                .ok_or("Invalid compensation deadline")?,
        })
    }
    fn now() -> Result<u64, String> {
        u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis(),
        )
        .map_err(|e| e.to_string())
    }
    #[cfg(test)]
    pub(in super::super) fn limited(milliseconds: u64) -> Result<Self, String> {
        if milliseconds > 15_000 {
            return Err("Compensation cannot exceed 15 seconds".into());
        }
        Ok(Self {
            deadline: Self::now()? + milliseconds,
        })
    }
    pub(in super::super) fn output(
        &self,
        program: &str,
        args: &[&str],
        root: &Path,
        retain: &mut Retain<'_>,
    ) -> Result<String, String> {
        let packets = self.run(&[json!({"program":program,"args":args})], root, retain)?;
        self.text(&packets[0])
    }
    /// Independent finite REST GETs only. Results retain endpoint order; no retries.
    pub(in super::super) fn batch(
        &self,
        repository: &str,
        endpoints: &[&str],
        root: &Path,
        retain: &mut Retain<'_>,
    ) -> Result<Vec<Value>, String> {
        let commands = read_commands(repository, endpoints)?;
        let packets = self.run(&commands, root, retain)?;
        packets
            .iter()
            .map(|packet| {
                serde_json::from_str(&self.text(packet)?)
                    .map_err(|_| "Compensation REST read did not return complete JSON".into())
            })
            .collect()
    }
    fn text(&self, packet: &Value) -> Result<String, String> {
        if packet["deadline"].as_u64() != Some(self.deadline)
            || packet["finished"]
                .as_u64()
                .is_none_or(|time| time > self.deadline)
            || packet["status"] != 0
            || packet["signal"] != Value::Null
            || packet["error"] != Value::Null
            || packet["outputComplete"] != true
        {
            return Err("Compensation command failed, timed out or has incomplete output; retained whole available process packet".into());
        }
        let bytes: Vec<u8> =
            serde_json::from_value(packet["stdout"].clone()).map_err(|e| e.to_string())?;
        String::from_utf8(bytes)
            .map_err(|_| "Compensation command returned non-UTF-8 output".into())
    }
    fn run(
        &self,
        commands: &[Value],
        root: &Path,
        retain: &mut Retain<'_>,
    ) -> Result<Vec<Value>, String> {
        let custody = private_directory()?;
        let input =
            json!({"deadline":self.deadline,"commands":commands,"root":root,"custody":custody})
                .to_string();
        let result = self.outer(
            &[MODULE_TYPE, "-e", COMPENSATION_RUNNER, &input],
            root,
            &custody,
            retain,
        );
        let mut packets = Vec::new();
        for (index, command) in commands.iter().enumerate() {
            let prefix = custody.join(index.to_string());
            let raw = fs::read(prefix.with_extension("state")).unwrap_or_default();
            let mut packet = serde_json::from_slice::<Value>(&raw).unwrap_or_else(|_| {
                json!({
                    "deadline":self.deadline,"pid":null,"started":null,"finished":null,
                    "status":null,"signal":null,"outputComplete":false,"completion":"unknown"
                })
            });
            packet["command"] = command.clone();
            packet["stateRaw"] = json!(raw);
            packet["custody"] = json!(custody);
            for stream in ["stdout", "stderr"] {
                packet[stream] = match fs::read(prefix.with_extension(stream)) {
                    Ok(bytes) if bytes.len() <= STREAM_CAP => json!(bytes),
                    Ok(_) => return Err("Compensation custody exceeded its stream bound".into()),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Value::Null,
                    Err(error) => {
                        return Err(format!(
                            "Compensation custody read failed: {}",
                            error.kind()
                        ));
                    }
                };
            }
            if packet["completion"] != "closed" && packet["completion"] != "not-started" {
                packet["completion"] = json!("unknown-after-outer-exit");
                packet["outputComplete"] = json!(false);
            }
            retain("compensation-process", &packet)?;
            packets.push(packet);
        }
        // A failing outer packet stays last for existing callers. Raw child custody
        // was captured first, so group KILL cannot erase a child stream prefix.
        let outer = result?;
        retain("compensation-outer", &outer)?;
        fs::remove_dir_all(&custody)
            .map_err(|error| format!("Compensation custody cleanup failed: {}", error.kind()))?;
        if outer["status"] != 0 || outer["signal"] != Value::Null {
            return Err("Outer compensation command failed; retained child completion remains explicitly unknown where unobserved".into());
        }
        Ok(packets)
    }
    fn outer(
        &self,
        args: &[&str],
        root: &Path,
        custody: &Path,
        retain: &mut Retain<'_>,
    ) -> Result<Value, String> {
        let remaining = self.deadline.saturating_sub(Self::now()?);
        if remaining == 0 {
            return Err("Compensation absolute deadline expired; no command started".into());
        }
        let duration = format!("{}.{:03}s", remaining / 1000, remaining % 1000);
        let stdout_path = custody.join("outer.stdout");
        let stderr_path = custody.join("outer.stderr");
        let stdout = private_file(&stdout_path)?;
        let stderr = private_file(&stderr_path)?;
        let started = Self::now()?;
        let result = Command::new("timeout")
            .args(["--signal=KILL", &duration, "node"])
            .args(args)
            .current_dir(root)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr)
            .spawn();
        let result = match result {
            Ok(result) => result,
            Err(error) => {
                retain(
                    "compensation-outer",
                    &json!({"deadline":self.deadline,"started":started,
                    "finished":Self::now()?,"spawnError":error.to_string(),"status":null,"signal":null,
                    "stdout":fs::read(stdout_path).unwrap_or_default(),"stderr":fs::read(stderr_path).unwrap_or_default(),
                    "innerOutput":"unavailable"}),
                )?;
                return Err("Required GNU timeout could not launch".into());
            }
        };
        let pid = result.id();
        let result = result
            .wait_with_output()
            .map_err(|error| format!("Compensation outer wait failed: {}", error.kind()))?
            .status;
        #[cfg(unix)]
        let signal = result.signal();
        #[cfg(not(unix))]
        let signal: Option<i32> = None;
        Ok(
            json!({"deadline":self.deadline,"started":started,"finished":Self::now()?,
            "pid":pid,"status":result.code(),"signal":signal,
            "stdout":fs::read(stdout_path).map_err(|e| e.to_string())?,
            "stderr":fs::read(stderr_path).map_err(|e| e.to_string())?,
            "innerOutput":if result.success(){"unparsed"}else{"missing-or-partial"}}),
        )
    }
    #[cfg(test)]
    pub(in super::super) fn outer_fixture(
        &self,
        script: &str,
        root: &Path,
        retain: &mut Retain<'_>,
    ) -> Result<Vec<u8>, String> {
        let custody = private_directory()?;
        let packet = self.outer(&["-e", script], root, &custody, retain)?;
        retain("compensation-outer", &packet)?;
        fs::remove_dir_all(custody).map_err(|e| e.to_string())?;
        if packet["status"] != 0 || packet["signal"] != Value::Null {
            return Err("Outer compensation fixture failed".into());
        }
        serde_json::from_value(packet["stdout"].clone()).map_err(|e| e.to_string())
    }
}
fn private_directory() -> Result<PathBuf, String> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    for _ in 0..100 {
        let path = std::env::temp_dir().join(format!(
            "vize-compensation-{}-{nonce}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        builder.mode(0o700);
        match builder.create(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "Cannot create private compensation custody: {}",
                    error.kind()
                ));
            }
        }
    }
    Err("Cannot allocate unique compensation custody".into())
}

fn read_commands(repository: &str, endpoints: &[&str]) -> Result<Vec<Value>, String> {
    let valid_component = |part: &str| {
        !part.is_empty()
            && part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-_.".contains(&byte))
            && part != "."
            && part != ".."
    };
    let parts: Vec<_> = repository.split('/').collect();
    if parts.len() != 2
        || !parts.iter().all(|part| valid_component(part))
        || endpoints.is_empty()
        || endpoints.len() > 13
        || endpoints.iter().any(|endpoint| {
            endpoint.is_empty()
                || endpoint.starts_with('/')
                || endpoint.contains(['\r', '\n', '\\', '#'])
                || endpoint
                    .split('?')
                    .next()
                    .is_none_or(|path| !path.split('/').all(valid_component))
        })
    {
        return Err("Invalid finite repository REST read batch".into());
    }
    let commands: Vec<_> = endpoints.iter().map(|endpoint| json!({
            "program":"gh","args":["api",format!("repos/{repository}/{endpoint}"),"--method","GET"]
        })).collect();
    Ok(commands)
}

#[cfg(test)]
#[path = "pr_pin_process_tests.rs"]
mod tests;

fn private_file(path: &Path) -> Result<File, String> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    options.open(path).map_err(|error| {
        format!(
            "Cannot create compensation stream custody: {}",
            error.kind()
        )
    })
}

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    cell::Cell,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
#[path = "typecheck_mutation_recovery.rs"]
mod recovery;
pub use recovery::{MutationLease, atomic_json};
#[cfg(target_os = "linux")]
#[path = "typecheck_process_session.rs"]
mod process_session;
#[cfg(target_os = "linux")]
use process_session::{cleanup_session, session_members};

pub struct ProjectContext {
    pub directory: PathBuf,
    started: Instant,
    budget: Duration,
    counter: Cell<u64>,
    identity_sha256: String,
}
pub struct Captured {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u128,
}

impl ProjectContext {
    pub fn worker(directory: PathBuf, budget_ms: u64) -> Result<Self, String> {
        #[cfg(target_os = "linux")]
        {
            let identity: Value = serde_json::from_slice(
                &fs::read(directory.join("identity.json")).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            if unsafe { libc::getsid(0) } != std::process::id() as i32
                || identity["parentPid"].as_u64() != Some(unsafe { libc::getppid() } as u64)
                || directory.canonicalize().map_err(|e| e.to_string())? != directory
            {
                return Err("project worker requires its physical supervisor-owned session and evidence directory".into());
            }
            Ok(Self {
                directory: directory.clone(),
                started: Instant::now(),
                budget: Duration::from_millis(budget_ms),
                counter: Cell::new(0),
                identity_sha256: format!(
                    "{:x}",
                    Sha256::digest(
                        fs::read(directory.join("identity.json")).map_err(|e| e.to_string())?
                    )
                ),
            })
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (directory, budget_ms);
            Err("whole-project typecheck containment currently requires Linux".into())
        }
    }
    pub fn phase(&self, phase: &str) -> Result<(), String> {
        self.remaining()?;
        let packet = json!({"schema":"vize.typecheckProjectPhase","phase":phase,"identitySha256":self.identity_sha256,"elapsedMs":self.started.elapsed().as_millis()});
        atomic_json(&self.directory.join("phase-current.json"), &packet)?;
        let mut log = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.directory.join("phases.jsonl"))
            .map_err(|e| e.to_string())?;
        if log.metadata().map_err(|e| e.to_string())?.len() > 8 * 1024 * 1024 {
            return Err("phase evidence exceeds its 8 MiB bound".into());
        }
        writeln!(log, "{packet}")
            .and_then(|_| log.sync_all())
            .map_err(|e| e.to_string())?;
        self.remaining().map(|_| ())
    }
    fn remaining(&self) -> Result<Duration, String> {
        self.budget
            .checked_sub(self.started.elapsed())
            .filter(|d| !d.is_zero())
            .ok_or_else(|| "whole-project typecheck deadline exhausted".to_string())
    }
    pub fn mutation(
        &self,
        fixture: &Path,
        source: &Path,
        original: &[u8],
        broken: &[u8],
    ) -> Result<MutationLease, String> {
        self.phase("mutation lease")?;
        MutationLease::begin(&self.directory, fixture, source, original, broken)
    }
    pub fn restore(&self, lease: MutationLease) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        if session_members(std::process::id())?
            .iter()
            .any(|m| m.0 != std::process::id() as i32 && !matches!(m.2, 'Z' | 'X'))
        {
            // Keep the journal active: only the parent may restore after stopping
            // a surviving checker in another process group within this session.
            return Err(
                "mutation restoration withheld while owned checker descendants remain".into(),
            );
        }
        lease.restore()
    }
    pub fn capture(
        &self,
        command: &Path,
        args: &[String],
        cwd: &Path,
        timeout_ms: u64,
        label: &str,
    ) -> Result<Captured, String> {
        self.phase(label)?;
        let number = self.counter.get();
        self.counter.set(number + 1);
        let stdout_path = self.directory.join(format!("command-{number}-stdout.bin"));
        let stderr_path = self.directory.join(format!("command-{number}-stderr.bin"));
        atomic_json(
            &self.directory.join(format!("command-{number}.json")),
            &json!({"command":command,"args":args,"cwd":cwd,"label":label,"elapsedMs":self.started.elapsed().as_millis()}),
        )?;
        let mut child_command = Command::new(command);
        child_command
            .args(args)
            .current_dir(cwd)
            .env("LANG", "C")
            .env("LC_ALL", "C")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            child_command.process_group(0);
        }
        let limit = self.remaining()?.min(Duration::from_millis(timeout_ms));
        let started = Instant::now();
        let mut child = child_command
            .spawn()
            .map_err(|e| format!("{label} failed to run: {e}"))?;
        let failed = Arc::new(AtomicBool::new(false));
        let out = child.stdout.take().ok_or("missing checker stdout")?;
        let err = child.stderr.take().ok_or("missing checker stderr")?;
        let (out_file, err_file) = (stdout_path.clone(), stderr_path.clone());
        let flag = failed.clone();
        let out = thread::spawn(move || retain_stream(out, &out_file, None, flag));
        let flag = failed.clone();
        let err = thread::spawn(move || retain_stream(err, &err_file, None, flag));
        let status = loop {
            if failed.load(Ordering::Acquire) {
                kill_group(child.id());
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "{label} output could not be retained within its 64 MiB stream bound"
                ));
            }
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                break status;
            }
            if started.elapsed() >= limit {
                kill_group(child.id());
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "{label} failed to run: spawn timed out after {timeout_ms}ms"
                ));
            }
            thread::sleep(Duration::from_millis(5));
        };
        kill_group(child.id());
        for stream in [out.join(), err.join()] {
            stream.map_err(|_| "checker stream reader panicked")??;
        }
        let duration_ms = started.elapsed().as_millis();
        let stdout = fs::read(stdout_path).map_err(|e| e.to_string())?;
        let stderr = fs::read(stderr_path).map_err(|e| e.to_string())?;
        atomic_json(
            &self.directory.join(format!("command-{number}-result.json")),
            &json!({"completed":true,"exitCode":status.code(),"success":status.success(),"elapsedMs":duration_ms,"stdoutBytes":stdout.len(),"stderrBytes":stderr.len(),"stdoutSha256":format!("{:x}",Sha256::digest(&stdout)),"stderrSha256":format!("{:x}",Sha256::digest(&stderr))}),
        )?;
        Ok(Captured {
            status,
            stdout: String::from_utf8_lossy(&stdout).into_owned(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
            duration_ms,
        })
    }
}

pub fn state_directory(report: &Path, identity: &Value) -> Result<PathBuf, String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let directory = report
        .join("typecheck-deadlines")
        .join(format!("{}-{nonce}", std::process::id()));
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let directory = directory.canonicalize().map_err(|e| e.to_string())?;
    atomic_json(&directory.join("identity.json"), identity)?;
    Ok(directory)
}

/// Supervise the same producer, preserving its session leader until all writers stop.
pub fn supervise(
    command: &mut Command,
    directory: &Path,
    started: Instant,
    budget: Duration,
) -> Result<bool, String> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (command, directory, started, budget);
        return Err("whole-project typecheck containment currently requires Linux".into());
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        unsafe {
            command.pre_exec(|| {
                if libc::setsid() == -1 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(())
                }
            });
        }
        let mut child = command.spawn().map_err(|e| e.to_string())?;
        let pid = child.id();
        if unsafe { libc::getsid(pid as i32) } != pid as i32 {
            let _ = child.kill();
            let _ = child.wait();
            return Err("worker did not acquire its owned session".into());
        }
        let stdout = child.stdout.take().ok_or("missing worker stdout")?;
        let stderr = child.stderr.take().ok_or("missing worker stderr")?;
        let out_path = directory.join("producer-stdout.bin");
        let err_path = directory.join("producer-stderr.bin");
        let failed = Arc::new(AtomicBool::new(false));
        let flag = failed.clone();
        let out = thread::spawn(move || retain_stream(stdout, &out_path, Some(false), flag));
        let flag = failed.clone();
        let err = thread::spawn(move || retain_stream(stderr, &err_path, Some(true), flag));
        let timed_out = loop {
            if started.elapsed() >= budget {
                break Ok(true);
            }
            if failed.load(Ordering::Acquire) {
                break Err(
                    "project output could not be retained within its 64 MiB stream bound"
                        .to_string(),
                );
            }
            let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
            if unsafe {
                libc::waitid(
                    libc::P_PID,
                    pid,
                    &mut info,
                    libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
                )
            } != 0
            {
                break Err(std::io::Error::last_os_error().to_string());
            }
            if unsafe { info.si_pid() } != 0 {
                break Ok(false);
            }
            if started.elapsed() >= budget {
                break Ok(true);
            }
            thread::sleep(Duration::from_millis(5));
        };
        let cleanup = cleanup_session(pid);
        let status = match &cleanup {
            Ok(_) => child.wait().map_err(|e| e.to_string()),
            Err(_) => {
                let _ = child.kill();
                child
                    .try_wait()
                    .map_err(|e| e.to_string())
                    .and_then(|status| {
                        status.ok_or_else(|| "worker remains unreaped after failed cleanup".into())
                    })
            }
        };
        // Restoration is mandatory even when recording/stream custody fails.
        let restored = if cleanup.is_ok() {
            recovery::recover(directory)
        } else {
            Err("mutation recovery withheld because live-writer cleanup failed".into())
        };
        let cleanup_value = match &cleanup {
            Ok(value) => value.clone(),
            Err(error) => json!({"error":error}),
        };
        let restoration_value = match &restored {
            Ok(value) => json!({"restored":value}),
            Err(error) => json!({"error":error}),
        };
        let retained = atomic_json(
            &directory.join("supervisor.json"),
            &json!({"schema":"vize.typecheckProjectSupervisor","workerPid":pid,"session":pid,"budgetMs":budget.as_millis(),"elapsedMs":started.elapsed().as_millis(),"timedOut":timed_out.as_ref().ok(),"observedExitCode":status.as_ref().ok().and_then(ExitStatus::code),"cleanup":cleanup_value,"mutationRecovery":restoration_value}),
        );
        cleanup?;
        let restored = restored?;
        let streams = [out.join(), err.join()];
        retained?;
        for stream in streams {
            stream.map_err(|_| "worker stream reader panicked")??;
        }
        let timed_out = timed_out?;
        let status = status?;
        if !timed_out && !status.success() {
            return Err(format!(
                "project worker exited {status}; mutation restored: {restored}"
            ));
        }

        Ok(timed_out)
    }
}

fn retain_stream(
    mut stream: impl Read,
    path: &Path,
    echo: Option<bool>,
    failed: Arc<AtomicBool>,
) -> Result<(), String> {
    let result = (|| {
        let mut file = fs::File::create(path).map_err(|e| e.to_string())?;
        let mut bytes = [0u8; 8192];
        let mut retained = 0usize;
        loop {
            let count = stream.read(&mut bytes).map_err(|e| e.to_string())?;
            if count == 0 {
                break;
            }
            let keep = count.min((64 * 1024 * 1024usize).saturating_sub(retained));
            file.write_all(&bytes[..keep]).map_err(|e| e.to_string())?;
            retained += keep;
            if keep != count {
                file.sync_all().map_err(|e| e.to_string())?;
                return Err("stream exceeds 64 MiB; retained prefix is incomplete".into());
            }
            if let Some(stderr) = echo {
                if stderr {
                    std::io::stderr().write_all(&bytes[..count])
                } else {
                    std::io::stdout().write_all(&bytes[..count])
                }
                .map_err(|e| e.to_string())?;
            }
        }
        file.sync_all().map_err(|e| e.to_string())
    })();
    if result.is_err() {
        failed.store(true, Ordering::Release);
    }
    result
}

fn kill_group(pid: u32) {
    #[cfg(unix)]
    unsafe {
        if let Ok(pid) = i32::try_from(pid) {
            if pid > 0 {
                libc::kill(-pid, libc::SIGKILL);
            }
        }
    }
}

use crate::fast::{CompactString, SmallVec};
use crate::process_receipt::{ProcessObservation, ProcessObserver, ProcessTermination};
use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Stdio},
    thread,
    time::{Duration, Instant},
};

/// Immutable process template for launching Corsa.
///
/// The command is cheap to clone and can be reused across multiple clients.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use corsa_core::CorsaCommand;
///
/// let command = CorsaCommand::new("/opt/bin/corsa")
///     .with_cwd("/workspace")
///     .with_env("CORSA_TRACE", "0");
///
/// assert_eq!(command.cwd(), &PathBuf::from("/workspace"));
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CorsaCommand {
    executable: PathBuf,
    cwd: PathBuf,
    env: SmallVec<[(CompactString, CompactString); 4]>,
    process_observer: Option<ProcessObserver>,
    lifetime_observer: Option<(ProcessLifetimeObserver, CompactString)>,
}

impl CorsaCommand {
    /// Creates a new command template rooted at the current working directory.
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            env: SmallVec::new(),
            process_observer: None,
            lifetime_observer: None,
        }
    }

    /// Returns a clone with a different working directory.
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = cwd.into();
        self
    }

    /// Returns a clone with an extra environment variable.
    pub fn with_env(
        mut self,
        key: impl Into<CompactString>,
        value: impl Into<CompactString>,
    ) -> Self {
        let key = key.into();
        let value = value.into();
        if let Some((_, entry)) = self.env.iter_mut().find(|(existing, _)| existing == key) {
            *entry = value;
        } else {
            self.env.push((key, value));
        }
        self
    }

    /// Returns the working directory used for child processes.
    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }

    /// Returns the executable path used for child processes.
    pub fn executable(&self) -> &PathBuf {
        &self.executable
    }

    /// Captures full stderr and terminal custody for explicitly observed spawns.
    /// Existing unobserved spawn methods keep inherited stderr.
    pub fn with_process_observer(mut self, observer: ProcessObserver) -> Self {
        self.process_observer = Some(observer);
        self
    }

    /// Observe every guarded SDK spawn without retaining its process lifetime.
    pub fn with_lifetime_observer(
        mut self,
        observer: ProcessLifetimeObserver,
        context: impl Into<CompactString>,
    ) -> Self {
        self.lifetime_observer = Some((observer, context.into()));
        self
    }

    /// Bind an opt-in observation directly to the actual child and its guard.
    pub fn spawn_observed_async<I, S>(
        &self,
        args: I,
    ) -> std::io::Result<(Child, Option<ProcessObservation>)>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut child = self.spawn_with_stderr(args, self.process_observer.is_some())?;
        let observation = if let Some(observer) = &self.process_observer {
            let Some(stderr) = child.stderr.take() else {
                let _ = terminate_child_process(&mut child);
                return Err(std::io::Error::other("missing stderr"));
            };
            match observer.start(child.id(), stderr) {
                Ok(observation) => Some(observation),
                Err(error) => {
                    let _ = terminate_child_process(&mut child);
                    return Err(error);
                }
            }
        } else {
            None
        };
        Ok((child, observation))
    }

    /// Spawns a child process with piped stdin/stdout for request/response flows.
    pub fn spawn_async<I, S>(&self, args: I) -> std::io::Result<Child>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.spawn(args)
    }

    /// Spawns a child process for blocking protocols such as the msgpack transport.
    pub fn spawn_blocking<I, S>(&self, args: I) -> std::io::Result<std::process::Child>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.spawn(args)
    }

    fn spawn<I, S>(&self, args: I) -> std::io::Result<std::process::Child>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.spawn_with_stderr(args, false)
    }

    fn spawn_with_stderr<I, S>(&self, args: I, capture: bool) -> std::io::Result<Child>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command = std::process::Command::new(&self.executable);
        command
            .args(args)
            .current_dir(&self.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(if capture {
                Stdio::piped()
            } else {
                Stdio::inherit()
            })
            .envs(
                self.env
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.as_str())),
            );
        command.spawn()
    }
}

mod guard;
pub use guard::AsyncChildGuard;
mod lifetime;
pub use lifetime::{
    ProcessCloseObservation, ProcessCloseReceipt, ProcessLifetimeObserver, ProcessLifetimeSnapshot,
    ProcessOwnership, ProcessTerminalObserver,
};
mod owned;
pub use owned::OwnedProcess;

/// Waits for a child process to exit and forcefully terminates it after a timeout.
///
/// This helper is safe to use in cleanup code because it guarantees that a
/// killed child is reaped before returning.
pub fn wait_for_child_exit(child: &mut Child, wait_for: Duration) -> std::io::Result<()> {
    wait_for_child_exit_receipt(child, wait_for).map(|_| ())
}

fn wait_for_child_exit_receipt(
    child: &mut Child,
    wait_for: Duration,
) -> std::io::Result<ProcessTermination> {
    let deadline = Instant::now() + wait_for;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(ProcessTermination {
                status,
                timed_out: false,
                kill_requested: false,
            });
        }
        if Instant::now() >= deadline {
            return terminate_child_process_receipt(child, true);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

/// Terminates a child process if it is still running and always reaps it before returning.
///
/// Reaping matters just as much as killing: without the final `wait`, exited
/// children can remain as zombies on Unix-like systems.
pub fn terminate_child_process(child: &mut Child) -> std::io::Result<()> {
    terminate_child_process_receipt(child, false).map(|_| ())
}

fn terminate_child_process_receipt(
    child: &mut Child,
    timed_out: bool,
) -> std::io::Result<ProcessTermination> {
    if let Some(status) = child.try_wait()? {
        return Ok(ProcessTermination {
            status,
            timed_out,
            kill_requested: false,
        });
    }
    match child.kill() {
        Ok(()) => {}
        Err(error) => {
            if child.try_wait()?.is_none() {
                return Err(error);
            }
            return child.wait().map(|status| ProcessTermination {
                status,
                timed_out,
                kill_requested: true,
            });
        }
    }
    child.wait().map(|status| ProcessTermination {
        status,
        timed_out,
        kill_requested: true,
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::{AsyncChildGuard, terminate_child_process, wait_for_child_exit};
    use std::{process::Command, time::Duration};

    #[test]
    fn startup_abort_after_natural_shutdown_retains_exact_terminal_acknowledgment() {
        use std::{
            future::Future,
            task::{Context, Poll, Waker},
        };
        let child = Command::new("sh")
            .args(["-c", "exit 0"])
            .spawn()
            .expect("spawn natural exit");
        let pid = child.id();
        let guard = AsyncChildGuard::new(child);
        let mut shutdown = Box::pin(guard.shutdown(Duration::from_secs(1)));
        assert!(matches!(
            shutdown
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop())),
            Poll::Ready(Ok(()))
        ));
        drop(shutdown);
        let receipt = guard
            .abort_receipt()
            .expect("cached shutdown")
            .expect("exact terminal receipt");
        assert_eq!(receipt.pid, pid);
        assert!(receipt.status.success() && receipt.reaped);
        assert!(!receipt.kill_requested && !receipt.timed_out);
    }

    #[test]
    fn terminate_child_process_reaps_running_child() {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg("exec sleep 30")
            .spawn()
            .expect("spawn sleeper");
        terminate_child_process(&mut child).expect("terminate child");
        assert!(child.try_wait().expect("try_wait").is_some());
    }

    #[test]
    fn wait_for_child_exit_times_out_and_reaps() {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg("exec sleep 30")
            .spawn()
            .expect("spawn sleeper");
        wait_for_child_exit(&mut child, Duration::from_millis(10)).expect("wait with timeout");
        assert!(child.try_wait().expect("try_wait").is_some());
    }

    #[test]
    fn terminate_child_process_is_ok_after_natural_exit() {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg("exit 0")
            .spawn()
            .expect("spawn exiting child");
        let _ = child.wait().expect("wait");
        terminate_child_process(&mut child).expect("terminate exited child");
    }
}

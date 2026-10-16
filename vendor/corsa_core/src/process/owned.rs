//! Acquire typed child custody before fallible SDK transport initialization.
use super::{AsyncChildGuard, CorsaCommand, ProcessTerminalObserver};
use std::{
    ffi::OsStr,
    process::{ChildStdin, ChildStdout},
    sync::Arc,
};

pub struct OwnedProcess {
    pub stdin: ChildStdin,
    pub stdout: ChildStdout,
    pub process: Arc<AsyncChildGuard>,
}

impl CorsaCommand {
    pub fn spawn_guarded<I, S>(&self, args: I) -> std::io::Result<OwnedProcess>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut child = self.spawn_with_stderr(args, self.process_observer.is_some())?;
        let pid = child.id();
        let terminal = match &self.lifetime_observer {
            Some((observer, context)) => observer.observe(pid, context.clone()),
            None => ProcessTerminalObserver::new(pid, Default::default()),
        };
        terminal.observe_stderr(self.process_observer.clone());
        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let process = Arc::new(AsyncChildGuard::new_guarded(child, None, terminal.clone()));
        let (stdin, stdout) = match (stdin, stdout) {
            (Some(stdin), Some(stdout)) => (stdin, stdout),
            _ => {
                terminal.record_error("guarded SDK spawn is missing stdin/stdout");
                return Err(std::io::Error::other(
                    "guarded SDK spawn is missing stdin/stdout",
                ));
            }
        };
        if let Some(observer) = &self.process_observer {
            let Some(stderr) = stderr else {
                terminal.record_error("guarded SDK spawn is missing stderr");
                return Err(std::io::Error::other("guarded SDK spawn is missing stderr"));
            };
            match observer.start(pid, stderr) {
                Ok(observation) => process.set_observation(observation),
                Err(error) => {
                    terminal
                        .record_error(format!("cannot observe owned PID {pid} stderr: {error}"));
                    return Err(error);
                }
            }
        }
        Ok(OwnedProcess {
            stdin,
            stdout,
            process,
        })
    }
}

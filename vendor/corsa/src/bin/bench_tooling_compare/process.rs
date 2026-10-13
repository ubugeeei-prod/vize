use std::{
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

use corsa::{CorsaError, Result, fast::CompactString};
use corsa_core::terminate_child_process;

pub fn run_command(
    command: &mut Command,
    timeout: Duration,
    expected_exit_codes: &[i32],
    label: &str,
) -> Result<()> {
    let started = Instant::now();
    let mut child = ManagedChild::spawn(command)?;
    loop {
        if let Some(status) = child.try_wait()? {
            child.disarm();
            return validate_exit_status(status, expected_exit_codes, label);
        }
        if started.elapsed() >= timeout {
            child.terminate()?;
            return Err(CorsaError::Protocol(CompactString::from(format!(
                "{label} timed out after {} ms",
                timeout.as_millis()
            ))));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

struct ManagedChild {
    child: Option<Child>,
}

impl ManagedChild {
    fn spawn(command: &mut Command) -> Result<Self> {
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        Ok(Self {
            child: Some(command.spawn()?),
        })
    }

    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        self.child
            .as_mut()
            .ok_or_else(|| std::io::Error::other("child already reaped"))?
            .try_wait()
    }

    fn disarm(&mut self) {
        self.child.take();
    }

    fn terminate(&mut self) -> Result<()> {
        if let Some(child) = self.child.as_mut() {
            terminate_child_process(child)?;
        }
        self.disarm();
        Ok(())
    }
}

impl Drop for ManagedChild {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = terminate_child_process(child);
        }
    }
}

fn validate_exit_status(
    status: ExitStatus,
    expected_exit_codes: &[i32],
    label: &str,
) -> Result<()> {
    if matches_expected_exit(status, expected_exit_codes) {
        return Ok(());
    }
    let rendered = status
        .code()
        .map(|code| code.to_string())
        .unwrap_or_else(|| "terminated by signal".to_string());
    Err(CorsaError::Protocol(CompactString::from(format!(
        "{label} exited with unexpected status {rendered}"
    ))))
}

fn matches_expected_exit(status: ExitStatus, expected_exit_codes: &[i32]) -> bool {
    if let Some(code) = status.code() {
        return expected_exit_codes.contains(&code);
    }
    status.success() && expected_exit_codes.contains(&0)
}

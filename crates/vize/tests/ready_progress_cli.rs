#![cfg(feature = "glyph")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]

use std::{fs, path::Path, process::Command};

fn project() -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    fs::write(
        project.path().join("App.vue"),
        include_str!("fixtures/ready/App.vue"),
    )
    .unwrap();
    fs::write(
        project.path().join("vize.config.json"),
        r#"{"linter":{"enabled":false},"typeChecker":{"enabled":false}}"#,
    )
    .unwrap();
    project
}

fn ready(root: &Path, pattern: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(root)
        .args(["ready", "--config", "vize.config.json", pattern])
        .env_remove("FORCE_COLOR")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("FRESCO_INTERACTIVE")
        .env("NO_COLOR", "1")
        .env("FRESCO_UNICODE", "1")
        .env("TERM", "xterm-256color")
        .env("CI", "0");
    command
}

#[test]
fn redirected_ready_keeps_stage_logs_and_creates_the_real_build_output() {
    let project = project();
    let output = ready(project.path(), "App.vue").output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    let stages: Vec<_> = stderr
        .lines()
        .filter(|line| line.starts_with("vize ready:"))
        .collect();
    assert_eq!(
        stages,
        [
            "vize ready: fmt",
            "vize ready: lint",
            "vize ready: check",
            "vize ready: build"
        ]
    );
    assert!(!stderr.contains("stages completed"));
    assert!(!stderr.contains("[1/4]"));
    assert!(project.path().join("dist/App.js").is_file());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("vize ready"));
}

#[test]
fn redirected_ready_stops_at_the_actual_failing_stage() {
    let project = project();
    let output = ready(project.path(), "Missing.vue").output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("vize ready: fmt\n"));
    assert!(!stderr.contains("vize ready: lint"));
    assert!(!stderr.contains("stages completed"));
    assert!(!project.path().join("dist").exists());
}

#[cfg(unix)]
#[test]
fn terminal_ready_numbers_real_stages_and_keeps_stdout_separate() {
    let project = project();
    let (output, stderr) = terminal_ready(ready(project.path(), "App.vue"));
    assert!(output.status.success(), "{stderr}");
    for expected in [
        "[1/4] Format",
        "[2/4] Lint",
        "[3/4] Type check",
        "[4/4] Build",
        "✓ Ready  4 stages completed in",
    ] {
        assert!(stderr.contains(expected), "missing {expected} in {stderr}");
    }
    assert!(stderr.contains("✓ Format"));
    assert!(stderr.contains("✓ Build"));
    assert!(project.path().join("dist/App.js").is_file());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("vize ready"));
}

#[cfg(unix)]
#[test]
fn terminal_ready_does_not_claim_success_for_an_early_exit() {
    let project = project();
    let (output, stderr) = terminal_ready(ready(project.path(), "Missing.vue"));
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(stderr.contains("[1/4] Format"));
    assert!(!stderr.contains("✓ Format"));
    assert!(!stderr.contains("[2/4]"));
    assert!(!stderr.contains("stages completed"));
    assert!(!stderr.contains('\x1b'));
}

#[cfg(unix)]
fn terminal_ready(mut command: Command) -> (std::process::Output, String) {
    use std::{io::Read, os::fd::FromRawFd, process::Stdio};
    let (mut master, mut slave) = (-1, -1);
    // SAFETY: openpty writes two fresh descriptors; no names or settings are requested.
    let status = unsafe {
        libc::openpty(
            &raw mut master,
            &raw mut slave,
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null(),
        )
    };
    assert_eq!(status, 0, "{}", std::io::Error::last_os_error());
    // SAFETY: each successful openpty descriptor transfers to exactly one File owner.
    let mut master = unsafe { fs::File::from_raw_fd(master) };
    // SAFETY: the distinct slave descriptor also transfers to exactly one File owner.
    let slave = unsafe { fs::File::from_raw_fd(slave) };
    command.stdout(Stdio::piped()).stderr(Stdio::from(slave));
    let child = command.spawn().unwrap();
    drop(command);
    let output = child.wait_with_output().unwrap();
    let mut bytes = Vec::new();
    let mut buffer = [0; 4096];
    loop {
        match master.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => bytes.extend_from_slice(&buffer[..count]),
            Err(error) if error.raw_os_error() == Some(libc::EIO) => break,
            Err(error) => panic!("reading terminal stderr: {error}"),
        }
    }
    (
        output,
        String::from_utf8(bytes).unwrap().replace("\r\n", "\n"),
    )
}

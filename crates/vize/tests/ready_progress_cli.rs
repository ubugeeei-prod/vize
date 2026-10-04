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

const FMT: &str =
    "Found 1 file(s)\nReformatted: App.vue\n\nFormatted 1 file(s)\n  1 file(s) reformatted\n";
const LINT: &str = "[vize] Skipping lint because linter.enabled is false in vize.config.\n";
const CHECK: &str = "[vize] Skipping check because typeChecker.enabled is false in vize.config.\n";
const BUILT: &str = "Built: App.vue -> ./dist/App.js\n";
const BUILD: &str = "\x1b[32m✓ 1 file compiled in <elapsed>s\x1b[0m\n";
const MISSING: &str = "No .vue, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, .tsx, .json, .jsonc, .yaml, .yml, .md, or .markdown files found matching the patterns\n";

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
    assert_eq!(
        normalize_times(&stderr),
        format!(
            "vize ready: fmt\n{FMT}vize ready: lint\n{LINT}vize ready: check\n{CHECK}vize ready: build\n{BUILT}{BUILD}"
        )
    );
    assert!(project.path().join("dist/App.js").is_file());
    assert_eq!(output.stdout, b"");
    assert_eq!(
        fs::read_to_string(project.path().join("App.vue")).unwrap(),
        "<template>\n  <div>Hello</div>\n</template>\n"
    );
}

#[test]
fn redirected_ready_stops_at_the_actual_failing_stage() {
    let project = project();
    let output = ready(project.path(), "Missing.vue").output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        format!("vize ready: fmt\n{MISSING}")
    );
    assert!(!project.path().join("dist").exists());
}

#[cfg(unix)]
#[test]
fn terminal_ready_numbers_real_stages_and_keeps_stdout_separate() {
    let project = project();
    let (output, stderr) = terminal_ready(ready(project.path(), "App.vue"));
    assert!(output.status.success(), "{stderr}");
    assert_eq!(
        normalize_times(&stderr),
        format!(
            "\n  vize ready\n\n  [1/4] Format\n{FMT}  ✓ Format  <elapsed>\n\n  [2/4] Lint\n{LINT}  ✓ Lint  <elapsed>\n\n  [3/4] Type check\n{CHECK}  ✓ Type check  <elapsed>\n\n  [4/4] Build\n{BUILT}{BUILD}  ✓ Build  <elapsed>\n\n  ✓ Ready  4 stages completed in <elapsed>\n\n"
        )
    );
    assert!(project.path().join("dist/App.js").is_file());
    assert_eq!(output.stdout, b"");
}

#[cfg(unix)]
#[test]
fn terminal_ready_does_not_claim_success_for_an_early_exit() {
    let project = project();
    let (output, stderr) = terminal_ready(ready(project.path(), "Missing.vue"));
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        stderr,
        format!("\n  vize ready\n\n  [1/4] Format\n{MISSING}")
    );
}

// Retain every output byte except validated numeric elapsed-time fields. Exact
// full transcripts reject missing, extra, reordered, styled or premature lines.
fn normalize_times(stderr: &str) -> String {
    let mut normalized = String::new();
    for line in stderr.lines() {
        let mut rewritten = false;
        for prefix in [
            "  ✓ Format  ",
            "  ✓ Lint  ",
            "  ✓ Type check  ",
            "  ✓ Build  ",
            "  ✓ Ready  4 stages completed in ",
        ] {
            if let Some(elapsed) = line.strip_prefix(prefix) {
                let (value, unit) = elapsed.split_once(' ').unwrap();
                assert!(matches!(unit, "ms" | "s"));
                validate_duration(value);
                normalized.push_str(prefix);
                normalized.push_str("<elapsed>");
                rewritten = true;
                break;
            }
        }
        if !rewritten {
            if let Some(elapsed) = line.strip_prefix("\x1b[32m✓ 1 file compiled in ") {
                validate_duration(elapsed.strip_suffix("s\x1b[0m").unwrap());
                normalized.push_str(BUILD.trim_end_matches('\n'));
            } else {
                normalized.push_str(line);
            }
        }
        normalized.push('\n');
    }
    normalized
}

fn validate_duration(value: &str) {
    assert!(
        value
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.')
    );
    let value = value.parse::<f64>().unwrap();
    assert!(value.is_finite() && value >= 0.0);
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

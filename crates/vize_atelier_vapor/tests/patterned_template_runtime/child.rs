//! The runtime child's result: its exact passing payload, or full evidence.
//! `Ok(Some(evidence))` is a passing result whose child crashed afterwards.

use serde_json::{Value, json};
use std::{io::Write, process::Output};

pub(crate) fn validate_runtime_output(
    backend: &str,
    source: &str,
    cases: &Value,
    code: &str,
    output: &Output,
) -> Result<Option<String>, String> {
    #[cfg(unix)]
    let signal = {
        use std::os::unix::process::ExitStatusExt;
        output.status.signal()
    };
    #[cfg(not(unix))]
    let signal: Option<i32> = None;
    let evidence = |reason: &str| {
        format!(
            "{backend}: {reason}\nstatus: {}\nexit_code: {:?}\nsignal: {signal:?}\nstdout:\n{}\nstdout_bytes: {:?}\nstderr:\n{}\nstderr_bytes: {:?}\nsource:\n{source}\ncases:\n{cases}\ncode:\n{code}",
            output.status,
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            output.stdout,
            String::from_utf8_lossy(&output.stderr),
            output.stderr,
        )
    };
    let expected = json!({"passed": cases.as_array().unwrap().len()});
    if !output.status.success() {
        // A signal after the exact passing payload is a crash during process
        // teardown (#9067): every case already passed. It is reported, not
        // swallowed; any other failure keeps failing the test.
        if signal.is_some()
            && serde_json::from_slice::<Value>(&output.stdout)
                .ok()
                .as_ref()
                == Some(&expected)
        {
            return Ok(Some(evidence(
                "runtime child crashed after printing a passing result (#9067)",
            )));
        }
        return Err(evidence("runtime child failed"));
    }
    let actual = serde_json::from_slice::<Value>(&output.stdout)
        .map_err(|error| evidence(&format!("invalid runtime JSON: {error}")))?;
    if actual != expected {
        return Err(evidence(&format!(
            "runtime result mismatch: expected {expected}, received {actual}"
        )));
    }
    Ok(None)
}

/// Writes past the test harness's output capture, so a passing test still
/// shows the crash: a GitHub annotation on stdout and the evidence on stderr.
pub(crate) fn report_teardown_crash(evidence: &str) {
    let summary = evidence.lines().next().unwrap_or_default();
    let _ = writeln!(
        std::io::stdout().lock(),
        "::warning title=Runtime child teardown crash::{summary}"
    );
    let _ = writeln!(
        std::io::stderr().lock(),
        "RUNTIME CHILD TEARDOWN CRASH\n{evidence}"
    );
}

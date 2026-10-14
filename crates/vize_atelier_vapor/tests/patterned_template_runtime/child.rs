//! The runtime child's result. A passing print is not enough: the process
//! must exit 0. A signal after `{"passed":N}` is still a failure (#9067).

use serde_json::{Value, json};
use std::process::Output;

pub(crate) fn validate_runtime_output(
    backend: &str,
    source: &str,
    cases: &Value,
    code: &str,
    output: &Output,
) -> Result<(), String> {
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
    if !output.status.success() {
        return Err(evidence("runtime child failed"));
    }
    let actual = serde_json::from_slice::<Value>(&output.stdout)
        .map_err(|error| evidence(&format!("invalid runtime JSON: {error}")))?;
    let expected = json!({"passed": cases.as_array().unwrap().len()});
    if actual != expected {
        return Err(evidence(&format!(
            "runtime result mismatch: expected {expected}, received {actual}"
        )));
    }
    Ok(())
}

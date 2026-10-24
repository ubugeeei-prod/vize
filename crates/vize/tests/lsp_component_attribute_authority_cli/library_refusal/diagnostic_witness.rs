//! Separate batch observation of the identical invalid editor project.

use serde_json::{Value, json};

use super::Fixture;

pub(super) fn capture(fixture: &Fixture) -> Value {
    let binary = env!("CARGO_BIN_EXE_vize");
    let arguments = [
        "check",
        "--show-virtual-ts",
        "--format",
        "json",
        "--corsa-path",
    ];
    let result = std::process::Command::new(binary)
        .current_dir(fixture.project.path())
        .args(arguments)
        .arg(&fixture.runtime)
        .output();
    match result {
        Ok(output) => json!({
            "binary":binary,"arguments":arguments.iter().copied().chain(std::iter::once(fixture.runtime.to_str().unwrap())).collect::<Vec<_>>(),"nativeRuntime":fixture.runtime,
            "project":fixture.project.path(),"success":output.status.success(),
            "exitCode":output.status.code(),"exitStatus":output.status.to_string(),
            "stdoutBytes":output.stdout,"stderrBytes":output.stderr,
            "scope":"Complete public batch stdout (including generated TypeScript and rendered diagnostics), stderr and process status; no raw native RPC or LSP qualification transfers."
        }),
        Err(error) => json!({
            "binary":binary,"arguments":arguments.iter().copied().chain(std::iter::once(fixture.runtime.to_str().unwrap())).collect::<Vec<_>>(),"nativeRuntime":fixture.runtime,
            "project":fixture.project.path(),"spawnError":error.to_string()
        }),
    }
}

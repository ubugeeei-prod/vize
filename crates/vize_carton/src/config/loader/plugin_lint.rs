//! The trusted JS plugin lint sidecar.
//!
//! One Node process per lint run evaluates `vite.config.*` with the shared
//! projector, refuses a plan whose digest differs from the one the native host
//! verified, and runs every admitted lint job from a single request. Plugin
//! callbacks therefore cross the process boundary once per lint run.

use std::{io::Write, path::Path, process::Stdio};

use super::js::runtime_file::NodeRuntimeFile;

const SIDECAR: &str = r#"
const configPath = process.argv[1];
console.log = (...args) => console.error(...args);
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const request = JSON.parse(Buffer.concat(chunks).toString("utf8"));
const namespace = await importConfigModule(configPath);
const exported = namespace.default ?? namespace;
process.stdout.write(JSON.stringify(await runViteConfigPluginLint(exported, configPath, request)));
"#;

/// Run one plugin lint request against `config_path`; returns the response JSON.
pub fn run_plugin_lint(config_path: &Path, request: &[u8]) -> Result<Vec<u8>, String> {
    let runtime = super::js::runtime_source::vite_tools_runtime().map_err(String::from)?;
    let script = [
        runtime.as_str(),
        include_str!("module-runtime.mjs"),
        SIDECAR,
    ]
    .concat();
    let runtime = NodeRuntimeFile::new(&script).map_err(|error| {
        crate::cstr!("failed to prepare the plugin lint sidecar: {error}").into_string()
    })?;
    let mut child = runtime
        .command(config_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| crate::cstr!("failed to start node: {error}").into_string())?;
    // The sidecar reads the whole request before it evaluates or writes anything.
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(request).map_err(|error| {
            crate::cstr!("failed to send the plugin lint request: {error}").into_string()
        })?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| crate::cstr!("the plugin lint sidecar failed: {error}").into_string())?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Surface the plugin refusal itself, not the Node stack around it.
        let refusal = stderr
            .lines()
            .find_map(|line| line.trim().strip_prefix("VizePluginPlanError: "))
            .map(str::to_owned);
        return Err(refusal.unwrap_or_else(|| stderr.trim_end().to_owned()));
    }
    Ok(output.stdout)
}

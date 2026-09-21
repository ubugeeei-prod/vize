//! Run the shared installed-package oracles from the canonical Rust command.
//!
//! These compatibility modules are test helpers, not legacy command entrypoints.
//! Keep their independently authored expectations and package-manager matrix in
//! one place instead of maintaining a weaker second implementation in Rust.

use serde_json::Value;
use std::{env, path::Path, process::Command};

const SCRIPT: &str = r#"
import { pathToFileURL } from "node:url";
import path from "node:path";
const context = JSON.parse(process.argv[1]);
const helper = (name) => import(pathToFileURL(path.join(context.repoRoot, "legacy-tools/npm", name)));
const { runInitTypecheckChecks } = await helper("smoke-release-init-typecheck.mjs");
const { runFreshProjectInitChecks } = await helper("smoke-release-init-fresh.mjs");
context.packed = new Map(Object.entries(context.packed));
context.versions = new Map(Object.entries(context.versions));
runInitTypecheckChecks(context.installDir, context.vizeBin, context.repoRoot, context.peers);
runFreshProjectInitChecks(context);
"#;

pub fn run(install_dir: &Path, context: &Value) -> Result<(), String> {
    let temp_dir = context["tempDir"]
        .as_str()
        .ok_or("installed init oracles require a temporary root")?;
    let output = Command::new(env::var("NODE_BIN").unwrap_or_else(|_| "node".to_string()))
        .args(["--input-type=module", "-e", SCRIPT, &context.to_string()])
        .current_dir(install_dir)
        // Corepack's host-wide last-known-good version/cache must not replace
        // the matrix's pinned manager when it invokes itself during a script.
        .env("COREPACK_HOME", Path::new(temp_dir).join("corepack"))
        .env("COREPACK_ENABLE_DOWNLOAD_PROMPT", "0")
        .env_remove("CORSA_PATH")
        .env_remove("CORSA_EXECUTABLE")
        .env_remove("TSGO_PATH")
        .env_remove("TSGO_EXECUTABLE")
        .output()
        .map_err(|error| format!("cannot run installed init oracles: {error}"))?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if !output.status.success() {
        return Err(format!(
            "installed init oracles failed with {}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

//! JavaScript and TypeScript config evaluation.
//!
//! Native CLI commands still need to support `vize.config.ts` / `.mjs`. Rather
//! than embed a JS runtime, the loader shells out to the local Node runtime,
//! imports the config as ESM, calls function exports with the default env, and
//! returns JSON for Rust deserialization.

use std::{
    io::{Error as IoError, ErrorKind},
    path::Path,
    process::Command,
};

use crate::config::model::RawVizeConfig;

/// Evaluate a JS-like config file through Node and deserialize the result.
pub(super) fn parse_js_config(path: &Path) -> Result<RawVizeConfig, Box<dyn std::error::Error>> {
    let script = r#"
import { pathToFileURL } from "node:url";

const configPath = process.argv[1];
const module = await import(pathToFileURL(configPath).href);
const exported = module.default ?? module;
const config = typeof exported === "function"
  ? await exported({ mode: "development", command: "serve" })
  : exported;
process.stdout.write(JSON.stringify(config ?? {}));
"#;
    let output = Command::new("node")
        .arg("--input-type=module")
        .arg("-e")
        .arg(script)
        .arg(path)
        .current_dir(path.parent().unwrap_or_else(|| Path::new(".")))
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Box::new(IoError::new(
            ErrorKind::InvalidData,
            crate::cstr!("node failed to load config: {}", stderr.trim()).to_string(),
        )));
    }

    Ok(serde_json::from_slice::<RawVizeConfig>(&output.stdout)?)
}

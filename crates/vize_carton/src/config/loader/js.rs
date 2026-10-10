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

use crate::config::ConfigDocument;

/// Evaluate a JS-like config file through Node and deserialize the result.
pub(super) fn parse_js_config(path: &Path) -> Result<ConfigDocument, Box<dyn std::error::Error>> {
    let config_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let config_path = config_path.canonicalize().unwrap_or(config_path);
    let script = [
        include_str!("vite-runtime.mjs"),
        r#"
import { pathToFileURL } from "node:url";
import { dirname } from "node:path";

const configPath = process.argv[1];
console.log = (...args) => console.error(...args);
const module = await import(pathToFileURL(configPath).href);
const exported = module.default ?? module;
const config = isViteConfigFile(configPath)
  ? await resolveViteConfigExport(exported, undefined, dirname(configPath))
  : typeof exported === "function"
  ? await exported({ mode: "development", command: "serve" })
  : exported;
process.stdout.write(JSON.stringify(config ?? {}));
"#,
    ]
    .concat();
    let output = Command::new("node")
        .arg("--input-type=module")
        .arg("-e")
        .arg(script)
        .arg(&config_path)
        .current_dir(config_path.parent().unwrap_or_else(|| Path::new(".")))
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Box::new(IoError::new(
            ErrorKind::InvalidData,
            crate::cstr!("node failed to load config: {}", stderr.trim()).to_string(),
        )));
    }

    // Public Vite settings permit flat entry arrays. Normalize only that shape
    // with the same pure Rust projection used by vize/config; ordinary objects
    // and dedicated config files retain their existing deserialization path.
    if config_path.file_name().is_some_and(|name| {
        super::discovery::CONFIG_FILE_NAMES[5..]
            .iter()
            .any(|candidate| name == *candidate)
    }) && output
        .stdout
        .iter()
        .find(|byte| !byte.is_ascii_whitespace())
        == Some(&b'[')
    {
        let value = serde_json::from_slice(&output.stdout)?;
        let normalized = crate::config::normalize_public_config_value(value)
            .map_err(|error| IoError::new(ErrorKind::InvalidData, error))?;
        return Ok(serde_json::from_value(normalized)?);
    }
    Ok(serde_json::from_slice::<ConfigDocument>(&output.stdout)?)
}

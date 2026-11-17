//! `vize plugin inspect`: the plugin plan of the project's `vite.config.*`.
//!
//! Inspection evaluates the authored export with the same projector as every
//! product, but reports refused capabilities instead of failing on them.

use std::path::{Path, PathBuf};

use super::{
    discovery::{CONFIG_FILE_NAMES, resolve_dir_path, resolve_file_path, search_directories},
    js::runtime_file::NodeRuntimeFile,
};

const INSPECT: &str = r#"
const configPath = process.argv[1];
console.log = (...args) => console.error(...args);
const namespace = await importConfigModule(configPath);
const exported = namespace.default ?? namespace;
const inspection = JSON.stringify(await inspectViteConfigPlugins(exported));
// One line, then exit: a timer the config left running must not keep it alive.
process.stdout.write(inspection + "\n", () => process.exit());
"#;

/// Evaluate the discovered or explicit Vite config and return its plan inspection.
pub fn inspect_config_plugins(path: Option<&Path>) -> Result<serde_json::Value, String> {
    let config_path = vite_config_path(path)?;
    let runtime = super::js::runtime_source::vite_tools_runtime().map_err(String::from)?;
    let script = [
        runtime.as_str(),
        include_str!("module-runtime.mjs"),
        INSPECT,
    ]
    .concat();
    let runtime = NodeRuntimeFile::new(&script).map_err(|error| {
        crate::cstr!("failed to prepare the config runtime: {error}").into_string()
    })?;
    let child = super::js_observation::evaluate(&mut runtime.command(&config_path), None)
        .map_err(|error| crate::cstr!("failed to start node: {error}").into_string())?;
    // Only a complete inspection is written, so a stopped child delivered one.
    if !child.status.success() && !child.stopped_after_result {
        let output = child
            .into_output()
            .map_err(|error| crate::cstr!("failed to read node: {error}").into_string())?;
        return Err(String::from_utf8_lossy(&output.stderr)
            .trim_end()
            .to_owned());
    }
    serde_json::from_slice(&child.stdout)
        .map_err(|error| crate::cstr!("unreadable plugin inspection: {error}").into_string())
}

fn vite_config_path(path: Option<&Path>) -> Result<PathBuf, String> {
    let base = path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    if let Some(file) = resolve_file_path(&base) {
        return Ok(file);
    }
    let directory = resolve_dir_path(&base)
        .ok_or_else(|| crate::cstr!("config file not found: {}", base.display()).into_string())?;
    search_directories(&directory, path.is_none())
        .into_iter()
        .flat_map(|directory| CONFIG_FILE_NAMES.map(|name| directory.join(name)))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| {
            crate::cstr!("no vite.config.* found from {}", directory.display()).into_string()
        })
}

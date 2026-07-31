//! A second, input-less `tsconfig` whose only job is to make the checker report
//! the user's `compilerOptions` diagnostics (#3448).
//!
//! [`super::tsconfig_gen`] deliberately writes a *sanitized* config: the
//! path-sensitive options are stripped so they cannot resolve against the mirror,
//! and the options the native checker removed are rewritten to their nearest
//! working equivalent (`target: ES5` becomes `ES2015`, legacy `moduleResolution`
//! becomes `bundler`). Every one of those edits also erases the diagnostic the
//! option would have produced, so `vize check` silently accepted a config that
//! `tsc` and `vue-tsc` reject:
//!
//! ```text
//! vue-tsc  tsconfig.json(15,5): error TS5101: Option 'baseUrl' is deprecated ...
//! vize     (nothing)
//! ```
//!
//! Options the sanitizer leaves alone were never affected — an unknown option
//! still reaches the checker inside the generated config and is already
//! reported — so the gap is exactly the sanitizer's own edits, and
//! [`option_probe_is_needed`] tests for exactly those.
//!
//! The probe config carries the *unsanitized* options with no inputs at all
//! (`"files": []`, `"include": []`). Config options are validated before any
//! program is built, so the checker still reports every one of them; measured
//! with `tsgo` at ~45 ms against ~200 ms for the same options with a single
//! source file. Nothing is resolved, so the unrebased path options are inert.

use std::path::PathBuf;

use serde_json::{Map, Value};
use vize_carton::profile;

use crate::batch::error::CorsaResult;
use crate::batch::materialize_fs::write_if_changed;

use super::VirtualProject;
use super::tsconfig_paths::parse_jsonc_value;

/// Name of the probe config inside the virtual root. Like the shard configs it
/// is written after materialization and pruned by the next run, so it is not
/// part of the expected materialized file set.
const OPTION_PROBE_CONFIG: &str = "tsconfig.options.json";

impl VirtualProject {
    /// Write the option probe config, or `None` when the generated config
    /// already carries every option the user declared and the main run
    /// therefore reports their diagnostics on its own.
    pub(crate) fn write_option_probe_tsconfig(&self) -> CorsaResult<Option<PathBuf>> {
        let Some(tsconfig_path) = self.resolved_tsconfig_path() else {
            return Ok(None);
        };
        let declared = self.load_compiler_options(Some(tsconfig_path.as_path()))?;
        if declared.is_empty() || !option_probe_is_needed(&declared, &self.generated_options()) {
            return Ok(None);
        }

        let path = self.virtual_root.join(OPTION_PROBE_CONFIG);
        let content = serde_json::to_string_pretty(&option_probe_value(declared))?;
        profile!(
            "canon.project.write_option_probe",
            write_if_changed(&path, content.as_bytes())
        )?;
        Ok(Some(path))
    }

    /// `compilerOptions` of the config `tsconfig_gen` wrote for this run. Read
    /// back from disk rather than recomputed so the comparison can never drift
    /// from what the checker actually sees.
    #[allow(clippy::disallowed_types)]
    fn generated_options(&self) -> Map<std::string::String, Value> {
        let path = self.virtual_root.join("tsconfig.json");
        std::fs::read_to_string(path)
            .ok()
            .and_then(|content| parse_jsonc_value(&content).ok())
            .and_then(|config| {
                config
                    .get("compilerOptions")
                    .and_then(Value::as_object)
                    .cloned()
            })
            .unwrap_or_default()
    }
}

/// Whether the generated config lost or rewrote an option the user declared.
///
/// An option missing from the generated config can no longer produce its
/// diagnostic. So can one whose value was rewritten, but only when the value is
/// what the diagnostic is about — `TS5108: Option 'target=ES5' has been removed`
/// names it, while the re-anchored `paths` map and `typeRoots` list keep their
/// key and with it every diagnostic their presence triggers. Comparing only
/// string-valued options draws exactly that line.
#[allow(clippy::disallowed_types)]
fn option_probe_is_needed(
    declared: &Map<std::string::String, Value>,
    generated: &Map<std::string::String, Value>,
) -> bool {
    declared
        .iter()
        .any(|(name, value)| match generated.get(name) {
            None => true,
            Some(written) => value.is_string() && written != value,
        })
}

/// The probe config: the declared options verbatim, with no inputs.
///
/// Only `types` is overridden, and only to empty, so a `@types` package that
/// resolves from the real tree alone cannot turn into a spurious `TS2688` here;
/// the generated config still carries the user's `types` and reports that case
/// itself. Nothing else is touched — an option added here could *invent* a
/// diagnostic the user's own config does not have (`noEmit` alone conflicts with
/// several emit options), and a program with no inputs emits nothing regardless.
#[allow(clippy::disallowed_types)]
fn option_probe_value(mut declared: Map<std::string::String, Value>) -> Value {
    declared.insert("types".into(), Value::Array(Vec::new()));

    let mut config = Map::new();
    config.insert("compilerOptions".into(), Value::Object(declared));
    config.insert("include".into(), Value::Array(Vec::new()));
    config.insert("files".into(), Value::Array(Vec::new()));
    Value::Object(config)
}

#[cfg(test)]
mod tests;

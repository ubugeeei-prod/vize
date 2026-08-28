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
    ///
    /// Returns the narrowing that reading the probe's output must apply, derived
    /// from the same declared options, so the caller does not re-read the config.
    pub(crate) fn write_option_probe_tsconfig(
        &self,
    ) -> CorsaResult<Option<(PathBuf, OptionDiagnosticNarrowing)>> {
        let Some(tsconfig_path) = self.resolved_tsconfig_path() else {
            return Ok(None);
        };
        let declared = self.load_compiler_options_verbatim(Some(tsconfig_path.as_path()))?;
        if declared.is_empty() || !option_probe_is_needed(&declared, &self.generated_options()) {
            return Ok(None);
        }

        let narrowing = OptionDiagnosticNarrowing::from_declared(
            &declared,
            installed_typescript_reports_removals(&self.project_root),
        );
        let path = self.virtual_root.join(OPTION_PROBE_CONFIG);
        let content = serde_json::to_string_pretty(&option_probe_value(declared))?;
        profile!(
            "canon.project.write_option_probe",
            write_if_changed(&path, content.as_bytes())
        )?;
        Ok(Some((path, narrowing)))
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

/// Which of the probe's option diagnostics the user's own `vue-tsc` toolchain
/// would also report.
///
/// vize's checker is the native TypeScript 7 runtime, which
/// *removed* options that TypeScript 6 merely deprecates and TypeScript 5.x
/// accepts silently. Its `TS5102`/`TS5108` "has been removed" family therefore
/// names exactly the 6.0-era deprecations — measured against `tsc` 6.0.3 and
/// 5.8.3, every option the family fires on (`baseUrl`, `downlevelIteration`,
/// `target=ES5`, `moduleResolution=node10`) errors under 6 as
/// `TS5101`/`TS5107` and is silent under 5.x, while 5.0-era removals surface
/// from the native checker as `TS5023`/`TS6046` instead and stay forwarded.
/// So the family is forwarded when the project's installed `typescript` is 6+
/// (same verdict, different code) and dropped below that (or when no
/// `typescript` is resolvable), where forwarding would report an error on a
/// config the user's `vue-tsc` accepts today — a false positive against the
/// very tool this measures (#3448, #3886).
///
/// This is the deliberate answer to "whose verdict does a vize option diagnostic
/// represent": the user's `vue-tsc` toolchain, not vize's own checker. The cost
/// is that vize cannot warn about what its checker has removed; the alternative
/// costs `vize check` the ability to pass configs that work today.
#[derive(Clone, Copy, Default)]
pub(crate) struct OptionDiagnosticNarrowing {
    /// The user declared `baseUrl`. TypeScript 6 resolves a non-relative `paths`
    /// target against it; TypeScript 7 removed `baseUrl`, so the same target is
    /// `TS5090: Non-relative paths are not allowed`. `baseUrl` plus non-relative
    /// `paths` is the single most common `paths` spelling in Vue projects, so
    /// forwarding that would fire across the ecosystem.
    declares_base_url: bool,
    /// The user set `ignoreDeprecations`, which silences TypeScript 6's
    /// deprecation errors. TypeScript 7 has nothing to silence — the options are
    /// removed rather than deprecated — so it reports them regardless, and a
    /// project that did exactly what TypeScript told it to do would be clean
    /// under `vue-tsc` and an error under `vize` (#3505).
    ignores_deprecations: bool,
    /// The installed `typescript` is 6+, where the 6.0-era deprecations the
    /// removal family names are errors too. Below 6 they are silent, so the
    /// family is dropped wholesale (#3886).
    removals_in_baseline: bool,
}

/// `TS5090: Non-relative paths are not allowed. Did you forget a leading './'?`
const NON_RELATIVE_PATHS: u32 = 5090;

/// The deprecated/removed option family. TypeScript 6 spells them `TS5101`
/// (option) and `TS5107` (option value); TypeScript 7 spells the same two
/// `TS5102` and `TS5108` because it removed rather than deprecated them.
const DEPRECATION_CODES: [u32; 4] = [5101, 5102, 5107, 5108];

impl OptionDiagnosticNarrowing {
    #[allow(clippy::disallowed_types)]
    pub(crate) fn from_declared(
        declared: &Map<std::string::String, Value>,
        removals_in_baseline: bool,
    ) -> Self {
        Self {
            declares_base_url: declared.contains_key("baseUrl"),
            ignores_deprecations: declared.contains_key("ignoreDeprecations"),
            removals_in_baseline,
        }
    }

    /// Whether a probe diagnostic with this code is one `vue-tsc` would report.
    pub(crate) fn keeps(&self, code: u32) -> bool {
        if code == NON_RELATIVE_PATHS && self.declares_base_url {
            return false;
        }
        if DEPRECATION_CODES.contains(&code)
            && (self.ignores_deprecations || !self.removals_in_baseline)
        {
            return false;
        }
        true
    }
}

/// Whether the project's installed TypeScript reports the 6.0-era deprecations
/// the removal family names.
///
/// The nearest `node_modules/typescript/package.json` above the project root is
/// the one Node resolution — and therefore `vue-tsc`'s `typescript` peer — would
/// load. No resolvable install means there is no `vue-tsc` verdict to diverge
/// from, so the conservative answer is `false`: report nothing the user's own
/// toolchain cannot reproduce.
fn installed_typescript_reports_removals(project_root: &std::path::Path) -> bool {
    for dir in project_root.ancestors() {
        let manifest = dir.join("node_modules/typescript/package.json");
        let Ok(content) = std::fs::read_to_string(&manifest) else {
            continue;
        };
        let major = parse_jsonc_value(&content)
            .ok()
            .as_ref()
            .and_then(|package| package.get("version"))
            .and_then(Value::as_str)
            .and_then(|version| version.split('.').next()?.parse::<u32>().ok());
        return major.is_some_and(|major| major >= 6);
    }
    false
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

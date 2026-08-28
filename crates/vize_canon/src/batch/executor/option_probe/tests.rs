use std::path::{Path, PathBuf};

use super::collect_option_diagnostics;
use crate::batch::Diagnostic;
use crate::batch::virtual_project::option_probe::OptionDiagnosticNarrowing;

/// A narrowing derived from the options named in `declared`, using the same
/// derivation the probe itself uses. The baseline is TypeScript 6 — the pinned
/// toolchain of the repo's own parity fixtures — so the removal family is
/// forwarded; [`the_removal_family_is_dropped_on_a_typescript_5_baseline`]
/// covers the 5.x side.
#[allow(clippy::disallowed_types)]
fn narrowing(declared: &[&str]) -> OptionDiagnosticNarrowing {
    narrowing_with_baseline(declared, true)
}

#[allow(clippy::disallowed_types)]
fn narrowing_with_baseline(
    declared: &[&str],
    removals_in_baseline: bool,
) -> OptionDiagnosticNarrowing {
    let mut options = serde_json::Map::new();
    for name in declared {
        options.insert((*name).into(), serde_json::Value::Bool(true));
    }
    OptionDiagnosticNarrowing::from_declared(&options, removals_in_baseline)
}

/// Verbatim `tsgo --pretty false` output for a probe config declaring
/// `baseUrl`, `target: ES5`, `moduleResolution: node`, `downlevelIteration` and
/// an unknown option, captured from a native TypeScript 7 runtime.
const PROBE_OUTPUT: &str = "\
tsconfig.options.json(6,5): error TS5102: Option 'baseUrl' has been removed. Please remove it from your configuration.
  Use '\"paths\": {\"*\": [\"./*\"]}' instead.
tsconfig.options.json(7,15): error TS5108: Option 'target=ES5' has been removed. Please remove it from your configuration.
tsconfig.options.json(8,25): error TS5108: Option 'moduleResolution=node10' has been removed. Please remove it from your configuration.
tsconfig.options.json(9,5): error TS5102: Option 'downlevelIteration' has been removed. Please remove it from your configuration.
tsconfig.options.json(11,5): error TS5023: Unknown compiler option 'importsNotUsedAsValues'.
tsconfig.options.json(14,12): error TS18002: The 'files' list in config file '/tmp/p/tsconfig.options.json' is empty.
";

fn collected(output: &str) -> Vec<(Option<u32>, u8, String)> {
    let mut diagnostics = Vec::new();
    collect_option_diagnostics(
        output,
        Path::new("tsconfig.options.json"),
        &PathBuf::from("/project/tsconfig.json"),
        narrowing(&[]),
        &mut diagnostics,
    );
    assert!(
        diagnostics
            .iter()
            .all(
                |diagnostic: &Diagnostic| diagnostic.file == Path::new("/project/tsconfig.json")
                    && diagnostic.line == 0
                    && diagnostic.column == 0
            ),
        "option diagnostics are anchored on the project: {diagnostics:?}"
    );
    diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                diagnostic.code,
                diagnostic.severity,
                diagnostic.message.to_string(),
            )
        })
        .collect()
}

#[test]
fn every_option_code_is_reported_with_its_continuation_line() {
    assert_eq!(
        collected(PROBE_OUTPUT),
        vec![
            (
                Some(5102),
                1,
                "Option 'baseUrl' has been removed. Please remove it from your configuration.\n\
                 Use '\"paths\": {\"*\": [\"./*\"]}' instead."
                    .to_owned()
            ),
            (
                Some(5108),
                1,
                "Option 'target=ES5' has been removed. Please remove it from your configuration."
                    .to_owned()
            ),
            (
                Some(5108),
                1,
                "Option 'moduleResolution=node10' has been removed. Please remove it from your \
                 configuration."
                    .to_owned()
            ),
            (
                Some(5102),
                1,
                "Option 'downlevelIteration' has been removed. Please remove it from your \
                 configuration."
                    .to_owned()
            ),
            (
                Some(5023),
                1,
                "Unknown compiler option 'importsNotUsedAsValues'.".to_owned()
            ),
        ]
    );
}

#[test]
fn a_continuation_of_a_dropped_diagnostic_is_dropped_with_it() {
    // TS18002 is the price of building no program; its follow-on lines must not
    // be glued onto the option diagnostic above it.
    assert_eq!(
        collected(
            "tsconfig.options.json(6,5): error TS5102: Option 'baseUrl' has been removed.\n\
             tsconfig.options.json(14,12): error TS18002: The 'files' list is empty.\n  \
             and here is a continuation of the dropped one\n"
        ),
        vec![(
            Some(5102),
            1,
            "Option 'baseUrl' has been removed.".to_owned()
        )]
    );
}

#[test]
fn diagnostics_on_any_other_file_are_ignored() {
    // The probe only speaks for its own config; anything the input-less program
    // says about a source file or a lib belongs to the main run.
    assert_eq!(
        collected(
            "src/App.vue.ts(5,23): error TS2345: Argument of type 'number' is not assignable.\n\
             tsconfig.json(6,5): error TS5102: Option 'baseUrl' has been removed.\n"
        ),
        Vec::new()
    );
}

#[test]
fn an_absolute_config_path_is_recognized() {
    let mut diagnostics = Vec::new();
    collect_option_diagnostics(
        "/virtual/root/tsconfig.options.json(6,5): error TS5102: Option 'baseUrl' has been \
         removed.\n",
        Path::new("/virtual/root/tsconfig.options.json"),
        &PathBuf::from("/project/tsconfig.json"),
        narrowing(&[]),
        &mut diagnostics,
    );

    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].code, Some(5102));
}

// -- vue-tsc narrowing (#3448) ------------------------------------------------
//
// vize's checker is TypeScript 7 and `vue-tsc`'s is TypeScript 6. Where the two
// merely spell the same verdict differently the probe forwards what it gets;
// where TypeScript 7 reports an error on a config TypeScript 6 accepts, the
// diagnostic is dropped, because forwarding it is a false positive against the
// tool the parity scorecard measures.

fn collected_with(output: &str, declared: &[&str]) -> Vec<(Option<u32>, u8, String)> {
    let mut diagnostics = Vec::new();
    collect_option_diagnostics(
        output,
        Path::new("tsconfig.options.json"),
        &PathBuf::from("/project/tsconfig.json"),
        narrowing(declared),
        &mut diagnostics,
    );
    diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                diagnostic.code,
                diagnostic.severity,
                diagnostic.message.to_string(),
            )
        })
        .collect()
}

const NON_RELATIVE_PATHS_OUTPUT: &str = "\
tsconfig.options.json(9,7): error TS5090: Non-relative paths are not allowed. Did you forget a leading './'?
";

/// `baseUrl` plus a non-relative `paths` target is the most common `paths`
/// spelling in Vue projects, and it is legal under TypeScript 6 precisely
/// because `baseUrl` resolves it. TypeScript 7 removed `baseUrl`, so the same
/// config becomes `TS5090` — an error on code `vue-tsc` accepts today. The CI
/// `vue-parity` job flagged exactly this on the pinned `vue-element-admin` and
/// `vue2-elm` fixtures.
#[test]
fn non_relative_paths_are_dropped_when_base_url_is_declared() {
    assert_eq!(
        collected_with(NON_RELATIVE_PATHS_OUTPUT, &["baseUrl", "paths"]),
        Vec::new()
    );
}

/// Without `baseUrl` a non-relative target is invalid under both compilers, so
/// the diagnostic is a real one and must survive.
#[test]
fn non_relative_paths_survive_without_base_url() {
    let collected = collected_with(NON_RELATIVE_PATHS_OUTPUT, &["paths"]);
    assert_eq!(collected.len(), 1, "{collected:?}");
    assert_eq!(collected[0].0, Some(5090));
}

const DEPRECATION_OUTPUT: &str = "\
tsconfig.options.json(6,5): error TS5102: Option 'baseUrl' has been removed. Please remove it from your configuration.
tsconfig.options.json(7,5): error TS5108: Option 'target=ES5' has been removed. Please remove it from your configuration.
";

/// `ignoreDeprecations` is what TypeScript 6 tells a user to set, and it
/// silences 6's deprecation errors. TypeScript 7 has nothing to silence — the
/// options are removed rather than deprecated — so a project that did exactly
/// what TypeScript instructed would be clean under `vue-tsc` and an error under
/// `vize` (#3505).
#[test]
fn the_deprecation_family_is_dropped_when_deprecations_are_ignored() {
    assert_eq!(
        collected_with(
            DEPRECATION_OUTPUT,
            &["baseUrl", "target", "ignoreDeprecations"]
        ),
        Vec::new()
    );
}

/// Without `ignoreDeprecations` both compilers agree the config is an error and
/// differ only in the code, so the diagnostic is forwarded as it comes.
#[test]
fn the_deprecation_family_survives_without_ignore_deprecations() {
    let collected = collected_with(DEPRECATION_OUTPUT, &["baseUrl", "target"]);
    assert_eq!(
        collected.iter().map(|entry| entry.0).collect::<Vec<_>>(),
        vec![Some(5102), Some(5108)],
        "{collected:?}"
    );
}

/// TypeScript 5.x accepts every option the removal family names — measured
/// against `tsc` 5.8.3, `baseUrl`, `downlevelIteration`, `target=ES5` and
/// `moduleResolution=node10` are all silent — so on a 5.x baseline the whole
/// family is dropped. A 5.0-era removal never reaches this filter: the native
/// checker reports those as `TS5023`/`TS6046` (`importsNotUsedAsValues` and
/// `target=ES3` in the captured output), which survive and match 5.x's own
/// error verdict.
#[test]
fn the_removal_family_is_dropped_on_a_typescript_5_baseline() {
    let mut diagnostics = Vec::new();
    collect_option_diagnostics(
        PROBE_OUTPUT,
        Path::new("tsconfig.options.json"),
        &PathBuf::from("/project/tsconfig.json"),
        narrowing_with_baseline(&["baseUrl", "downlevelIteration"], false),
        &mut diagnostics,
    );
    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![Some(5023)],
        "{diagnostics:?}"
    );
}

/// The narrowing is per-code, not a blanket mute: an unknown option is wrong
/// under every TypeScript version and is still reported next to a suppressed
/// one.
#[test]
fn an_unrelated_option_diagnostic_is_untouched_by_the_narrowing() {
    let output = "\
tsconfig.options.json(9,7): error TS5090: Non-relative paths are not allowed. Did you forget a leading './'?
tsconfig.options.json(4,5): error TS5023: Unknown compiler option 'nosuchoption'.
";
    let collected = collected_with(output, &["baseUrl", "paths", "ignoreDeprecations"]);
    assert_eq!(
        collected.iter().map(|entry| entry.0).collect::<Vec<_>>(),
        vec![Some(5023)],
        "{collected:?}"
    );
}

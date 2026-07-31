use std::path::{Path, PathBuf};

use super::collect_option_diagnostics;
use crate::batch::Diagnostic;

/// Verbatim `tsgo --pretty false` output for a probe config declaring
/// `baseUrl`, `target: ES5`, `moduleResolution: node`, `downlevelIteration` and
/// an unknown option, captured from
/// `@typescript/native-preview 7.0.0-dev.20260602.1`.
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
        &mut diagnostics,
    );

    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].code, Some(5102));
}

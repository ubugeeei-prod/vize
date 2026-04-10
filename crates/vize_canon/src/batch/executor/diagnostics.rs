use std::path::{Path, PathBuf};

use serde_json::Value;

use super::super::{Diagnostic, VirtualProject};
use crate::corsa_client::LspDiagnostic;
use vize_carton::String;

pub(super) fn map_batch_diagnostics(
    results: Vec<(String, Vec<LspDiagnostic>)>,
    project: &VirtualProject,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (uri, lsp_diagnostics) in results {
        let virtual_path = uri_to_path(uri.as_str());
        for diagnostic in lsp_diagnostics {
            if let Some(diagnostic) = map_lsp_diagnostic(&virtual_path, diagnostic, project) {
                diagnostics.push(diagnostic);
            }
        }
    }

    diagnostics
}

fn map_lsp_diagnostic(
    virtual_path: &Path,
    diagnostic: LspDiagnostic,
    project: &VirtualProject,
) -> Option<Diagnostic> {
    let code = parse_diagnostic_code(diagnostic.code.as_ref());
    if should_skip_diagnostic(code) {
        return None;
    }
    let original = project.map_to_original(
        virtual_path,
        diagnostic.range.start.line,
        diagnostic.range.start.character,
    );

    if let Some(original) = original {
        return Some(Diagnostic {
            file: original.path,
            line: original.line,
            column: original.column,
            message: diagnostic.message,
            code,
            severity: parse_severity(diagnostic.severity),
            block_type: original.block_type,
        });
    }

    None
}

fn uri_to_path(uri: &str) -> PathBuf {
    PathBuf::from(uri.strip_prefix("file://").unwrap_or(uri))
}

fn parse_diagnostic_code(code: Option<&Value>) -> Option<u32> {
    match code {
        Some(Value::Number(value)) => value.as_u64().and_then(|value| u32::try_from(value).ok()),
        Some(Value::String(value)) => value
            .strip_prefix("TS")
            .unwrap_or(value.as_str())
            .parse()
            .ok(),
        _ => None,
    }
}

fn parse_severity(severity: Option<i32>) -> u8 {
    match severity {
        Some(value) if (1..=4).contains(&value) => value as u8,
        _ => 1,
    }
}

pub(super) fn should_skip_diagnostic(code: Option<u32>) -> bool {
    matches!(
        code,
        Some(2307) | Some(2666) | Some(6133) | Some(7006) | Some(7043) | Some(7044)
    )
}

#[cfg(test)]
mod tests {
    use super::{map_batch_diagnostics, parse_diagnostic_code, parse_severity, uri_to_path};
    use crate::batch::VirtualProject;
    use serde_json::json;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use vize_carton::cstr;

    #[test]
    fn parses_numeric_and_string_diagnostic_codes() {
        assert_eq!(parse_diagnostic_code(Some(&json!(2322))), Some(2322));
        assert_eq!(parse_diagnostic_code(Some(&json!("TS2304"))), Some(2304));
        assert_eq!(parse_diagnostic_code(Some(&json!("2551"))), Some(2551));
        assert_eq!(parse_diagnostic_code(Some(&json!(false))), None);
    }

    #[test]
    fn normalizes_lsp_severity() {
        assert_eq!(parse_severity(Some(1)), 1);
        assert_eq!(parse_severity(Some(2)), 2);
        assert_eq!(parse_severity(Some(9)), 1);
        assert_eq!(parse_severity(None), 1);
    }

    #[test]
    fn strips_file_uri_scheme() {
        assert_eq!(
            uri_to_path("file:///workspace/src/App.vue.ts"),
            PathBuf::from("/workspace/src/App.vue.ts")
        );
    }

    #[test]
    fn maps_unmapped_diagnostics_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let project = VirtualProject::new(temp_dir.path()).unwrap();
        let diagnostics = map_batch_diagnostics(
            vec![(
                cstr!("file:///workspace/src/App.vue.ts"),
                vec![crate::corsa_client::LspDiagnostic {
                    range: crate::corsa_client::LspRange {
                        start: crate::corsa_client::LspPosition {
                            line: 3,
                            character: 5,
                        },
                        end: crate::corsa_client::LspPosition {
                            line: 3,
                            character: 12,
                        },
                    },
                    severity: Some(1),
                    code: Some(json!("TS2322")),
                    source: Some("ts".into()),
                    message: "Type 'string' is not assignable to type 'number'.".into(),
                }],
            )],
            &project,
        );

        insta::assert_debug_snapshot!("maps_unmapped_diagnostics_snapshot", diagnostics);
    }
}

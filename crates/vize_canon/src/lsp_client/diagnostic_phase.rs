//! Keep provenance in the existing diagnostic cache entry and read result.

use crate::corsa_bridge::DiagnosticPhase;
use lsp_types::Diagnostic;
use serde::Serialize;
use vize_l0::{FxHashMap, String};

#[derive(Debug, Clone)]
pub(crate) struct PhasedDiagnostic {
    pub(crate) diagnostic: Diagnostic,
    pub(crate) phase: DiagnosticPhase,
}

impl Serialize for PhasedDiagnostic {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.diagnostic.serialize(serializer)
    }
}

pub(super) fn remap_diagnostics(
    diagnostics: Vec<PhasedDiagnostic>,
    mappings: &FxHashMap<String, String>,
) -> Vec<PhasedDiagnostic> {
    let wire = diagnostics
        .iter()
        .map(|row| row.diagnostic.clone())
        .collect::<Vec<_>>();
    match super::utils::remap_serialized_uris(wire, mappings) {
        Ok(mapped) => diagnostics
            .into_iter()
            .zip(mapped)
            .map(|(mut row, diagnostic)| {
                row.diagnostic = diagnostic;
                row
            })
            .collect(),
        Err(_) => diagnostics,
    }
}

impl PhasedDiagnostic {
    pub(crate) fn unknown(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostic,
            phase: DiagnosticPhase::Unknown,
        }
    }
}

impl std::borrow::Borrow<Diagnostic> for PhasedDiagnostic {
    fn borrow(&self) -> &Diagnostic {
        &self.diagnostic
    }
}

impl std::ops::Deref for PhasedDiagnostic {
    type Target = Diagnostic;
    fn deref(&self) -> &Diagnostic {
        &self.diagnostic
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn phase_metadata_never_changes_the_legacy_diagnostic_wire() {
        let diagnostic = Diagnostic {
            code: Some(lsp_types::NumberOrString::Number(1192)),
            message: "No default export.".into(),
            ..Diagnostic::default()
        };
        for phase in [
            DiagnosticPhase::Unknown,
            DiagnosticPhase::Syntactic,
            DiagnosticPhase::Semantic,
            DiagnosticPhase::Suggestion,
            DiagnosticPhase::Declaration,
        ] {
            let record = PhasedDiagnostic {
                diagnostic: diagnostic.clone(),
                phase,
            };
            assert_eq!(
                serde_json::to_value(record).unwrap(),
                serde_json::to_value(&diagnostic).unwrap()
            );
        }
    }

    #[test]
    fn uri_remapping_preserves_phase_and_the_legacy_wire_shape() {
        let native = "file:///mirror/dep.ts";
        let authored = "file:///source/dep.ts";
        let diagnostic = Diagnostic {
            related_information: Some(vec![lsp_types::DiagnosticRelatedInformation {
                location: lsp_types::Location {
                    uri: native.parse().unwrap(),
                    range: lsp_types::Range::default(),
                },
                message: "related".into(),
            }]),
            ..Diagnostic::default()
        };
        let records = remap_diagnostics(
            vec![PhasedDiagnostic {
                diagnostic,
                phase: DiagnosticPhase::Semantic,
            }],
            &[(native.into(), authored.into())].into_iter().collect(),
        );
        assert_eq!(records[0].phase, DiagnosticPhase::Semantic);
        let wire = serde_json::to_value(&records[0]).unwrap();
        assert_eq!(wire["relatedInformation"][0]["location"]["uri"], authored);
        assert!(wire.get("phase").is_none());
        assert!(wire.get("diagnostic").is_none());
    }

    #[test]
    fn an_unphased_transport_never_classifies_a_numeric_code() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/native-diagnostic-phase-11626/native7-lsp-receipt.json"
        )))
        .unwrap();
        let diagnostic: Diagnostic =
            serde_json::from_value(fixture["result"]["items"][0].clone()).unwrap();
        assert_eq!(
            diagnostic.code,
            Some(lsp_types::NumberOrString::Number(1192))
        );
        assert_eq!(
            PhasedDiagnostic::unknown(diagnostic).phase,
            DiagnosticPhase::Unknown
        );
    }
}

//! SARIF textual fixes grouped into distinct artifact changes.

use std::collections::BTreeMap;

use serde::Serialize;

use super::super::plan::{SarifPlan, SarifRegion};
use crate::{FindingFix, TextEdit};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SarifFix<'finding> {
    description: SarifMessage<'finding>,
    artifact_changes: Vec<SarifArtifactChange<'finding>>,
    properties: SarifFixProperties<'finding>,
}

impl<'finding> SarifFix<'finding> {
    pub(super) fn new<'source>(
        plan: &SarifPlan<'finding, 'source>,
        fix: &'finding FindingFix,
    ) -> Self {
        let mut by_path = BTreeMap::<&str, Vec<&TextEdit>>::new();
        for edit in &fix.edits {
            by_path.entry(&edit.location.path).or_default().push(edit);
        }
        let artifact_changes = by_path
            .into_iter()
            .map(|(path, edits)| SarifArtifactChange {
                artifact_location: SarifArtifactLocation {
                    uri: plan.artifact(path).uri().into(),
                },
                replacements: edits
                    .into_iter()
                    .map(|edit| SarifReplacement {
                        deleted_region: plan
                            .region(&edit.location)
                            .expect("fix sources are mandatory during SARIF preflight"),
                        inserted_content: SarifArtifactContent {
                            text: &edit.replacement,
                        },
                    })
                    .collect(),
            })
            .collect();
        Self {
            description: SarifMessage { text: &fix.title },
            artifact_changes,
            properties: SarifFixProperties {
                safety: fix.safety,
                verification: &fix.verification,
            },
        }
    }
}

#[derive(Serialize)]
struct SarifMessage<'finding> {
    text: &'finding str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifArtifactChange<'finding> {
    artifact_location: SarifArtifactLocation,
    replacements: Vec<SarifReplacement<'finding>>,
}

#[derive(Serialize)]
struct SarifArtifactLocation {
    uri: vize_s0::String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifReplacement<'finding> {
    deleted_region: SarifRegion,
    inserted_content: SarifArtifactContent<'finding>,
}

#[derive(Serialize)]
struct SarifArtifactContent<'finding> {
    text: &'finding str,
}

#[derive(Serialize)]
struct SarifFixProperties<'finding> {
    #[serde(rename = "vizeSafety")]
    safety: crate::FixSafety,
    #[serde(rename = "vizeVerification", skip_serializing_if = "Vec::is_empty")]
    verification: &'finding Vec<vize_s0::String>,
}

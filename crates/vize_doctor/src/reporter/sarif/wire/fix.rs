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
    ) -> Result<Self, &'static str> {
        let mut by_path = BTreeMap::<&str, Vec<&TextEdit>>::new();
        for edit in &fix.edits {
            by_path.entry(&edit.location.path).or_default().push(edit);
        }
        let artifact_changes = by_path
            .into_iter()
            .map(|(path, edits)| {
                Ok(SarifArtifactChange {
                    artifact_location: SarifArtifactLocation {
                        uri: plan
                            .artifact_uri(path)
                            .ok_or("SARIF preflight omitted a fix artifact")?
                            .into(),
                    },
                    replacements: edits
                        .into_iter()
                        .map(|edit| {
                            Ok(SarifReplacement {
                                deleted_region: plan
                                    .region(&edit.location)
                                    .ok_or("SARIF preflight omitted a fix region")?,
                                inserted_content: SarifArtifactContent {
                                    text: &edit.replacement,
                                },
                            })
                        })
                        .collect::<Result<Vec<_>, &'static str>>()?,
                })
            })
            .collect::<Result<Vec<_>, &'static str>>()?;
        Ok(Self {
            description: SarifMessage { text: &fix.title },
            artifact_changes,
            properties: SarifFixProperties {
                safety: fix.safety,
                verification: &fix.verification,
            },
        })
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::SarifFix;
    use crate::{
        AnalysisProvenance, DoctorCategory, DoctorFinding, FindingAssessment, FindingConfidence,
        FindingFix, FindingImpact, FindingSeverity, FixSafety, HealthPenalty, RuleCost,
        SourceLocation, TextEdit,
    };

    #[test]
    fn missing_fix_region_fails_instead_of_dropping_an_edit() {
        let report = crate::DoctorReport::new(
            "workspace",
            [DoctorFinding::new(
                "DOCTOR_001",
                DoctorCategory::Accessibility,
                FindingAssessment::new(
                    FindingSeverity::Warning,
                    FindingConfidence::High,
                    FindingImpact::Medium,
                    HealthPenalty::new(1, "fixture"),
                ),
                SourceLocation::new("src/example.vue", 0, 1),
                "Fixture",
                "Fixture finding",
                AnalysisProvenance::new("fixture", RuleCost::Low),
            )],
        );
        let sources = BTreeMap::new();
        let plan = crate::reporter::sarif::plan::SarifPlan::new(
            &report,
            &sources,
            crate::SarifMissingSourcePolicy::ArtifactOnly,
        )
        .expect("artifact-only report passes preflight");
        let fix = FindingFix::new(FixSafety::Safe, "Replace").with_edit(TextEdit::new(
            SourceLocation::new("src/example.vue", 0, 1),
            "x",
        ));

        let error = SarifFix::new(&plan, &fix)
            .err()
            .expect("missing region must fail fix construction");
        assert_eq!(error, "SARIF preflight omitted a fix region");
    }
}

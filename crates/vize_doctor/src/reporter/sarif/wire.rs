//! Allocation-bounded SARIF wire serialization.

mod fix;
mod properties;

use std::collections::BTreeMap;

use serde::{Serialize, Serializer, ser::SerializeSeq};
use vize_s0::cstr;

use super::plan::{SarifPlan, SarifRegion};
use crate::{DoctorFinding, EvidenceKind, FindingEvidence, FindingSeverity, SourceLocation};

use fix::SarifFix;
use properties::{SarifResultProperties, SarifRunProperties};

const SARIF_SCHEMA: &str =
    "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json";

#[derive(Serialize)]
pub(super) struct SarifLog<'plan, 'report, 'source> {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: [SarifRun<'plan, 'report, 'source>; 1],
}

impl<'plan, 'report, 'source> SarifLog<'plan, 'report, 'source> {
    pub(super) fn new(plan: &'plan SarifPlan<'report, 'source>) -> Self {
        Self {
            schema: SARIF_SCHEMA,
            version: "2.1.0",
            runs: [SarifRun {
                tool: SarifTool {
                    driver: SarifDriver {
                        name: "Vize Doctor",
                        semantic_version: env!("CARGO_PKG_VERSION"),
                        rules: SarifRules { plan },
                    },
                },
                column_kind: "unicodeCodePoints",
                results: SarifResults { plan },
                properties: SarifRunProperties::new(plan.report()),
            }],
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRun<'plan, 'report, 'source> {
    tool: SarifTool<'plan, 'report, 'source>,
    column_kind: &'static str,
    results: SarifResults<'plan, 'report, 'source>,
    properties: SarifRunProperties<'report>,
}

#[derive(Serialize)]
struct SarifTool<'plan, 'report, 'source> {
    driver: SarifDriver<'plan, 'report, 'source>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifDriver<'plan, 'report, 'source> {
    name: &'static str,
    semantic_version: &'static str,
    rules: SarifRules<'plan, 'report, 'source>,
}

struct SarifRules<'plan, 'report, 'source> {
    plan: &'plan SarifPlan<'report, 'source>,
}

impl Serialize for SarifRules<'_, '_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.plan.rules().len()))?;
        for finding in self.plan.rules() {
            sequence.serialize_element(&SarifRule::new(finding))?;
        }
        sequence.end()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRule<'finding> {
    id: &'finding str,
    short_description: SarifMessage<'finding>,
    default_configuration: SarifRuleConfiguration,
    properties: SarifRuleProperties<'finding>,
}

impl<'finding> SarifRule<'finding> {
    fn new(finding: &'finding DoctorFinding) -> Self {
        Self {
            id: &finding.code,
            short_description: SarifMessage::borrowed(&finding.title),
            default_configuration: SarifRuleConfiguration {
                level: sarif_level(finding.assessment.severity),
            },
            properties: SarifRuleProperties {
                category: finding.category,
                documentation: finding.documentation.as_deref(),
            },
        }
    }
}

#[derive(Serialize)]
struct SarifRuleConfiguration {
    level: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRuleProperties<'finding> {
    #[serde(rename = "vizeCategory")]
    category: crate::DoctorCategory,
    #[serde(rename = "vizeDocumentation", skip_serializing_if = "Option::is_none")]
    documentation: Option<&'finding str>,
}

struct SarifResults<'plan, 'report, 'source> {
    plan: &'plan SarifPlan<'report, 'source>,
}

impl Serialize for SarifResults<'_, '_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let findings = self.plan.report().findings();
        let mut sequence = serializer.serialize_seq(Some(findings.len()))?;
        for finding in findings {
            sequence.serialize_element(&SarifResult::new(self.plan, finding))?;
        }
        sequence.end()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifResult<'finding> {
    rule_id: &'finding str,
    rule_index: usize,
    level: &'static str,
    message: SarifMessage<'finding>,
    locations: [SarifLocation<'finding>; 1],
    #[serde(skip_serializing_if = "Vec::is_empty")]
    related_locations: Vec<SarifLocation<'finding>>,
    partial_fingerprints: SarifPartialFingerprints,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fixes: Vec<SarifFix<'finding>>,
    properties: SarifResultProperties<'finding>,
}

impl<'finding> SarifResult<'finding> {
    fn new<'source>(plan: &SarifPlan<'finding, 'source>, finding: &'finding DoctorFinding) -> Self {
        let mut related_locations = finding
            .related
            .iter()
            .map(|related| {
                SarifLocation::new(plan, &related.location, Some(&related.message), None)
            })
            .collect::<Vec<_>>();
        related_locations.extend(finding.evidence.iter().filter_map(|evidence| {
            evidence.location.as_ref().map(|location| {
                SarifLocation::new(plan, location, Some(&evidence.summary), Some(evidence))
            })
        }));
        let fixes = finding
            .fix
            .as_ref()
            .filter(|fix| !fix.edits.is_empty() && plan.can_render_fix(fix))
            .map(|fix| vec![SarifFix::new(plan, fix)])
            .unwrap_or_default();
        Self {
            rule_id: &finding.code,
            rule_index: plan.rule_index(&finding.code),
            level: sarif_level(finding.assessment.severity),
            message: SarifMessage::owned(cstr!("{}: {}", finding.title, finding.message)),
            locations: [SarifLocation::new(plan, &finding.primary, None, None)],
            related_locations,
            partial_fingerprints: SarifPartialFingerprints {
                baseline_key: finding.baseline_key(),
            },
            fixes,
            properties: SarifResultProperties::new(plan, finding),
        }
    }
}

#[derive(Serialize)]
struct SarifPartialFingerprints {
    #[serde(rename = "vizeBaselineKey/v1")]
    baseline_key: vize_s0::String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifLocation<'finding> {
    physical_location: SarifPhysicalLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<SarifMessage<'finding>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<SarifLocationProperties<'finding>>,
}

impl<'finding> SarifLocation<'finding> {
    fn new<'source>(
        plan: &SarifPlan<'finding, 'source>,
        location: &'finding SourceLocation,
        message: Option<&'finding str>,
        evidence: Option<&'finding FindingEvidence>,
    ) -> Self {
        Self {
            physical_location: SarifPhysicalLocation {
                artifact_location: SarifArtifactLocation {
                    uri: plan.artifact(&location.path).uri().into(),
                },
                region: plan.region(location),
            },
            message: message.map(SarifMessage::borrowed),
            properties: evidence.map(|evidence| SarifLocationProperties {
                evidence_kind: evidence.kind,
                evidence_details: &evidence.details,
            }),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifPhysicalLocation {
    artifact_location: SarifArtifactLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<SarifRegion>,
}

#[derive(Serialize)]
struct SarifArtifactLocation {
    uri: vize_s0::String,
}

#[derive(Serialize)]
struct SarifLocationProperties<'finding> {
    #[serde(rename = "vizeEvidenceKind")]
    evidence_kind: EvidenceKind,
    #[serde(rename = "vizeEvidenceDetails")]
    evidence_details: &'finding BTreeMap<vize_s0::String, vize_s0::String>,
}

#[derive(Serialize)]
struct SarifMessage<'text> {
    text: SarifMessageText<'text>,
}

enum SarifMessageText<'text> {
    Borrowed(&'text str),
    Owned(vize_s0::String),
}

impl Serialize for SarifMessageText<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Borrowed(text) => serializer.serialize_str(text),
            Self::Owned(text) => serializer.serialize_str(text),
        }
    }
}

impl<'text> SarifMessage<'text> {
    fn borrowed(text: &'text str) -> Self {
        Self {
            text: SarifMessageText::Borrowed(text),
        }
    }

    fn owned(text: vize_s0::String) -> Self {
        Self {
            text: SarifMessageText::Owned(text),
        }
    }
}

const fn sarif_level(severity: FindingSeverity) -> &'static str {
    match severity {
        FindingSeverity::Error => "error",
        FindingSeverity::Warning => "warning",
        FindingSeverity::Notice => "note",
    }
}

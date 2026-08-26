use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, de};
use vize_s0::String;

use super::assessment::{EvidenceKind, RuleCost};
use crate::ContentFingerprint;

/// Authored byte span used by diagnostics, editors, fixes, and reports.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceLocation {
    /// Workspace-relative, slash-normalized source path.
    pub path: String,
    /// Inclusive UTF-8 byte offset.
    pub start: u32,
    /// Exclusive UTF-8 byte offset.
    pub end: u32,
}

impl SourceLocation {
    /// Creates a source location and clamps an inverted end to the start.
    pub fn new(path: impl Into<String>, start: u32, end: u32) -> Self {
        Self {
            path: path.into(),
            start,
            end: end.max(start),
        }
    }

    /// Returns the byte length of the span.
    pub const fn len(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    /// Returns whether the source span is empty.
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

/// Related source span that explains a cross-file or cross-node relationship.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RelatedLocation {
    /// Related authored location.
    pub location: SourceLocation,
    /// Explanation of how the related location contributes to the finding.
    pub message: String,
}

impl RelatedLocation {
    /// Creates related source information.
    pub fn new(location: SourceLocation, message: impl Into<String>) -> Self {
        Self {
            location,
            message: message.into(),
        }
    }
}

/// Application graph context attached to a finding.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FindingContext {
    /// User-visible target identifier. Defaults to absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Execution environment identifier. Defaults to absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    /// Route identifier. Defaults to absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    /// Workspace package identifier. Defaults to absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// Component identifier. Defaults to absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    /// Capability identifier. Defaults to absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,
    /// Build-graph node identifier. Defaults to absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_node: Option<String>,
}

/// Provenance and invalidation contract for one rule result.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnalysisProvenance {
    /// Stable analysis capability identifier.
    pub capability: String,
    /// Expected analysis cost.
    pub cost: RuleCost,
    /// Inputs whose fingerprints invalidate this result. Defaults to empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invalidation_inputs: Vec<String>,
    /// Exact content identities available for invalidation inputs. Defaults to empty.
    ///
    /// Keys are a subset of [`Self::invalidation_inputs`]. Missing entries mean
    /// the producer knows the dependency boundary but could not fingerprint
    /// that input; they never mean unchanged content.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub invalidation_fingerprints: BTreeMap<String, ContentFingerprint>,
}

impl AnalysisProvenance {
    /// Creates provenance with no additional invalidation inputs.
    pub fn new(capability: impl Into<String>, cost: RuleCost) -> Self {
        Self {
            capability: capability.into(),
            cost,
            invalidation_inputs: Vec::new(),
            invalidation_fingerprints: BTreeMap::new(),
        }
    }

    /// Adds stable input identifiers used by incremental analysis.
    pub fn with_invalidation_inputs(
        mut self,
        inputs: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.invalidation_inputs = inputs.into_iter().map(Into::into).collect();
        self.invalidation_inputs.sort();
        self.invalidation_inputs.dedup();
        self.invalidation_fingerprints
            .retain(|input, _| self.invalidation_inputs.binary_search(input).is_ok());
        self
    }

    /// Adds canonical fingerprints and declares their keys as invalidation inputs.
    pub fn with_invalidation_fingerprints(
        mut self,
        fingerprints: BTreeMap<String, ContentFingerprint>,
    ) -> Self {
        self.invalidation_inputs
            .extend(fingerprints.keys().cloned());
        self.invalidation_inputs.sort();
        self.invalidation_inputs.dedup();
        self.invalidation_fingerprints = fingerprints;
        self
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AnalysisProvenanceWire {
    capability: String,
    cost: RuleCost,
    #[serde(default)]
    invalidation_inputs: Vec<String>,
    #[serde(default)]
    invalidation_fingerprints: BTreeMap<String, ContentFingerprint>,
}

impl<'de> Deserialize<'de> for AnalysisProvenance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = AnalysisProvenanceWire::deserialize(deserializer)?;
        let provenance = Self::new(wire.capability, wire.cost)
            .with_invalidation_inputs(wire.invalidation_inputs);
        if let Some(input) = wire
            .invalidation_fingerprints
            .keys()
            .find(|input| provenance.invalidation_inputs.binary_search(input).is_err())
        {
            return Err(de::Error::custom(vize_s0::cstr!(
                "invalidation fingerprint {input:?} has no declared input"
            )));
        }
        Ok(provenance.with_invalidation_fingerprints(wire.invalidation_fingerprints))
    }
}

/// One source, graph, contract, or measurement fact supporting a finding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FindingEvidence {
    /// Analysis domain that produced the evidence.
    pub kind: EvidenceKind,
    /// Concise factual summary.
    pub summary: String,
    /// Supporting authored location. Defaults to absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
    /// Deterministically ordered structured details. Defaults to empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub details: BTreeMap<String, String>,
}

impl FindingEvidence {
    /// Creates evidence without a source location or structured details.
    pub fn new(kind: EvidenceKind, summary: impl Into<String>) -> Self {
        Self {
            kind,
            summary: summary.into(),
            location: None,
            details: BTreeMap::new(),
        }
    }

    /// Attaches the authored location that supports this evidence.
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Adds one stable structured detail.
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }
}

/// One source replacement proposed by a doctor fix.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextEdit {
    /// Authored span to replace.
    pub location: SourceLocation,
    /// UTF-8 replacement source.
    pub replacement: String,
}

impl TextEdit {
    /// Creates a source replacement.
    pub fn new(location: SourceLocation, replacement: impl Into<String>) -> Self {
        Self {
            location,
            replacement: replacement.into(),
        }
    }
}

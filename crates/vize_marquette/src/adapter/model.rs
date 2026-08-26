use serde::{Deserialize, Serialize};
use vize_s0::String;

/// Current serialized adapter-capability manifest format.
pub const ADAPTER_CAPABILITY_FORMAT_VERSION: u32 = 1;

/// Inclusive version range supported for one capability contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdapterCapabilitySupport {
    /// Stable capability identifier from an application marquette.
    pub id: String,
    /// Oldest supported capability contract version, inclusive.
    pub min_version: u32,
    /// Newest supported capability contract version, inclusive.
    pub max_version: u32,
}

/// Versioned, language-neutral capabilities offered by one adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdapterCapabilityManifest {
    /// Serialized manifest format.
    ///
    /// Defaults to `1`.
    #[serde(default = "default_format_version")]
    pub format_version: u32,
    /// Stable adapter identifier shown in diagnostics and reports.
    pub adapter: String,
    /// Supported capability ranges.
    ///
    /// Defaults to an empty list.
    #[serde(default)]
    pub capabilities: Vec<AdapterCapabilitySupport>,
}

const fn default_format_version() -> u32 {
    ADAPTER_CAPABILITY_FORMAT_VERSION
}

/// Stable validation code for an adapter capability manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdapterCapabilityDiagnosticCode {
    /// The serialized manifest format is unsupported.
    InvalidFormatVersion,
    /// The adapter identifier is not portable.
    InvalidAdapterId,
    /// A capability identifier is not portable.
    InvalidCapabilityId,
    /// A version bound is zero.
    InvalidVersion,
    /// The minimum version exceeds the maximum version.
    InvalidVersionRange,
    /// A capability identifier occurs more than once.
    DuplicateCapability,
}

/// Deterministic validation diagnostic for an adapter capability manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdapterCapabilityDiagnostic {
    /// Stable machine-readable diagnostic code.
    pub code: AdapterCapabilityDiagnosticCode,
    /// JSON-style path of the invalid value.
    pub path: String,
    /// Human-readable explanation.
    pub message: String,
}

/// Stable incompatibility code emitted during capability negotiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdapterCapabilityMismatchCode {
    /// The application references an undeclared capability definition.
    UnknownRequirement,
    /// The adapter does not offer a required capability.
    MissingCapability,
    /// The application requires an older contract than the adapter supports.
    VersionBelowMinimum,
    /// The application requires a newer contract than the adapter supports.
    VersionAboveMaximum,
}

/// One failed adapter capability requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdapterCapabilityMismatch {
    /// Stable machine-readable mismatch code.
    pub code: AdapterCapabilityMismatchCode,
    /// Required capability identifier.
    pub capability: String,
    /// JSON-style path of the unsupported application capability requirement.
    pub path: String,
    /// Human-readable explanation with stable wording for renderer diagnostics.
    pub message: String,
    /// Required contract version when the capability is declared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_version: Option<u32>,
    /// Adapter minimum when support exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<u32>,
    /// Adapter maximum when support exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_version: Option<u32>,
}

/// Deterministic result of negotiating requirements with one adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdapterCapabilityNegotiation {
    /// Adapter whose support was inspected.
    pub adapter: String,
    /// Whether the manifest is valid and every requirement is supported.
    pub compatible: bool,
    /// Manifest validation failures, in stable path order.
    pub diagnostics: Vec<AdapterCapabilityDiagnostic>,
    /// Unsupported or unknown requirements, in stable capability order.
    pub mismatches: Vec<AdapterCapabilityMismatch>,
}

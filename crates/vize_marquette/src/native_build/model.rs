use serde::{Deserialize, Serialize};
use vize_l0::String;

/// Stable discriminator for serialized native build profiles.
pub const NATIVE_BUILD_PROFILE_FORMAT: &str = "vize.native-build-profile";

/// Current serialized native build profile version.
pub const NATIVE_BUILD_PROFILE_FORMAT_VERSION: u32 = 1;

/// Native application platforms understood by the managed build contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NativeBuildPlatform {
    /// Android application artifact.
    Android,
    /// Apple iOS application artifact.
    Ios,
}

/// Intended lifecycle stage of a native build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NativeBuildConfiguration {
    /// Fast local development artifact.
    Development,
    /// Distribution-like artifact for pre-production verification.
    Preview,
    /// Production distribution artifact.
    Production,
}

/// Installation or distribution destination produced by one target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NativeBuildDestination {
    /// Platform simulator or emulator.
    Simulator,
    /// Direct device installation.
    Device,
    /// Platform application store.
    Store,
}

/// How signing material is selected without embedding credentials in the profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NativeBuildSigning {
    /// Do not sign the target.
    None,
    /// Let the executor select managed signing material.
    Automatic,
    /// Select explicitly configured signing material.
    Manual,
}

/// One platform and destination emitted from a native build profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeBuildTarget {
    /// Native platform to build.
    pub platform: NativeBuildPlatform,
    /// Installation or distribution destination.
    pub destination: NativeBuildDestination,
    /// Signing selection implemented by the local or hosted executor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signing: Option<NativeBuildSigning>,
}

/// Over-the-air update selection attached to a native build.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeBuildUpdateSelection {
    /// Portable rollout channel identifier.
    pub channel: String,
    /// Runtime compatibility version checked before applying an update.
    pub runtime_version: String,
}

/// Language-neutral profile consumed unchanged by local and hosted native builders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeBuildProfile {
    /// Serialized record discriminator.
    pub format: String,
    /// Serialized contract version.
    pub format_version: u32,
    /// Stable profile identifier.
    pub profile: String,
    /// Intended lifecycle stage of the produced artifacts.
    pub configuration: NativeBuildConfiguration,
    /// Native environment identifier declared by the application marquette.
    pub environment: String,
    /// Non-empty platform and destination matrix.
    pub targets: Vec<NativeBuildTarget>,
    /// Native capabilities required by every target in this profile.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
    /// Optional over-the-air update compatibility selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updates: Option<NativeBuildUpdateSelection>,
}

/// Stable validation code for a native build profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeBuildProfileDiagnosticCode {
    /// The record discriminator is unsupported.
    InvalidFormat,
    /// The serialized profile version is unsupported.
    InvalidFormatVersion,
    /// The profile identifier is not portable.
    InvalidProfileId,
    /// The environment identifier is not portable.
    InvalidEnvironmentId,
    /// The target matrix is empty.
    MissingTarget,
    /// A platform and destination pair occurs more than once.
    DuplicateTarget,
    /// A store target is not a production build.
    StoreRequiresProduction,
    /// A store target does not select signing.
    StoreRequiresSigning,
    /// The update channel identifier is not portable.
    InvalidUpdateChannel,
    /// The runtime compatibility version is not portable.
    InvalidRuntimeVersion,
    /// A capability identifier is not portable.
    InvalidCapabilityId,
    /// A capability identifier occurs more than once.
    DuplicateCapability,
}

/// Deterministic semantic validation diagnostic for a native build profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeBuildProfileDiagnostic {
    /// Stable machine-readable diagnostic code.
    pub code: NativeBuildProfileDiagnosticCode,
    /// JSON-style path of the invalid value.
    pub path: String,
    /// Human-readable explanation shared by every executor.
    pub message: String,
}

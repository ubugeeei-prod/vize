//! Versioned reporter descriptors and semantic capabilities.

mod validation;
mod wire;

use serde::Serialize;
use vize_s0::String;

pub use validation::ReporterContractError;

/// Current machine-readable reporter descriptor contract.
pub const DOCTOR_REPORTER_CONTRACT_VERSION: u32 = 1;

/// How a reporter presents a Doctor report to its consumer.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum ReporterTransport {
    /// One finite document written from a complete report.
    Document,
    /// Independently consumable records suitable for incremental pipelines.
    RecordStream,
    /// A stateful terminal or graphical view controlled by user input.
    Interactive,
}

/// Consumer classes a reporter is designed to serve.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum ReporterAudience {
    /// A person reading a finite report.
    Human,
    /// A continuous-integration or other automated policy consumer.
    Automation,
    /// A code-hosting annotation consumer.
    CodeHost,
    /// An editor, language server, or code action consumer.
    Editor,
    /// A provider-neutral AI context consumer.
    Ai,
}

/// Doctor semantics a reporter promises to preserve in its output.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum ReporterCapability {
    /// Versioned overall and per-category health summaries.
    HealthSummary,
    /// Stable findings and their assessments.
    Findings,
    /// Primary authored source spans.
    PrimaryLocations,
    /// Related authored source spans and their relationships.
    RelatedLocations,
    /// Structured evidence and evidence locations.
    Evidence,
    /// Application target, environment, route, component, and graph context.
    ApplicationContext,
    /// Fix safety, edits, and post-fix verification steps.
    Fixes,
    /// Suppression policy and stable baseline identities.
    Policy,
    /// Analysis capability, cost, and invalidation inputs.
    Provenance,
}

impl ReporterCapability {
    /// Every semantic capability in stable descriptor order.
    pub const ALL: [Self; 9] = [
        Self::HealthSummary,
        Self::Findings,
        Self::PrimaryLocations,
        Self::RelatedLocations,
        Self::Evidence,
        Self::ApplicationContext,
        Self::Fixes,
        Self::Policy,
        Self::Provenance,
    ];
}

/// Versioned machine-readable contract advertised by one reporter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReporterDescriptor {
    contract_version: u32,
    id: String,
    display_name: String,
    format_version: u32,
    media_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_extension: Option<String>,
    transport: ReporterTransport,
    audiences: Vec<ReporterAudience>,
    capabilities: Vec<ReporterCapability>,
}

impl ReporterDescriptor {
    /// Creates a descriptor with format version `1` and no declared consumers.
    ///
    /// Call [`Self::with_audiences`] and [`Self::with_capabilities`] before
    /// registration. [`Self::validate`] documents every accepted identifier,
    /// media-type, extension, and version constraint.
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        media_type: impl Into<String>,
        transport: ReporterTransport,
    ) -> Self {
        Self {
            contract_version: DOCTOR_REPORTER_CONTRACT_VERSION,
            id: id.into(),
            display_name: display_name.into(),
            format_version: 1,
            media_type: media_type.into(),
            file_extension: None,
            transport,
            audiences: Vec::new(),
            capabilities: Vec::new(),
        }
    }

    /// Sets the reporter-specific output format version. Defaults to `1`.
    pub const fn with_format_version(mut self, format_version: u32) -> Self {
        self.format_version = format_version;
        self
    }

    /// Sets the conventional filename extension without a leading dot.
    /// Defaults to absent.
    pub fn with_file_extension(mut self, extension: impl Into<String>) -> Self {
        self.file_extension = Some(extension.into());
        self
    }

    /// Declares intended consumer classes in deterministic order.
    /// Defaults to empty, which validation rejects.
    pub fn with_audiences(mut self, audiences: impl IntoIterator<Item = ReporterAudience>) -> Self {
        self.audiences = audiences.into_iter().collect();
        self.audiences.sort_unstable();
        self.audiences.dedup();
        self
    }

    /// Declares preserved Doctor semantics in deterministic order.
    /// Defaults to empty, which validation rejects.
    pub fn with_capabilities(
        mut self,
        capabilities: impl IntoIterator<Item = ReporterCapability>,
    ) -> Self {
        self.capabilities = capabilities.into_iter().collect();
        self.capabilities.sort_unstable();
        self.capabilities.dedup();
        self
    }

    /// Returns the reporter contract version.
    pub const fn contract_version(&self) -> u32 {
        self.contract_version
    }

    /// Returns the stable reporter identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the user-visible reporter name.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Returns the reporter-specific output format version.
    pub const fn format_version(&self) -> u32 {
        self.format_version
    }

    /// Returns the output media type without environment-dependent parameters.
    pub fn media_type(&self) -> &str {
        &self.media_type
    }

    /// Returns the conventional filename extension without a leading dot.
    pub fn file_extension(&self) -> Option<&str> {
        self.file_extension.as_deref()
    }

    /// Returns the presentation transport.
    pub const fn transport(&self) -> ReporterTransport {
        self.transport
    }

    /// Returns intended consumers in stable order.
    pub fn audiences(&self) -> &[ReporterAudience] {
        &self.audiences
    }

    /// Returns preserved Doctor semantics in stable order.
    pub fn capabilities(&self) -> &[ReporterCapability] {
        &self.capabilities
    }
}

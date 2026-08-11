//! OASIS SARIF 2.1.0 report rendering.

mod plan;
mod source;
mod wire;

#[cfg(test)]
mod tests;

use std::{collections::BTreeMap, io::Write};

use super::{
    DoctorReporter, ReporterAudience, ReporterCapability, ReporterDescriptor, ReporterError,
    ReporterOutput, ReporterTransport,
};
use crate::DoctorReport;

use source::IndexedSource;
pub use source::{SarifMissingSourcePolicy, SarifSource, SarifSourceError};

/// Built-in OASIS SARIF 2.1.0 reporter for code-hosting annotations.
///
/// Doctor locations are UTF-8 byte spans while SARIF text regions use lines
/// and Unicode columns. Callers therefore inject the exact analyzed source via
/// [`Self::with_sources`]. The reporter never reads the filesystem, process
/// environment, clock, network, or code-host credentials.
pub struct SarifReporter<'source> {
    descriptor: ReporterDescriptor,
    sources: BTreeMap<&'source str, IndexedSource<'source>>,
    missing_source_policy: SarifMissingSourcePolicy,
    pretty: bool,
}

impl<'source> SarifReporter<'source> {
    /// Creates a strict reporter with no injected sources.
    ///
    /// Human-readable indentation defaults to `true`. Missing sources default
    /// to [`SarifMissingSourcePolicy::Reject`] so a code host never receives a
    /// silently imprecise annotation.
    pub fn new() -> Self {
        Self {
            descriptor: ReporterDescriptor::new(
                "vize.sarif",
                "Vize Doctor SARIF",
                "application/sarif+json",
                ReporterTransport::Document,
            )
            .with_file_extension("sarif")
            .with_audiences([
                ReporterAudience::Automation,
                ReporterAudience::CodeHost,
                ReporterAudience::Editor,
            ])
            .with_capabilities(ReporterCapability::ALL),
            sources: BTreeMap::new(),
            missing_source_policy: SarifMissingSourcePolicy::Reject,
            pretty: true,
        }
    }

    /// Injects analyzed UTF-8 sources used to translate byte spans.
    ///
    /// Sources may arrive in any order. Paths must be unique, slash-normalized,
    /// workspace-relative paths identical to the paths in the report.
    pub fn with_sources(
        mut self,
        sources: impl IntoIterator<Item = SarifSource<'source>>,
    ) -> Result<Self, SarifSourceError> {
        for source in sources {
            let indexed = IndexedSource::new(source)?;
            if self.sources.insert(source.path(), indexed).is_some() {
                return Err(SarifSourceError::duplicate(source.path()));
            }
        }
        Ok(self)
    }

    /// Selects behavior when a finding references a source that was not
    /// injected. Defaults to [`SarifMissingSourcePolicy::Reject`].
    pub const fn with_missing_source_policy(mut self, policy: SarifMissingSourcePolicy) -> Self {
        self.missing_source_policy = policy;
        self
    }

    /// Selects human-readable indentation. Defaults to `true`.
    pub const fn with_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    /// Returns whether human-readable indentation is enabled.
    pub const fn pretty(&self) -> bool {
        self.pretty
    }

    /// Returns the configured missing-source behavior.
    pub const fn missing_source_policy(&self) -> SarifMissingSourcePolicy {
        self.missing_source_policy
    }
}

impl Default for SarifReporter<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl DoctorReporter for SarifReporter<'_> {
    fn descriptor(&self) -> &ReporterDescriptor {
        &self.descriptor
    }

    fn write_report(
        &self,
        report: &DoctorReport,
        output: &mut ReporterOutput<'_>,
    ) -> Result<(), ReporterError> {
        let plan = plan::SarifPlan::new(report, &self.sources, self.missing_source_policy)?;
        let result = if self.pretty {
            serde_json::to_writer_pretty(&mut *output, &wire::SarifLog::new(&plan))
        } else {
            serde_json::to_writer(&mut *output, &wire::SarifLog::new(&plan))
        };
        result.map_err(|error| {
            if error.is_io() {
                ReporterError::write(error)
            } else {
                ReporterError::encode(error)
            }
        })?;
        output.write_all(b"\n").map_err(ReporterError::write)
    }
}

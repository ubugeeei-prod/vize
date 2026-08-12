//! Built-in stable JSON report rendering.

use std::io::Write;

use super::{
    DoctorReporter, ReporterAudience, ReporterCapability, ReporterDescriptor, ReporterError,
    ReporterOutput, ReporterTransport,
};
use crate::DoctorReport;

/// Built-in stable JSON reporter used by CLI, editor, CI, and AI consumers.
pub struct JsonReporter {
    descriptor: ReporterDescriptor,
    pretty: bool,
}

impl JsonReporter {
    /// Creates the JSON reporter with human-readable indentation enabled.
    /// Pretty printing defaults to `true` for compatibility with the CLI.
    pub fn new() -> Self {
        Self {
            descriptor: ReporterDescriptor::new(
                "vize.json",
                "Vize Doctor JSON",
                "application/vnd.vize.doctor+json",
                ReporterTransport::Document,
            )
            .with_file_extension("json")
            .with_audiences([
                ReporterAudience::Automation,
                ReporterAudience::CodeHost,
                ReporterAudience::Editor,
                ReporterAudience::Ai,
            ])
            .with_capabilities(ReporterCapability::ALL),
            pretty: true,
        }
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
}

impl Default for JsonReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl DoctorReporter for JsonReporter {
    fn descriptor(&self) -> &ReporterDescriptor {
        &self.descriptor
    }

    fn write_report(
        &self,
        report: &DoctorReport,
        output: &mut ReporterOutput<'_>,
    ) -> Result<(), ReporterError> {
        let result = if self.pretty {
            serde_json::to_writer_pretty(&mut *output, report)
        } else {
            serde_json::to_writer(&mut *output, report)
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

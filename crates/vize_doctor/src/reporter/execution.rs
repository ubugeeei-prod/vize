//! Object-safe reporter execution and exact output telemetry.

use std::{
    error::Error,
    fmt,
    io::{self, IoSlice, Write},
};

use vize_s0::{String, ToCompactString};

use super::ReporterDescriptor;
use crate::DoctorReport;

/// Error category reported by a reporter implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReporterErrorKind {
    /// The destination rejected or could not persist output.
    Write,
    /// The reporter could not encode a valid Doctor report.
    Encode,
    /// Reporter-specific configuration or source data was invalid.
    InvalidData,
    /// Rendering was explicitly cancelled by its owner.
    Cancelled,
}

/// Provider-neutral reporter failure with no provider-specific error type leak.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReporterError {
    kind: ReporterErrorKind,
    message: String,
}

impl ReporterError {
    /// Creates an error with a stable category and actionable message.
    pub fn new(kind: ReporterErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// Creates an output write failure.
    pub fn write(error: impl fmt::Display) -> Self {
        Self::new(ReporterErrorKind::Write, error.to_compact_string())
    }

    /// Creates an encoding failure.
    pub fn encode(error: impl fmt::Display) -> Self {
        Self::new(ReporterErrorKind::Encode, error.to_compact_string())
    }

    /// Returns the stable failure category.
    pub const fn kind(&self) -> ReporterErrorKind {
        self.kind
    }

    /// Returns the actionable provider-neutral failure message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ReporterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ReporterError {}

impl From<io::Error> for ReporterError {
    fn from(error: io::Error) -> Self {
        Self::write(error)
    }
}

/// Counted output passed to reporters.
///
/// The wrapper records every successfully written byte, including partial
/// writes before a destination failure. It never buffers the complete report.
pub struct ReporterOutput<'a> {
    inner: &'a mut dyn Write,
    bytes_written: u64,
}

impl<'a> ReporterOutput<'a> {
    fn new(inner: &'a mut dyn Write) -> Self {
        Self {
            inner,
            bytes_written: 0,
        }
    }

    /// Returns bytes accepted by the destination so far.
    pub const fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
}

impl Write for ReporterOutput<'_> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(buffer)?;
        self.bytes_written = self.bytes_written.saturating_add(written as u64);
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    fn write_vectored(&mut self, buffers: &[IoSlice<'_>]) -> io::Result<usize> {
        let written = self.inner.write_vectored(buffers)?;
        self.bytes_written = self.bytes_written.saturating_add(written as u64);
        Ok(written)
    }
}

/// Object-safe integration point implemented by built-in and external reporters.
///
/// Implementations must produce identical bytes for identical descriptors,
/// reports, and explicit reporter configuration. They must not read clocks,
/// random sources, process-global mutable state, or vendor credentials while
/// rendering. I/O and encoding failures must be returned, never printed.
pub trait DoctorReporter: Send + Sync {
    /// Returns the reporter's stable, machine-readable descriptor.
    fn descriptor(&self) -> &ReporterDescriptor;

    /// Writes one normalized report to the counted destination.
    fn write_report(
        &self,
        report: &DoctorReport,
        output: &mut ReporterOutput<'_>,
    ) -> Result<(), ReporterError>;
}

/// Successful reporter execution telemetry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReporterReceipt {
    reporter_id: String,
    reporter_format_version: u32,
    report_format_version: u32,
    findings_emitted: u64,
    bytes_written: u64,
}

impl ReporterReceipt {
    /// Returns the reporter identifier used for this execution.
    pub fn reporter_id(&self) -> &str {
        &self.reporter_id
    }

    /// Returns the reporter-specific output format version.
    pub const fn reporter_format_version(&self) -> u32 {
        self.reporter_format_version
    }

    /// Returns the normalized Doctor report format version.
    pub const fn report_format_version(&self) -> u32 {
        self.report_format_version
    }

    /// Returns the number of normalized findings supplied to the reporter.
    pub const fn findings_emitted(&self) -> u64 {
        self.findings_emitted
    }

    /// Returns bytes accepted by the destination.
    pub const fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
}

/// A rendering failure with reporter identity and partial-output telemetry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReporterFailure {
    /// The reporter descriptor did not satisfy the public contract.
    InvalidContract(super::ReporterContractError),
    /// The reporter failed after writing zero or more bytes.
    Rendering {
        /// Stable reporter identifier.
        reporter_id: String,
        /// Bytes accepted before the failure.
        bytes_written: u64,
        /// Provider-neutral failure.
        error: ReporterError,
    },
}

impl ReporterFailure {
    /// Returns the reporter identifier when descriptor validation reached it.
    pub fn reporter_id(&self) -> Option<&str> {
        match self {
            Self::InvalidContract(_) => None,
            Self::Rendering { reporter_id, .. } => Some(reporter_id),
        }
    }

    /// Returns bytes accepted before failure. Invalid contracts always return zero.
    pub const fn bytes_written(&self) -> u64 {
        match self {
            Self::InvalidContract(_) => 0,
            Self::Rendering { bytes_written, .. } => *bytes_written,
        }
    }
}

impl fmt::Display for ReporterFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidContract(error) => error.fmt(formatter),
            Self::Rendering {
                reporter_id,
                bytes_written,
                error,
            } => write!(
                formatter,
                "reporter {reporter_id} failed after {bytes_written} byte(s): {error}"
            ),
        }
    }
}

impl Error for ReporterFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidContract(error) => Some(error),
            Self::Rendering { error, .. } => Some(error),
        }
    }
}

/// Renders a report and returns deterministic output-size telemetry.
pub fn render_report(
    reporter: &dyn DoctorReporter,
    report: &DoctorReport,
    destination: &mut dyn Write,
) -> Result<ReporterReceipt, ReporterFailure> {
    let descriptor = reporter.descriptor();
    descriptor
        .validate()
        .map_err(ReporterFailure::InvalidContract)?;
    let mut output = ReporterOutput::new(destination);
    if let Err(error) = reporter.write_report(report, &mut output) {
        return Err(ReporterFailure::Rendering {
            reporter_id: descriptor.id().into(),
            bytes_written: output.bytes_written(),
            error,
        });
    }
    output.flush().map_err(|error| ReporterFailure::Rendering {
        reporter_id: descriptor.id().into(),
        bytes_written: output.bytes_written(),
        error: error.into(),
    })?;
    Ok(ReporterReceipt {
        reporter_id: descriptor.id().into(),
        reporter_format_version: descriptor.format_version(),
        report_format_version: report.format_version(),
        findings_emitted: report.findings().len() as u64,
        bytes_written: output.bytes_written(),
    })
}

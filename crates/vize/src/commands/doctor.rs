//! Whole-application health analysis command.

mod analysis;
mod canonical_sfc;
mod discovery;
mod filters;
mod output;
mod tui;

#[cfg(feature = "profiling")]
#[doc(hidden)]
pub use tui::DoctorTuiBenchmark;

#[cfg(test)]
mod tests;

use clap::{Args, ValueEnum};
use std::{fmt, io, path::PathBuf};
use vize_doctor::{
    DoctorCategory, DoctorFilterError, DoctorReport, FindingConfidence, FindingSeverity,
    ReporterFailure, SarifSourceError, application_analysis::ApplicationAnalysisError,
};
use vize_s0::String;

use self::{
    analysis::analyze_application,
    discovery::{DoctorSource, discover_sources},
    output::write_report,
};

/// Output representation for `vize doctor`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum DoctorFormat {
    /// Concise terminal-oriented health report.
    Text,
    /// Stable, versioned JSON report for automation and AI consumers.
    Json,
    /// OASIS SARIF 2.1.0 report for code-hosting annotations.
    Sarif,
}

/// Arguments for whole-application health analysis.
#[derive(Args, Debug)]
#[allow(clippy::disallowed_types)]
pub struct DoctorArgs {
    /// Files or directories to analyze. Defaults to the workspace root.
    #[arg(default_value = ".")]
    pub paths: Vec<String>,

    /// Workspace boundary used for discovery and report paths. Defaults to `.`.
    #[arg(long, default_value = ".")]
    pub root: PathBuf,

    /// Output format. Defaults to `text`.
    #[arg(short, long, value_enum, default_value = "text")]
    pub format: DoctorFormat,

    /// Return success even when the report contains blocking findings. Defaults to false.
    #[arg(long)]
    pub exit_zero: bool,

    /// Explore the report in the interactive Fresco terminal workspace. Disabled by default.
    #[arg(long)]
    pub tui: bool,

    /// Enforce the canonical public component SFC contract. Defaults to false.
    #[arg(long)]
    pub public_sfc: bool,

    /// Include category values (repeat or comma-separate). Defaults to every category.
    #[arg(long = "category", value_delimiter = ',', value_parser = filters::parse_category)]
    pub categories: Vec<DoctorCategory>,

    /// Include severity values (repeat or comma-separate). Defaults to every severity.
    #[arg(long = "severity", value_delimiter = ',', value_parser = filters::parse_severity)]
    pub severities: Vec<FindingSeverity>,

    /// Include confidence values (repeat or comma-separate). Defaults to every confidence.
    #[arg(long = "confidence", value_delimiter = ',', value_parser = filters::parse_confidence)]
    pub confidences: Vec<FindingConfidence>,

    /// Include target identifiers matching a glob. Defaults to every target.
    #[arg(long = "target")]
    pub targets: Vec<String>,

    /// Include stable rule codes matching a glob. Defaults to every rule.
    #[arg(long = "rule")]
    pub rules: Vec<String>,

    /// Include primary workspace-relative paths matching a glob. Defaults to every path.
    #[arg(long = "path")]
    pub path_filters: Vec<String>,

    /// Include route identifiers matching a glob. Defaults to every route.
    #[arg(long = "route")]
    pub routes: Vec<String>,

    /// Include environment identifiers matching a glob. Defaults to every environment.
    #[arg(long = "environment")]
    pub environments: Vec<String>,

    /// Include workspace package identifiers matching a glob. Defaults to every package.
    #[arg(long = "package")]
    pub packages: Vec<String>,

    /// Include findings affected by a changed-file glob. Defaults to every file.
    #[arg(long = "changed-file")]
    pub changed_files: Vec<String>,
}

#[derive(Debug)]
enum DoctorError {
    CurrentDirectory(io::Error),
    InvalidRoot {
        path: PathBuf,
        source: io::Error,
    },
    InvalidInput {
        path: PathBuf,
        reason: &'static str,
    },
    WalkDirectory {
        path: PathBuf,
        source: ignore::Error,
    },
    ReadSource {
        path: PathBuf,
        source: io::Error,
    },
    ParseSfc {
        path: PathBuf,
        message: String,
    },
    ParseScriptModule {
        path: PathBuf,
        message: String,
    },
    ParseSfcScript {
        path: PathBuf,
        block: &'static str,
        message: String,
    },
    Analysis(ApplicationAnalysisError),
    SarifSource(SarifSourceError),
    Filter(DoctorFilterError),
    Report(ReporterFailure),
    Tui(tui::DoctorTuiError),
    Write(io::Error),
}

impl fmt::Display for DoctorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentDirectory(error) => {
                write!(formatter, "cannot read current directory: {error}")
            }
            Self::InvalidRoot { path, source } => {
                write!(
                    formatter,
                    "cannot resolve workspace root {}: {source}",
                    path.display()
                )
            }
            Self::InvalidInput { path, reason } => {
                write!(
                    formatter,
                    "invalid doctor input {}: {reason}",
                    path.display()
                )
            }
            Self::WalkDirectory { path, source } => {
                write!(
                    formatter,
                    "cannot traverse directory {}: {source}",
                    path.display()
                )
            }
            Self::ReadSource { path, source } => {
                write!(formatter, "cannot read source {}: {source}", path.display())
            }
            Self::ParseSfc { path, message } => {
                write!(
                    formatter,
                    "cannot parse component {}: {message}",
                    path.display()
                )
            }
            Self::ParseScriptModule { path, message } => {
                write!(
                    formatter,
                    "cannot parse script module {}: {message}",
                    path.display()
                )
            }
            Self::ParseSfcScript {
                path,
                block,
                message,
            } => {
                write!(
                    formatter,
                    "cannot parse component {block} {}: {message}",
                    path.display()
                )
            }
            Self::Analysis(error) => write!(formatter, "cannot build health report: {error}"),
            Self::SarifSource(error) => write!(formatter, "cannot prepare SARIF source: {error}"),
            Self::Filter(error) => write!(formatter, "cannot apply finding filters: {error}"),
            Self::Report(error) => write!(formatter, "cannot render health report: {error}"),
            Self::Tui(error) => write!(formatter, "cannot run interactive health report: {error}"),
            Self::Write(error) => write!(formatter, "cannot write health report: {error}"),
        }
    }
}

impl From<ApplicationAnalysisError> for DoctorError {
    fn from(error: ApplicationAnalysisError) -> Self {
        Self::Analysis(error)
    }
}

struct DoctorOutcome {
    report: DoctorReport,
    sources: Vec<DoctorSource>,
    root: PathBuf,
    format: DoctorFormat,
    exit_zero: bool,
}

pub fn run(args: DoctorArgs) {
    let capabilities = if args.tui {
        match tui::validate_request(args.format) {
            Ok(capabilities) => Some(capabilities),
            Err(error) => {
                eprintln!("vize doctor: {}", DoctorError::Tui(error));
                std::process::exit(2);
            }
        }
    } else {
        None
    };
    match execute(args) {
        Ok(outcome) => {
            let blocking = outcome.report.summary().has_blocking_errors;
            let result = if let Some(capabilities) = capabilities {
                tui::run(
                    &outcome.report,
                    &outcome.sources,
                    &outcome.root,
                    capabilities,
                )
                .map_err(DoctorError::Tui)
            } else {
                write_report(&outcome.report, outcome.format, &outcome.sources)
            };
            if let Err(error) = result {
                eprintln!("vize doctor: {error}");
                std::process::exit(2);
            }
            if blocking && !outcome.exit_zero {
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("vize doctor: {error}");
            std::process::exit(2);
        }
    }
}

fn execute(args: DoctorArgs) -> Result<DoctorOutcome, DoctorError> {
    let filter = filters::compile(&args).map_err(DoctorError::Filter)?;
    let cwd = std::env::current_dir().map_err(DoctorError::CurrentDirectory)?;
    let requested_root = if args.root.is_absolute() {
        args.root
    } else {
        cwd.join(args.root)
    };
    let root = requested_root
        .canonicalize()
        .map_err(|source| DoctorError::InvalidRoot {
            path: requested_root,
            source,
        })?;
    let sources = discover_sources(&root, &args.paths)?;
    let report = filter.apply(&analyze_application(&root, &sources, args.public_sfc)?);
    Ok(DoctorOutcome {
        report,
        sources,
        root,
        format: args.format,
        exit_zero: args.exit_zero,
    })
}

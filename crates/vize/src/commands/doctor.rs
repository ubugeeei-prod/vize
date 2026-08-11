//! Whole-application health analysis command.

mod analysis;
mod canonical_sfc;
mod discovery;
mod output;

#[cfg(test)]
mod tests;

use clap::{Args, ValueEnum};
use std::{fmt, io, path::PathBuf};
use vize_carton::String;
use vize_doctor::{DoctorReport, ReporterFailure, application_analysis::ApplicationAnalysisError};

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

    /// Enforce the canonical public component SFC contract. Defaults to false.
    #[arg(long)]
    pub public_sfc: bool,
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
    Report(ReporterFailure),
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
            Self::Report(error) => write!(formatter, "cannot render health report: {error}"),
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
    format: DoctorFormat,
    exit_zero: bool,
}

pub fn run(args: DoctorArgs) {
    match execute(args) {
        Ok(outcome) => {
            let blocking = outcome.report.summary().has_blocking_errors;
            if let Err(error) = write_report(&outcome.report, outcome.format) {
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
    let report = analyze_application(&root, &sources, args.public_sfc)?;
    Ok(DoctorOutcome {
        report,
        format: args.format,
        exit_zero: args.exit_zero,
    })
}

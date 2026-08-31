//! Bench-report JSON export.
//!
//! Every bench run is exported as one JSON file under
//! `tools/benchmarks/results/davinci/<bench_id>.json` (workspace-relative), shaped by
//! `schema/davinci-bench.schema.json`. In debug builds the exporter validates
//! each report against the committed schema before writing; the validator is
//! strict - a schema keyword it does not implement is an error, never a
//! silently skipped check.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use davinci_test_support::schema::{SchemaError, validate};
use serde::Serialize;

/// Version stamped into every report as `harness_version`.
pub const HARNESS_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Report directory relative to the workspace root.
pub const RESULTS_SUBDIR: &str = "tools/benchmarks/results/davinci";

/// Schema location relative to the workspace root.
pub const SCHEMA_SUBPATH: &str = "tools/benchmarks/crates/davinci_harness/schema/davinci-bench.schema.json";

/// Wall-clock percentiles in nanoseconds over the criterion measurement samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct WallNs {
    /// Median wall time per iteration.
    pub p50: u64,
    /// 95th-percentile wall time per iteration.
    pub p95: u64,
}

/// One bench run, as exported to `tools/benchmarks/results/davinci/<bench_id>.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BenchReport<'a> {
    /// Stable bench identity; doubles as the report file name.
    pub bench_id: &'a str,
    /// Fixture identity (fixture path or a `synthetic:` tag).
    pub fixture: &'a str,
    /// Operating system that produced allocation metrics.
    pub platform: &'a str,
    /// Wall-clock percentiles.
    pub wall_ns: WallNs,
    /// Allocation-like calls; `None` when the counting allocator is not installed.
    pub allocs: Option<u64>,
    /// Peak live-byte growth over the measured window; `None` as above.
    pub alloc_bytes_peak: Option<u64>,
    /// Peak-RSS growth over the process baseline; `None` off macOS/Linux.
    pub rss_peak_bytes: Option<u64>,
    /// Producing harness version.
    pub harness_version: &'a str,
}

/// Everything that can go wrong while exporting a report.
#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("bench id `{0}` has characters outside [A-Za-z0-9._-]")]
    InvalidBenchId(Box<str>),
    #[error("no `[workspace]` Cargo.toml found above `{}`", .0.display())]
    WorkspaceRootNotFound(PathBuf),
    #[error("schema violation at `{path}`: expected {expected}, found {found}")]
    SchemaType {
        path: Box<str>,
        expected: Box<str>,
        found: Box<str>,
    },
    #[error("schema violation at `{path}`: value does not equal const `{expected}`")]
    SchemaConst { path: Box<str>, expected: Box<str> },
    #[error("schema violation at `{path}`: missing required property `{property}`")]
    SchemaRequired { path: Box<str>, property: Box<str> },
    #[error("schema violation at `{path}`: unexpected property `{property}`")]
    SchemaUnexpectedProperty { path: Box<str>, property: Box<str> },
    #[error("schema violation at `{path}`: value is below minimum {minimum}")]
    SchemaMinimum { path: Box<str>, minimum: u64 },
    #[error("schema violation at `{path}`: string is shorter than minLength {min_length}")]
    SchemaMinLength { path: Box<str>, min_length: u64 },
    #[error("schema violation at `{path}`: string does not match pattern `{pattern}`")]
    SchemaPattern { path: Box<str>, pattern: Box<str> },
    #[error("schema pattern `{pattern}` at `{path}` does not compile")]
    SchemaBadPattern { path: Box<str>, pattern: Box<str> },
    #[error(
        "schema keyword `{keyword}` at `{path}` is not implemented by the davinci_harness validator"
    )]
    SchemaUnimplementedKeyword { path: Box<str>, keyword: Box<str> },
    #[error("schema at `{path}` must be a JSON object")]
    SchemaNotAnObject { path: Box<str> },
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl From<SchemaError> for ReportError {
    fn from(error: SchemaError) -> Self {
        match error {
            SchemaError::NotObject { path } => Self::SchemaNotAnObject {
                path: path.as_str().into(),
            },
            SchemaError::UnimplementedKeyword { path, keyword } => {
                Self::SchemaUnimplementedKeyword {
                    path: path.as_str().into(),
                    keyword: keyword.as_str().into(),
                }
            }
            SchemaError::Type {
                path,
                expected,
                found,
            } => Self::SchemaType {
                path: path.as_str().into(),
                expected: expected.as_str().into(),
                found: found.into(),
            },
            SchemaError::Const { path, expected } => Self::SchemaConst {
                path: path.as_str().into(),
                expected: expected.as_str().into(),
            },
            SchemaError::Required { path, property } => Self::SchemaRequired {
                path: path.as_str().into(),
                property: property.as_str().into(),
            },
            SchemaError::UnexpectedProperty { path, property } => Self::SchemaUnexpectedProperty {
                path: path.as_str().into(),
                property: property.as_str().into(),
            },
            SchemaError::Minimum { path, minimum } => Self::SchemaMinimum {
                path: path.as_str().into(),
                minimum,
            },
            SchemaError::MinLength { path, min_length } => Self::SchemaMinLength {
                path: path.as_str().into(),
                min_length,
            },
            SchemaError::Pattern { path, pattern } => Self::SchemaPattern {
                path: path.as_str().into(),
                pattern: pattern.as_str().into(),
            },
            SchemaError::BadPattern { path, pattern } => Self::SchemaBadPattern {
                path: path.as_str().into(),
                pattern: pattern.as_str().into(),
            },
        }
    }
}

/// Reject bench ids that cannot serve as report file names.
pub fn validate_bench_id(bench_id: &str) -> Result<(), ReportError> {
    let valid = !bench_id.is_empty()
        && bench_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
    if valid {
        Ok(())
    } else {
        Err(ReportError::InvalidBenchId(bench_id.into()))
    }
}

/// Walk up from the current directory to the `[workspace]` Cargo.toml.
///
/// Cargo runs bench and test binaries with the package directory as the
/// working directory, so this resolves deterministically for every workspace
/// member without environment configuration.
pub fn workspace_root() -> Result<PathBuf, ReportError> {
    let start = std::env::current_dir()?;
    let mut dir = start.as_path();
    loop {
        let manifest = dir.join("Cargo.toml");
        if manifest.is_file() {
            let text = fs::read_to_string(&manifest)?;
            if text.lines().any(|line| line.trim() == "[workspace]") {
                return Ok(dir.to_path_buf());
            }
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => return Err(ReportError::WorkspaceRootNotFound(start)),
        }
    }
}

/// Write `report` to `tools/benchmarks/results/davinci/<bench_id>.json` under the
/// workspace root, returning the written path.
pub fn write(report: &BenchReport<'_>) -> Result<PathBuf, ReportError> {
    let dir = workspace_root()?.join(RESULTS_SUBDIR);
    write_to_dir(&dir, report)
}

/// Write `report` into an explicit directory (test seam for [`write`]).
pub fn write_to_dir(dir: &Path, report: &BenchReport<'_>) -> Result<PathBuf, ReportError> {
    validate_bench_id(report.bench_id)?;
    #[cfg(debug_assertions)]
    validate_against_schema(&serde_json::to_value(report)?)?;
    fs::create_dir_all(dir)?;
    let mut file_name = report.bench_id.to_owned();
    file_name.push_str(".json");
    let path = dir.join(file_name);
    let mut text = serde_json::to_string_pretty(report)?;
    text.push('\n');
    fs::write(&path, text)?;
    Ok(path)
}

/// Load the committed schema from the workspace.
pub fn load_schema() -> Result<serde_json::Value, ReportError> {
    let path = workspace_root()?.join(SCHEMA_SUBPATH);
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

/// Validate a serialized report against the committed schema.
pub fn validate_against_schema(report_json: &serde_json::Value) -> Result<(), ReportError> {
    let schema = load_schema()?;
    validate(&schema, report_json, "$").map_err(ReportError::from)
}

#[cfg(test)]
mod tests;

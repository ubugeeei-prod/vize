//! One private original-path selection and configured HTML execution operation.

use std::path::PathBuf;
use std::time::{Duration, Instant};

#[cfg(feature = "napi")]
use super::super::lint_fix::lint_source;
use super::file_collection::oxlint_html_profile as profile;
#[cfg(all(test, not(feature = "napi")))]
use crate::lint_fix_tests::lint_source;
use profile::{Refusal, RefusalKind, Request, Selection};
use vize_l0::{String, ToCompactString};
use vize_patina::{LintResult, Linter, Severity};

mod config;
mod root_json;
mod rule_options;
mod settings;
#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(super) struct ProjectedRule {
    pub name: String,
    pub severity: Option<Severity>,
    pub active: bool,
    pub authored_options: Vec<serde_json::Value>,
}

#[derive(Debug)]
pub(super) struct ProjectedSettings {
    pub locale: String,
    pub help_level: String,
    pub preset: String,
}

#[derive(Debug)]
pub(super) struct Projection {
    pub rules: Vec<ProjectedRule>,
    pub settings: ProjectedSettings,
    pub deny_warnings: bool,
}

pub(super) struct FileResult {
    pub path: PathBuf,
    pub result: LintResult,
}

pub(super) struct Operation {
    pub selection: Selection,
    pub projection: Projection,
    pub files: Vec<FileResult>,
    pub executed_file_count: usize,
    pub elapsed: Duration,
}

pub(super) fn run(request: Request<'_>, expected_root: &[u8]) -> Result<Operation, Refusal> {
    run_with(request, expected_root, lint_source)
}

/// The private executor seam also lets laws wrap the unchanged real HTML engine.
pub(super) fn run_with(
    request: Request<'_>,
    expected_root: &[u8],
    mut lint: impl FnMut(&Linter, &str, &str) -> LintResult,
) -> Result<Operation, Refusal> {
    let start = Instant::now();
    let (selection, plan) = profile::select_with_root_decoder(request, |path, bytes| {
        config::decode(path, bytes, expected_root)
    })?;
    let mut files = Vec::with_capacity(selection.originals.len());
    for original in &selection.originals {
        let source = std::str::from_utf8(&original.bytes).map_err(|error| Refusal {
            kind: RefusalKind::SourceEncoding,
            path: original.path.clone(),
            details: error.to_compact_string(),
            original_bytes: Some(original.bytes.clone()),
        })?;
        // The selector already admits UTF-8 original paths; do not project one.
        let filename = original.path.to_str().ok_or_else(|| Refusal {
            kind: RefusalKind::Internal,
            path: original.path.clone(),
            details: "selected original lost its admitted UTF-8 filename".into(),
            original_bytes: Some(original.bytes.clone()),
        })?;
        let result = lint(&plan.linter, source, filename);
        files.push(FileResult {
            path: original.path.clone(),
            result,
        });
    }
    Ok(Operation {
        executed_file_count: files.len(),
        selection,
        projection: plan.projection,
        files,
        elapsed: start.elapsed(),
    })
}

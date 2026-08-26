//! Deterministic, reporter-neutral finding filters.

mod matcher;

#[cfg(test)]
mod tests;

use std::fmt;

use serde::{Deserialize, Serialize};
use vize_s0::String;

use crate::{DoctorCategory, DoctorFinding, DoctorReport, FindingConfidence, FindingSeverity};

use self::matcher::PatternSet;

/// Serializable selection contract shared by CLI, TUI, editor, and AI clients.
///
/// Empty dimensions accept every value. Values within one dimension are joined
/// with OR, while populated dimensions are joined with AND. String dimensions
/// accept shell-style `*`, `?`, character classes, and recursive `**` globs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DoctorFilterSpec {
    /// Accepted health categories. Defaults to every category.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<DoctorCategory>,
    /// Accepted gate severities. Defaults to every severity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub severities: Vec<FindingSeverity>,
    /// Accepted evidence confidence levels. Defaults to every confidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub confidences: Vec<FindingConfidence>,
    /// Target identifier globs. Defaults to every target, including absent context.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<String>,
    /// Stable rule-code globs. Defaults to every rule.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<String>,
    /// Primary workspace-relative path globs. Defaults to every path.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,
    /// Route identifier globs. Defaults to every route, including absent context.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routes: Vec<String>,
    /// Environment identifier globs. Defaults to every environment, including absent context.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub environments: Vec<String>,
    /// Workspace package identifier globs. Defaults to every package, including absent context.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<String>,
    /// Changed-file globs matched against all source evidence and invalidation inputs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_files: Vec<String>,
}

impl DoctorFilterSpec {
    /// Compiles and validates every string pattern.
    pub fn compile(&self) -> Result<DoctorFilter, DoctorFilterError> {
        DoctorFilter::compile(self)
    }

    fn normalized(&self) -> Self {
        let mut normalized = self.clone();
        sort_dedup(&mut normalized.categories);
        sort_dedup(&mut normalized.severities);
        sort_dedup(&mut normalized.confidences);
        normalize_patterns(&mut normalized.targets, false);
        normalize_patterns(&mut normalized.rules, false);
        normalize_patterns(&mut normalized.paths, true);
        normalize_patterns(&mut normalized.routes, false);
        normalize_patterns(&mut normalized.environments, false);
        normalize_patterns(&mut normalized.packages, false);
        normalize_patterns(&mut normalized.changed_files, true);
        normalized
    }
}

/// Validated finding predicate with precompiled string matchers.
#[derive(Debug, Clone)]
pub struct DoctorFilter {
    spec: DoctorFilterSpec,
    targets: PatternSet,
    rules: PatternSet,
    paths: PatternSet,
    routes: PatternSet,
    environments: PatternSet,
    packages: PatternSet,
    changed_files: PatternSet,
}

impl DoctorFilter {
    /// Compiles a filter spec, rejecting malformed or empty glob patterns.
    pub fn compile(spec: &DoctorFilterSpec) -> Result<Self, DoctorFilterError> {
        let spec = spec.normalized();
        Ok(Self {
            targets: PatternSet::compile(DoctorFilterDimension::Target, &spec.targets)?,
            rules: PatternSet::compile(DoctorFilterDimension::Rule, &spec.rules)?,
            paths: PatternSet::compile(DoctorFilterDimension::Path, &spec.paths)?,
            routes: PatternSet::compile(DoctorFilterDimension::Route, &spec.routes)?,
            environments: PatternSet::compile(
                DoctorFilterDimension::Environment,
                &spec.environments,
            )?,
            packages: PatternSet::compile(DoctorFilterDimension::Package, &spec.packages)?,
            changed_files: PatternSet::compile(
                DoctorFilterDimension::ChangedFile,
                &spec.changed_files,
            )?,
            spec,
        })
    }

    /// Returns the normalized, deterministically ordered source spec.
    pub const fn spec(&self) -> &DoctorFilterSpec {
        &self.spec
    }

    /// Returns whether a finding satisfies every populated dimension.
    pub fn matches(&self, finding: &DoctorFinding) -> bool {
        matches_enum(&self.spec.categories, finding.category)
            && matches_enum(&self.spec.severities, finding.assessment.severity)
            && matches_enum(&self.spec.confidences, finding.assessment.confidence)
            && self.rules.matches(&finding.code)
            && self.paths.matches_path(&finding.primary.path)
            && self
                .targets
                .matches_optional(finding.context.target.as_deref())
            && self
                .routes
                .matches_optional(finding.context.route.as_deref())
            && self
                .environments
                .matches_optional(finding.context.environment.as_deref())
            && self
                .packages
                .matches_optional(finding.context.package.as_deref())
            && self.matches_changed_files(finding)
    }

    /// Creates a newly scored report containing only matching findings.
    pub fn apply(&self, report: &DoctorReport) -> DoctorReport {
        DoctorReport::new(
            report.workspace(),
            report
                .findings()
                .iter()
                .filter(|finding| self.matches(finding))
                .cloned(),
        )
    }

    fn matches_changed_files(&self, finding: &DoctorFinding) -> bool {
        self.changed_files.is_unrestricted()
            || finding_paths(finding).any(|path| self.changed_files.matches_path(path))
    }
}

/// String-valued filter dimension associated with a pattern error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorFilterDimension {
    /// Application target.
    Target,
    /// Stable rule code.
    Rule,
    /// Primary source path.
    Path,
    /// Application route.
    Route,
    /// Execution environment.
    Environment,
    /// Workspace package.
    Package,
    /// Changed source or invalidation input.
    ChangedFile,
}

impl fmt::Display for DoctorFilterDimension {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Target => "target",
            Self::Rule => "rule",
            Self::Path => "path",
            Self::Route => "route",
            Self::Environment => "environment",
            Self::Package => "package",
            Self::ChangedFile => "changed-file",
        })
    }
}

/// Invalid string pattern supplied to a doctor filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorFilterError {
    dimension: DoctorFilterDimension,
    pattern: String,
    reason: String,
}

impl DoctorFilterError {
    pub(crate) fn new(
        dimension: DoctorFilterDimension,
        pattern: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            dimension,
            pattern: pattern.into(),
            reason: reason.into(),
        }
    }

    /// Returns the dimension containing the invalid pattern.
    pub const fn dimension(&self) -> DoctorFilterDimension {
        self.dimension
    }

    /// Returns the rejected pattern exactly as normalized for matching.
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Returns the glob compiler's bounded explanation.
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl fmt::Display for DoctorFilterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid {} filter pattern {:?}: {}",
            self.dimension, self.pattern, self.reason
        )
    }
}

impl std::error::Error for DoctorFilterError {}

fn finding_paths(finding: &DoctorFinding) -> impl Iterator<Item = &str> {
    std::iter::once(finding.primary.path.as_str())
        .chain(
            finding
                .related
                .iter()
                .map(|related| related.location.path.as_str()),
        )
        .chain(
            finding
                .evidence
                .iter()
                .filter_map(|evidence| evidence.location.as_ref())
                .map(|location| location.path.as_str()),
        )
        .chain(
            finding
                .fix
                .iter()
                .flat_map(|fix| &fix.edits)
                .map(|edit| edit.location.path.as_str()),
        )
        .chain(
            finding
                .provenance
                .invalidation_inputs
                .iter()
                .map(String::as_str),
        )
}

fn matches_enum<T: PartialEq>(accepted: &[T], value: T) -> bool {
    accepted.is_empty() || accepted.contains(&value)
}

fn sort_dedup<T: Ord>(values: &mut Vec<T>) {
    values.sort();
    values.dedup();
}

fn normalize_patterns(patterns: &mut Vec<String>, path: bool) {
    for pattern in patterns.iter_mut() {
        *pattern = pattern.trim().into();
        if path && pattern.contains('\\') {
            *pattern = pattern.replace('\\', "/").into();
        }
    }
    sort_dedup(patterns);
}

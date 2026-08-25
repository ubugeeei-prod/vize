//! Corpus-root preflight for Davinci real-project differential lanes.
//!
//! The canonical closure corpus is the current checkout's own
//! `tests/_fixtures/_git` gitlink tree. Arbitrary roots can still be swept as
//! smoke shards, but they are labeled as non-closure evidence and do not claim
//! the phase gate.

use std::env;
use std::fmt;
use std::path::{Component, Path, PathBuf};

use vize_s0::String;

mod preflight;

use preflight::assert_canonical_hydrated;

pub const DIFFERENTIAL_CORPUS_ENV: &str = "VIZE_DAVINCI_DIFFERENTIAL_CORPUS";
pub const CANONICAL_CORPUS_ROOT: &str = "tests/_fixtures/_git";

const HYDRATE_COMMAND: &str =
    "git submodule update --init --checkout --force -- tests/_fixtures/_git";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorpusScope {
    ClosureEvidence,
    SmokeShard,
}

impl CorpusScope {
    #[must_use]
    pub const fn closure_evidence(self) -> bool {
        matches!(self, Self::ClosureEvidence)
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ClosureEvidence => "canonical-closure",
            Self::SmokeShard => "external-smoke",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusRoot {
    path: PathBuf,
    scope: CorpusScope,
}

impl CorpusRoot {
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub const fn scope(&self) -> CorpusScope {
        self.scope
    }

    #[must_use]
    pub const fn closure_evidence(&self) -> bool {
        self.scope.closure_evidence()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorpusPreflightError {
    Git { command: String, detail: String },
    Hydration(Box<HydrationReport>),
}

impl fmt::Display for CorpusPreflightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Git { command, detail } => write!(f, "`{command}` failed: {detail}"),
            Self::Hydration(report) => report.fmt(f),
        }
    }
}

impl std::error::Error for CorpusPreflightError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HydrationReport {
    root: String,
    indexed: usize,
    clean: usize,
    missing: Vec<String>,
    drifted: Vec<String>,
    conflicted: Vec<String>,
    invalid: Vec<String>,
    inventory_mismatch: Vec<String>,
}

impl HydrationReport {
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.indexed > 0
            && self.clean == self.indexed
            && self.missing.is_empty()
            && self.drifted.is_empty()
            && self.conflicted.is_empty()
            && self.invalid.is_empty()
            && self.inventory_mismatch.is_empty()
    }
}

impl fmt::Display for HydrationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Davinci canonical corpus preflight failed:")?;
        writeln!(f, "  root: {}", self.root)?;
        writeln!(f, "  closure_evidence: true")?;
        writeln!(f, "  indexed gitlinks: {}", self.indexed)?;
        writeln!(f, "  clean submodules: {}", self.clean)?;
        write_group(f, "missing submodules", &self.missing)?;
        write_group(f, "drifted submodules", &self.drifted)?;
        write_group(f, "conflicted submodules", &self.conflicted)?;
        write_group(f, "invalid submodules", &self.invalid)?;
        write_group(f, "inventory mismatches", &self.inventory_mismatch)?;
        writeln!(f, "  hydrate with:")?;
        writeln!(f, "    {HYDRATE_COMMAND}")?;
        write!(
            f,
            "  partial fixture trees are refused before collecting .vue files"
        )
    }
}

fn write_group(f: &mut fmt::Formatter<'_>, label: &str, rows: &[String]) -> fmt::Result {
    if rows.is_empty() {
        return Ok(());
    }
    writeln!(f, "  {label}: {}", rows.len())?;
    for row in rows.iter().take(5) {
        writeln!(f, "    {row}")?;
    }
    if rows.len() > 5 {
        writeln!(f, "    ... and {} more", rows.len() - 5)?;
    }
    Ok(())
}

pub fn corpus_root_from_env(
    package_manifest_dir: impl AsRef<Path>,
) -> Result<Option<CorpusRoot>, CorpusPreflightError> {
    let Some(value) = env::var_os(DIFFERENTIAL_CORPUS_ENV) else {
        return Ok(None);
    };
    prepare_corpus_root(value, package_manifest_dir).map(Some)
}

pub fn prepare_corpus_root(
    raw_root: impl Into<PathBuf>,
    package_manifest_dir: impl AsRef<Path>,
) -> Result<CorpusRoot, CorpusPreflightError> {
    let workspace = workspace_root(package_manifest_dir.as_ref());
    let root = resolve_root(raw_root.into(), &workspace);
    let canonical = normalize_path(&workspace.join(CANONICAL_CORPUS_ROOT));
    let scope = if normalize_path(&root) == canonical {
        assert_canonical_hydrated(&workspace)?;
        CorpusScope::ClosureEvidence
    } else {
        CorpusScope::SmokeShard
    };
    Ok(CorpusRoot { path: root, scope })
}

fn workspace_root(package_manifest_dir: &Path) -> PathBuf {
    package_manifest_dir
        .ancestors()
        .nth(2)
        .unwrap_or(package_manifest_dir)
        .to_path_buf()
}

fn resolve_root(raw_root: PathBuf, workspace: &Path) -> PathBuf {
    let path = if raw_root.is_absolute() {
        raw_root
    } else {
        workspace.join(raw_root)
    };
    normalize_path(&path)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                out.push(component.as_os_str());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;

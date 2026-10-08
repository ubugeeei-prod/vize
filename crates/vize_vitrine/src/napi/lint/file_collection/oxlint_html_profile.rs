//! Private original HTML selection, not a wrapper or installed acceptance path.
//!
//! Ordering/defaults follow OXC's MIT-licensed `apps/oxlint/src/{lint,walk}.rs`
//! and `crates/oxc_config/src/walk.rs` at c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd
//! (1.78.0) and 2ae2939bb2fd98796393658b21556b2a2467e047 (1.86.0).
//! Upstream copyright and permission notice: `oxlint_html_profile/OXC_LICENSE`.
//! Matching is performed by the same locked ignore 0.4.33, not copied glob logic.
//! Sorted paths are this internal contract, not Oxlint's parallel report order.

use std::path::{Path, PathBuf};
use vize_l0::{String, ToCompactString};

mod envelope;
mod policy;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub(super) enum HostProfile {
    Oxlint178,
    Oxlint186,
}

#[derive(Debug)]
pub(super) struct Request<'a> {
    pub cwd: &'a Path,
    pub literal_target: &'a str,
    pub root_json: &'a Path,
    pub host: HostProfile,
    pub no_ignore: bool,
    /// Original CLI strings in their original order; no glob rewriting here.
    pub cli_ignore_patterns: &'a [String],
    /// A single filename, default `.eslintignore`, as in add_custom_ignore_filename.
    pub custom_ignore_filename: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub(super) enum Origin {
    ExplicitFile,
    DirectoryDiscovery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub(super) enum RootDecision {
    Eligible,
    CliOrCustomExcluded,
    VcsExcluded,
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub(super) struct Original {
    pub path: PathBuf,
    pub cwd_relative: PathBuf,
    pub origin: Origin,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(test, derive(serde::Serialize))]
pub(super) enum SourceRole {
    RootJson,
    GitIgnore,
    GitInfoExclude,
    CustomIgnore,
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub(super) struct SourceCustody {
    pub path: PathBuf,
    pub role: SourceRole,
    /// None records an absent source, rather than silently dropping authority.
    pub bytes: Option<Vec<u8>>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(serde::Serialize))]
pub(super) struct Selection {
    pub host: HostProfile,
    pub cwd: PathBuf,
    pub literal_target: String,
    pub target: PathBuf,
    pub repository: PathBuf,
    pub root_json: PathBuf,
    pub no_ignore: bool,
    pub cli_ignore_patterns: Vec<String>,
    pub custom_ignore_filename: String,
    pub root_decision: RootDecision,
    pub originals: Vec<Original>,
    pub sources: Vec<SourceCustody>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub(super) enum RefusalKind {
    NonPosix,
    InvalidCwd,
    NonLiteralTarget,
    OutsideCwd,
    MissingTarget,
    UnsupportedFileType,
    Symlink,
    MissingGitBoundary,
    LinkedGitMetadata,
    JujutsuMetadata,
    NestedVcs,
    CustomIgnoreFilename,
    ExternalCustomIgnore,
    RootJsonLocation,
    RootJsonSyntax,
    ConfigInheritance,
    ConfigOverrides,
    NestedConfig,
    SuppressionState,
    IgnoreSyntax,
    Io,
    Internal,
}

#[derive(Debug)]
#[cfg_attr(test, derive(serde::Serialize))]
pub(super) struct Refusal {
    pub kind: RefusalKind,
    pub path: PathBuf,
    pub details: String,
    /// Preserve bytes of a malformed original authority, when readable.
    pub original_bytes: Option<Vec<u8>>,
}

impl Refusal {
    fn new(kind: RefusalKind, path: &Path, details: impl Into<String>) -> Self {
        Self {
            kind,
            path: path.to_path_buf(),
            details: details.into(),
            original_bytes: None,
        }
    }

    fn io(path: &Path, error: impl std::fmt::Display) -> Self {
        Self::new(RefusalKind::Io, path, error.to_compact_string())
    }
}

/// One existing literal target, explicit regular JSON in cwd, regular Git marker.
/// No config discovery, inherited/override configs, symlinks, linked/JJ/nested
/// repositories, outside-cwd roots, non-POSIX/non-UTF8 paths, or wildcard targets.
/// This returns HTML custody only; it does not produce any lint diagnostic packet.
pub(super) fn select(request: Request<'_>) -> Result<Selection, Refusal> {
    let admitted = envelope::admit(&request)?;
    let mut selection = Selection {
        host: request.host,
        cwd: request.cwd.to_path_buf(),
        literal_target: request.literal_target.to_compact_string(),
        target: admitted.target.clone(),
        repository: admitted.repository,
        root_json: request.root_json.to_path_buf(),
        no_ignore: request.no_ignore,
        cli_ignore_patterns: request.cli_ignore_patterns.to_vec(),
        custom_ignore_filename: request.custom_ignore_filename.to_compact_string(),
        root_decision: RootDecision::Eligible,
        originals: Vec::new(),
        sources: Vec::new(),
    };
    let config = policy::root_config(&request, &mut selection)?;
    envelope::record_ancestors(&request, &admitted.target, &mut selection)?;
    let overrides = policy::cli_overrides(&request)?;
    if !admitted.is_dir
        && policy::explicit_custom_excluded(&request, &selection.target, &overrides)?
    {
        selection.root_decision = RootDecision::CliOrCustomExcluded;
    } else if (admitted.is_dir || request.host == HostProfile::Oxlint178)
        && policy::vcs_root_excluded(&selection)?
    {
        selection.root_decision = RootDecision::VcsExcluded;
    } else {
        policy::walk_originals(
            &request,
            &config,
            overrides,
            admitted.is_dir,
            &mut selection,
        )?;
    }
    selection
        .originals
        .sort_unstable_by(|left, right| left.path.cmp(&right.path));
    selection
        .sources
        .sort_unstable_by(|left, right| (&left.path, left.role).cmp(&(&right.path, right.role)));
    Ok(selection)
}

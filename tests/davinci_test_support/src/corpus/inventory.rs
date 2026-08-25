//! Pure submodule-inventory parsing and reconciliation.
//!
//! The canonical differential corpus is a tree of mode-160000 gitlinks; a
//! sweep over a partially hydrated tree must fail closed instead of being
//! reported as closure evidence. The parsers here consume the raw output of
//! `git ls-files --stage -z` and `git submodule status` so every rejection
//! class can be pinned synthetically without a real repository.

use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;
use vize_s0::CompactString;

/// A deterministic reason the corpus inventory is not closure evidence.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum InventoryError {
    /// No gitlinks are indexed at all.
    #[error(
        "differential corpus inventory is empty: no mode-160000 gitlinks are indexed under the fixture root"
    )]
    Empty,
    /// A `git ls-files --stage -z` record did not parse.
    #[error("differential corpus inventory is invalid: unparseable git index record `{record}`")]
    InvalidIndexRecord { record: CompactString },
    /// A `git submodule status` line did not parse.
    #[error("differential corpus inventory is invalid: unparseable submodule status line `{line}`")]
    InvalidStatusLine { line: CompactString },
    /// A `git submodule status` line carries a marker outside ` `, `-`, `+`, `U`.
    #[error(
        "differential corpus inventory is invalid: unknown submodule status marker `{marker}` on `{path}`"
    )]
    UnknownMarker { marker: char, path: CompactString },
    /// Indexed submodules whose worktrees are not hydrated.
    #[error(
        "differential corpus is not closure evidence: {missing} of {indexed} indexed fixture submodules are missing (unhydrated); first missing `{first}`; hydrate with `git submodule update --init --checkout`"
    )]
    Missing {
        missing: usize,
        indexed: usize,
        first: CompactString,
    },
    /// Hydrated submodules whose checked-out commit differs from the gitlink.
    #[error(
        "differential corpus is not closure evidence: {drifted} of {indexed} fixture submodules have drifted from their indexed gitlink; first drifted `{first}`"
    )]
    Drifted {
        drifted: usize,
        indexed: usize,
        first: CompactString,
    },
    /// Submodules that are merge-conflicted in the index or worktree.
    #[error(
        "differential corpus is not closure evidence: {conflicted} of {indexed} fixture submodules are merge-conflicted; first conflicted `{first}`"
    )]
    Conflicted {
        conflicted: usize,
        indexed: usize,
        first: CompactString,
    },
    /// The indexed gitlink set and the submodule status set name different paths.
    #[error(
        "differential corpus inventory mismatch: {index_only} path(s) only in the git index, {status_only} path(s) only in submodule status; first index-only `{first_index_only}`, first status-only `{first_status_only}`"
    )]
    SetMismatch {
        index_only: usize,
        status_only: usize,
        first_index_only: CompactString,
        first_status_only: CompactString,
    },
}

/// Indexed mode-160000 gitlinks split by index stage.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct IndexedGitlinks {
    /// Stage-0 gitlinks.
    pub clean: BTreeSet<CompactString>,
    /// Gitlinks carrying conflict stages (1/2/3).
    pub conflicted: BTreeSet<CompactString>,
}

impl IndexedGitlinks {
    /// Every indexed gitlink path, conflicted or not.
    #[must_use]
    pub fn total(&self) -> usize {
        let overlap = self.clean.intersection(&self.conflicted).count();
        self.clean.len() + self.conflicted.len() - overlap
    }
}

/// The worktree state of one submodule as reported by `git submodule status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmoduleState {
    /// Checked out at the indexed commit.
    Clean,
    /// Not initialized / not hydrated (`-`).
    Missing,
    /// Checked out at a different commit than the index records (`+`).
    Drifted,
    /// Merge conflicts (`U`).
    Conflicted,
}

/// Parse NUL-separated `git ls-files --stage -z` output, keeping only
/// mode-160000 gitlink entries.
pub fn parse_indexed_gitlinks(stage_output: &str) -> Result<IndexedGitlinks, InventoryError> {
    let mut indexed = IndexedGitlinks::default();
    for record in stage_output.split('\0') {
        if record.is_empty() {
            continue;
        }
        let invalid = || InventoryError::InvalidIndexRecord {
            record: record.into(),
        };
        let (meta, path) = record.split_once('\t').ok_or_else(invalid)?;
        let mut fields = meta.split_ascii_whitespace();
        let (Some(mode), Some(_object), Some(stage), None) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            return Err(invalid());
        };
        if path.is_empty() || !matches!(stage, "0" | "1" | "2" | "3") {
            return Err(invalid());
        }
        if mode != "160000" {
            continue;
        }
        if stage == "0" {
            indexed.clean.insert(path.into());
        } else {
            indexed.conflicted.insert(path.into());
        }
    }
    Ok(indexed)
}

/// Parse `git submodule status` output into per-path worktree states.
pub fn parse_submodule_status(
    status_output: &str,
) -> Result<BTreeMap<CompactString, SubmoduleState>, InventoryError> {
    let mut states = BTreeMap::new();
    for line in status_output.lines() {
        if line.is_empty() {
            continue;
        }
        let invalid = || InventoryError::InvalidStatusLine { line: line.into() };
        let mut chars = line.chars();
        let marker = chars.next().ok_or_else(invalid)?;
        let rest = chars.as_str();
        let (object, spaced_path) = rest.split_at_checked(40).ok_or_else(invalid)?;
        if object.chars().any(|ch| !ch.is_ascii_hexdigit()) {
            return Err(invalid());
        }
        let path = spaced_path.strip_prefix(' ').ok_or_else(invalid)?;
        if path.is_empty() {
            return Err(invalid());
        }
        let state = match marker {
            ' ' => SubmoduleState::Clean,
            '-' => SubmoduleState::Missing,
            '+' => SubmoduleState::Drifted,
            'U' => SubmoduleState::Conflicted,
            other => {
                return Err(InventoryError::UnknownMarker {
                    marker: other,
                    path: strip_describe(path).into(),
                });
            }
        };
        // Hydrated entries append ` (<describe>)`; missing ones never do.
        let path = if marker == '-' {
            path
        } else {
            strip_describe(path)
        };
        if states.insert(path.into(), state).is_some() {
            return Err(invalid());
        }
    }
    Ok(states)
}

fn strip_describe(path: &str) -> &str {
    if path.ends_with(')')
        && let Some(cut) = path.rfind(" (")
        && cut > 0
    {
        return &path[..cut];
    }
    path
}

/// Reconcile the indexed gitlink inventory against submodule worktree states.
///
/// Returns the reconciled submodule count only when every indexed gitlink is
/// hydrated at its indexed commit; every other shape yields a deterministic
/// [`InventoryError`].
pub fn reconcile(
    indexed: &IndexedGitlinks,
    states: &BTreeMap<CompactString, SubmoduleState>,
) -> Result<usize, InventoryError> {
    if indexed.total() == 0 && states.is_empty() {
        return Err(InventoryError::Empty);
    }

    let mut conflicted: BTreeSet<&CompactString> = indexed.conflicted.iter().collect();
    conflicted.extend(paths_in_state(states, SubmoduleState::Conflicted));
    if let Some(first) = conflicted.first() {
        return Err(InventoryError::Conflicted {
            conflicted: conflicted.len(),
            indexed: indexed.total(),
            first: (*first).clone(),
        });
    }

    let index_paths: BTreeSet<&CompactString> =
        indexed.clean.iter().chain(&indexed.conflicted).collect();
    let status_paths: BTreeSet<&CompactString> = states.keys().collect();
    let index_only: Vec<&CompactString> = index_paths.difference(&status_paths).copied().collect();
    let status_only: Vec<&CompactString> = status_paths.difference(&index_paths).copied().collect();
    if !index_only.is_empty() || !status_only.is_empty() {
        let placeholder = || CompactString::const_new("<none>");
        return Err(InventoryError::SetMismatch {
            index_only: index_only.len(),
            status_only: status_only.len(),
            first_index_only: index_only
                .first()
                .map_or_else(placeholder, |p| (*p).clone()),
            first_status_only: status_only
                .first()
                .map_or_else(placeholder, |p| (*p).clone()),
        });
    }

    let drifted: Vec<&CompactString> = paths_in_state(states, SubmoduleState::Drifted).collect();
    if let Some(first) = drifted.first() {
        return Err(InventoryError::Drifted {
            drifted: drifted.len(),
            indexed: indexed.total(),
            first: (*first).clone(),
        });
    }

    let missing: Vec<&CompactString> = paths_in_state(states, SubmoduleState::Missing).collect();
    if let Some(first) = missing.first() {
        return Err(InventoryError::Missing {
            missing: missing.len(),
            indexed: indexed.total(),
            first: (*first).clone(),
        });
    }

    Ok(indexed.total())
}

fn paths_in_state(
    states: &BTreeMap<CompactString, SubmoduleState>,
    state: SubmoduleState,
) -> impl Iterator<Item = &CompactString> {
    states
        .iter()
        .filter(move |(_, entry)| **entry == state)
        .map(|(path, _)| path)
}

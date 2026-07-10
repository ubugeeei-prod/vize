//! Stable source identities, revisions, and embedded-source provenance.

use std::fmt;
use vize_carton::FxHashMap;

use crate::{Shared, SourceError};

/// Stable identity of one source in a compilation.
#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceId(u64);

impl SourceId {
    /// Return the numeric identity for diagnostics or persistence adapters.
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "source#{}", self.0)
    }
}

/// Monotonic revision of a stable [`SourceId`].
#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceRevision(u64);

impl SourceRevision {
    /// Revision assigned when a source first enters the store.
    pub const INITIAL: Self = Self(1);

    /// Return the numeric revision.
    pub const fn get(self) -> u64 {
        self.0
    }

    fn next(self, source: SourceId) -> Result<Self, SourceError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(SourceError::RevisionOverflow(source))
    }
}

impl fmt::Display for SourceRevision {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "r{}", self.0)
    }
}

/// Byte range in a parent source that produced an embedded source.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct SourceRange {
    pub start: usize,
    pub end: usize,
}

impl SourceRange {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// How a source entered the compilation.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SourceProvenance {
    /// An independently supplied root document.
    Root,
    /// A virtual or embedded document derived from a parent snapshot.
    Embedded {
        parent: SourceId,
        parent_revision: SourceRevision,
        range: SourceRange,
    },
}

/// Immutable, cheaply cloned source snapshot passed to providers.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SourceSnapshot {
    id: SourceId,
    revision: SourceRevision,
    name: Shared<str>,
    text: Shared<str>,
    provenance: SourceProvenance,
}

impl SourceSnapshot {
    pub const fn id(&self) -> SourceId {
        self.id
    }
    pub const fn revision(&self) -> SourceRevision {
        self.revision
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Clone source storage for a borrowed or lazy [`crate::ProductView`].
    pub fn shared_text(&self) -> Shared<str> {
        Shared::clone(&self.text)
    }

    pub const fn provenance(&self) -> &SourceProvenance {
        &self.provenance
    }
}

/// Revision change caused by one source update.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SourceRevisionChange {
    pub source: SourceId,
    pub previous: SourceRevision,
    pub current: SourceRevision,
}

/// Source snapshots owned by one compilation.
#[derive(Debug, Clone)]
pub struct SourceStore {
    next_id: u64,
    entries: FxHashMap<SourceId, SourceSnapshot>,
}

impl Default for SourceStore {
    fn default() -> Self {
        Self {
            next_id: 1,
            entries: FxHashMap::default(),
        }
    }
}

impl SourceStore {
    pub fn add(
        &mut self,
        name: impl Into<Shared<str>>,
        text: impl Into<Shared<str>>,
    ) -> Result<SourceId, SourceError> {
        self.insert(name.into(), text.into(), SourceProvenance::Root)
    }

    pub fn add_embedded(
        &mut self,
        parent: SourceId,
        range: SourceRange,
        name: impl Into<Shared<str>>,
        text: impl Into<Shared<str>>,
    ) -> Result<SourceId, SourceError> {
        if self.stale_edge(parent).is_some() {
            return Err(SourceError::StaleParent(parent));
        }
        let parent_snapshot = self
            .entries
            .get(&parent)
            .ok_or(SourceError::SourceNotFound(parent))?;
        validate_range(parent_snapshot, range)?;
        let provenance = SourceProvenance::Embedded {
            parent,
            parent_revision: parent_snapshot.revision,
            range,
        };
        self.insert(name.into(), text.into(), provenance)
    }

    pub fn get(&self, source: SourceId) -> Option<&SourceSnapshot> {
        self.entries.get(&source)
    }

    /// Iterate over every immutable source snapshot in this store.
    ///
    /// Iteration order is unspecified. Callers that expose ordered output
    /// should sort by [`SourceId`].
    pub fn iter(&self) -> impl Iterator<Item = &SourceSnapshot> {
        self.entries.values()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn insert(
        &mut self,
        name: Shared<str>,
        text: Shared<str>,
        provenance: SourceProvenance,
    ) -> Result<SourceId, SourceError> {
        let id = SourceId(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(SourceError::SourceIdExhausted)?;
        self.entries.insert(
            id,
            SourceSnapshot {
                id,
                revision: SourceRevision::INITIAL,
                name,
                text,
                provenance,
            },
        );
        Ok(id)
    }

    pub(crate) fn update(
        &mut self,
        source: SourceId,
        text: Shared<str>,
        range: Option<SourceRange>,
    ) -> Result<SourceMutation, SourceError> {
        let existing = self
            .entries
            .get(&source)
            .cloned()
            .ok_or(SourceError::SourceNotFound(source))?;
        let refreshed = self.refreshed_provenance(source, &existing.provenance, range)?;
        let affected = self.descendants_including(source);
        let mut changes = Vec::with_capacity(affected.len());
        for id in affected {
            let entry = self
                .entries
                .get_mut(&id)
                .ok_or(SourceError::SourceNotFound(id))?;
            let previous = entry.revision;
            entry.revision = previous.next(id)?;
            if id == source {
                entry.text = Shared::clone(&text);
                entry.provenance = refreshed.clone();
            }
            changes.push(SourceRevisionChange {
                source: id,
                previous,
                current: entry.revision,
            });
        }
        Ok(SourceMutation { changes })
    }

    fn refreshed_provenance(
        &self,
        source: SourceId,
        provenance: &SourceProvenance,
        new_range: Option<SourceRange>,
    ) -> Result<SourceProvenance, SourceError> {
        let SourceProvenance::Embedded { parent, range, .. } = provenance else {
            return if new_range.is_some() {
                Err(SourceError::NotEmbedded(source))
            } else {
                Ok(SourceProvenance::Root)
            };
        };
        if self.stale_edge(*parent).is_some() {
            return Err(SourceError::StaleParent(*parent));
        }
        let parent_snapshot = self
            .entries
            .get(parent)
            .ok_or(SourceError::SourceNotFound(*parent))?;
        let range = new_range.unwrap_or(*range);
        validate_range(parent_snapshot, range)?;
        Ok(SourceProvenance::Embedded {
            parent: *parent,
            parent_revision: parent_snapshot.revision,
            range,
        })
    }

    fn descendants_including(&self, source: SourceId) -> Vec<SourceId> {
        let mut affected = vec![source];
        let mut index = 0;
        while index < affected.len() {
            let parent = affected[index];
            let mut children: Vec<_> = self
                .entries
                .values()
                .filter_map(|entry| match entry.provenance {
                    SourceProvenance::Embedded {
                        parent: candidate, ..
                    } if candidate == parent => Some(entry.id),
                    _ => None,
                })
                .collect();
            children.sort_unstable();
            affected.extend(children);
            index += 1;
        }
        affected
    }

    pub(crate) fn stale_edge(&self, source: SourceId) -> Option<StaleProvenance> {
        let mut current = source;
        loop {
            let entry = self.entries.get(&current)?;
            let SourceProvenance::Embedded {
                parent,
                parent_revision,
                ..
            } = entry.provenance
            else {
                return None;
            };
            let parent_snapshot = self.entries.get(&parent)?;
            if parent_revision != parent_snapshot.revision {
                return Some(StaleProvenance {
                    source: current,
                    parent,
                    recorded: parent_revision,
                    current: parent_snapshot.revision,
                });
            }
            current = parent;
        }
    }
}

fn validate_range(parent: &SourceSnapshot, range: SourceRange) -> Result<(), SourceError> {
    if range.start > range.end || range.end > parent.text.len() {
        return Err(SourceError::InvalidEmbeddedRange {
            parent: parent.id,
            range,
            parent_len: parent.text.len(),
        });
    }
    if !parent.text.is_char_boundary(range.start) || !parent.text.is_char_boundary(range.end) {
        return Err(SourceError::RangeNotCharBoundary {
            parent: parent.id,
            range,
        });
    }
    Ok(())
}

pub(crate) struct SourceMutation {
    pub changes: Vec<SourceRevisionChange>,
}

pub(crate) struct StaleProvenance {
    pub source: SourceId,
    pub parent: SourceId,
    pub recorded: SourceRevision,
    pub current: SourceRevision,
}

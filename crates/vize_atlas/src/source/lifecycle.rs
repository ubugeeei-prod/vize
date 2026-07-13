//! Source identity lifecycle operations.

use crate::{Shared, SourceError};

use super::{SourceId, SourceMutation, SourceProvenance, SourceRevisionChange, SourceStore};

impl SourceStore {
    pub(crate) fn rename(
        &mut self,
        source: SourceId,
        name: Shared<str>,
    ) -> Result<SourceMutation, SourceError> {
        if !self.entries.contains_key(&source) {
            return Err(SourceError::SourceNotFound(source));
        }
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
                entry.name = Shared::clone(&name);
            }
            changes.push(SourceRevisionChange {
                source: id,
                previous,
                current: entry.revision,
            });
        }
        Ok(SourceMutation { changes })
    }

    pub(crate) fn remove(&mut self, source: SourceId) -> Result<Vec<SourceId>, SourceError> {
        if !self.entries.contains_key(&source) {
            return Err(SourceError::SourceNotFound(source));
        }
        let removed = self.descendants_including(source);
        for id in removed.iter().rev() {
            self.entries.remove(id);
        }
        Ok(removed)
    }

    pub(super) fn descendants_including(&self, source: SourceId) -> Vec<SourceId> {
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
}

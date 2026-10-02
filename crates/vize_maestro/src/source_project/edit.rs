//! Host binding for the actual source-owned native edit model.

use vize_l0::{SourceFrameError, SourceRoot, String};
use vize_l1::edit::{EditError, EditSet, VersionedSource};

use crate::document::DocumentStore;

use super::{SnapshotKey, SnapshotRefusal, SourceSnapshot};

/// The host or original source-owned edit model refused to produce new text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceEditRefusal {
    Host(SnapshotRefusal),
    Frame(SourceFrameError),
    Edit(EditError),
}

impl SourceSnapshot {
    /// Borrow the once-captured physical buffer, not an equal-text reread.
    /// This frame is source/edit evidence only, never native File admission.
    pub fn versioned_source(
        &self,
    ) -> Result<VersionedSource<'_, SnapshotKey<'_>>, SourceFrameError> {
        Ok(VersionedSource::new(
            self.key(),
            self.version(),
            SourceRoot::new(self.source())?,
        ))
    }

    /// Validate against the actual current host under its read guard, then
    /// produce a new authored string. This does not mutate DocumentStore or
    /// claim an atomic editor application. Publish via SourceQueryResult to
    /// recheck cancellation and the host after intervening work.
    pub fn apply_edits(
        &self,
        documents: &DocumentStore,
        edits: EditSet<'_, '_, '_, SnapshotKey<'_>>,
    ) -> Result<String, SourceEditRefusal> {
        self.with_current(documents, |document| {
            let current = VersionedSource::new(
                SnapshotKey {
                    uri: &document.uri,
                    revision: document.revision(),
                },
                document.version,
                SourceRoot::new(self.source()).map_err(SourceEditRefusal::Frame)?,
            );
            edits.apply(current).map_err(SourceEditRefusal::Edit)
        })
        .map_err(SourceEditRefusal::Host)?
    }
}

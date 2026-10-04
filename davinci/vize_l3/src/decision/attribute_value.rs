//! Original value rows consumed at the sole actual Element attribute visit.

use super::DecisionBuildError;
use vize_l0::id::NodeId;
use vize_l2::{
    artifact::Artifact,
    file::{FileArtifact, FileAttributeValue, NativeFileAttributeValueState},
    op::ElementOp,
};

/// Complete original value custody from the actual selected File's sole walk.
/// This keeps the whole normal observations and maps borrowed from that File;
/// it is not a new attribute table, target-name policy or decoded-source factory.
/// ```compile_fail
/// use vize_l2::file::FileArtifact;
/// use vize_l3::decision::OriginalAttributeFacts;
/// fn forge<'f,'a>(file: &'f FileArtifact<'a>) {
///     let _ = OriginalAttributeFacts { file, consumed: 0 };
/// }
/// ```
pub struct OriginalAttributeFacts<'owner, 'arena> {
    file: &'owner FileArtifact<'arena>,
    consumed: usize,
}
impl<'owner, 'arena> OriginalAttributeFacts<'owner, 'arena> {
    #[must_use]
    pub const fn file(&self) -> &'owner FileArtifact<'arena> {
        self.file
    }
    #[must_use]
    pub const fn len(&self) -> usize {
        self.consumed
    }
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.consumed == 0
    }
    /// Readback still requires the same actual Element and canonical slot.
    #[must_use]
    pub fn value(
        &self,
        index: usize,
        element: &'owner ElementOp<'arena>,
        slot: usize,
    ) -> Option<FileAttributeValue<'owner, 'arena>> {
        if index >= self.consumed {
            return None;
        }
        self.file.native_attribute_value_for(index, element, slot)
    }
}

pub(super) struct OriginalAttributeCursor<'owner, 'arena> {
    file: &'owner FileArtifact<'arena>,
    next: usize,
    failure: Option<DecisionBuildError>,
}
impl<'owner, 'arena> OriginalAttributeCursor<'owner, 'arena> {
    pub(super) fn new(
        file: &'owner FileArtifact<'arena>,
        artifact: &'owner Artifact<'arena>,
    ) -> Result<Self, DecisionBuildError> {
        if !file.is_complete() {
            return Err(DecisionBuildError::IncompleteFile);
        }
        if !core::ptr::eq(file.artifact(), artifact) {
            return Err(DecisionBuildError::OriginalAttributeValue {
                node: None,
                slot: 0,
            });
        }
        Ok(Self {
            file,
            next: 0,
            failure: None,
        })
    }

    // Called only from the current target's existing attribute loop. A bare
    // attribute has no preparation row and retains its separate target policy.
    pub(super) fn observe(
        &mut self,
        node: NodeId,
        element: &'owner ElementOp<'arena>,
        slot: usize,
    ) {
        if self.failure.is_some() {
            return;
        }
        let valid = (|| {
            let attribute = element.attributes.get(slot)?;
            if attribute.value.is_none() {
                return Some(true);
            }
            let joined = self
                .file
                .native_attribute_value_for(self.next, element, slot)?;
            let record = self.file.native_attribute_values().get(self.next)?;
            if record.state() != (NativeFileAttributeValueState::Attached { node, slot }) {
                return None;
            }
            let value = joined.observation()?;
            let source = value.source();
            if !core::ptr::eq(source.authored_root(), self.file.source())
                || source.span() != value.value_span()
                || attribute.span.start != value.name_span().start
                || attribute.span.end != value.full_value_span().end
                || !core::ptr::eq(
                    value.raw_value(),
                    self.file
                        .source()
                        .get(value.value_span().start as usize..value.value_span().end as usize)?,
                )
            {
                return None;
            }
            // The sealed whole observation retains its complete unchanged
            // decode map. No map scan, source decode or expression parse occurs.
            self.next = self.next.checked_add(1)?;
            Some(true)
        })();
        if valid != Some(true) {
            self.failure = Some(DecisionBuildError::OriginalAttributeValue {
                node: Some(node),
                slot,
            });
        }
    }

    pub(super) fn finish(
        self,
    ) -> Result<OriginalAttributeFacts<'owner, 'arena>, DecisionBuildError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if self.next != self.file.native_attribute_values().len() {
            return Err(DecisionBuildError::OriginalAttributeValue {
                node: None,
                slot: self.next,
            });
        }
        Ok(OriginalAttributeFacts {
            file: self.file,
            consumed: self.next,
        })
    }
}

pub(super) fn observe<'owner, 'arena>(
    cursor: &mut Option<OriginalAttributeCursor<'owner, 'arena>>,
    node: NodeId,
    element: &'owner ElementOp<'arena>,
    slot: usize,
) {
    if let Some(cursor) = cursor {
        cursor.observe(node, element, slot);
    }
}

#[cfg(test)]
mod tests;

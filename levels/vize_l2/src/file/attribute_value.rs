//! Whole original value preparation sealed only to its actual canonical slot.

use super::{FileArtifact, RejectedFile};
use crate::lang::js::NativeTemplateIssueKind;
use crate::op::{Attribute, ElementOp};
use core::ptr::NonNull;
use vize_l0::id::NodeId;
use vize_l1::markup::{NativeAttributeValue, NativeAttributeValueFailure};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeFileAttributeValueState {
    Pending,
    Attached { node: NodeId, slot: usize },
    Refused(NativeTemplateIssueKind),
}

/// Normally owned observations survive failed policy, headers and unwinds.
/// No caller tuple, decoded source or neutral Element can mint an attachment.
/// ```compile_fail
/// use vize_l2::file::NativeFileAttributeValue;
/// fn clone<T: Clone>() {}
/// clone::<NativeFileAttributeValue<'static>>();
/// ```
pub struct NativeFileAttributeValue<'a> {
    pub(crate) observation: Result<NativeAttributeValue<'a>, NativeAttributeValueFailure<'a>>,
    pub(crate) state: NativeFileAttributeValueState,
    pub(crate) slot: usize,
    pub(crate) name: &'a str,
    pub(crate) element: Option<NonNull<ElementOp<'a>>>,
    pub(crate) attribute: Option<NonNull<Attribute<'a>>>,
}
impl<'a> NativeFileAttributeValue<'a> {
    #[must_use]
    pub fn observation(&self) -> Option<&NativeAttributeValue<'a>> {
        self.observation.as_ref().ok()
    }
    #[must_use]
    pub fn failure(&self) -> Option<&NativeAttributeValueFailure<'a>> {
        self.observation.as_ref().err()
    }
    #[must_use]
    pub const fn state(&self) -> NativeFileAttributeValueState {
        self.state
    }
}

/// A short actual File + Element allocation + canonical attribute slot join.
/// Numeric row lookup or equal name/value/spans cannot construct this view.
/// ```compile_fail
/// use vize_l2::file::{FileArtifact, FileAttributeValue};
/// use vize_l2::op::ElementOp;
/// fn forge<'f,'a>(file: &'f FileArtifact<'a>, element: &'f ElementOp<'a>) {
///     let _ = FileAttributeValue { file, element, index: 0, slot: 0 };
/// }
/// ```
#[derive(Clone, Copy)]
pub struct FileAttributeValue<'f, 'a> {
    file: &'f FileArtifact<'a>,
    element: &'f ElementOp<'a>,
    index: usize,
    slot: usize,
}
impl<'a> FileArtifact<'a> {
    #[must_use]
    pub fn native_attribute_values(&self) -> &[NativeFileAttributeValue<'a>] {
        &self.facts.native_attribute_values
    }
    #[must_use]
    pub fn native_attribute_value_for<'f>(
        &'f self,
        index: usize,
        element: &'f ElementOp<'a>,
        slot: usize,
    ) -> Option<FileAttributeValue<'f, 'a>> {
        let record = self.facts.native_attribute_values.get(index)?;
        let attribute = element.attributes.get(slot)?;
        let observation = record.observation()?;
        if !matches!(record.state, NativeFileAttributeValueState::Attached { slot: actual, .. } if actual == slot)
            || record.element != Some(NonNull::from(element))
            || record.attribute != Some(NonNull::from(attribute))
            || !core::ptr::eq(record.name, attribute.name)
            || !attribute
                .value
                .is_some_and(|value| core::ptr::eq(value, observation.source().text()))
        {
            return None;
        }
        Some(FileAttributeValue {
            file: self,
            element,
            index,
            slot,
        })
    }
}
impl<'a> RejectedFile<'a> {
    #[must_use]
    pub fn native_attribute_values(&self) -> &[NativeFileAttributeValue<'a>] {
        &self.facts.native_attribute_values
    }
}
impl<'f, 'a> FileAttributeValue<'f, 'a> {
    #[must_use]
    pub const fn file(self) -> &'f FileArtifact<'a> {
        self.file
    }
    #[must_use]
    pub const fn element(self) -> &'f ElementOp<'a> {
        self.element
    }
    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }
    #[must_use]
    pub const fn slot(self) -> usize {
        self.slot
    }
    #[must_use]
    pub fn observation(self) -> Option<&'f NativeAttributeValue<'a>> {
        self.file
            .facts
            .native_attribute_values
            .get(self.index)?
            .observation()
    }
    #[must_use]
    pub fn attribute(self) -> Option<&'f Attribute<'a>> {
        self.element.attributes.get(self.slot)
    }
}

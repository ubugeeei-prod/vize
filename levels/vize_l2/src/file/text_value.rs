//! Normally owned original preparation, joined only to its real Text mint.

use super::{FileArtifact, RejectedFile};
use crate::{lang::js::NativeTemplateIssueKind, op::TextOp};
use core::ptr::NonNull;
use vize_l0::id::NodeId;
use vize_l1::markup::{NativeTextValue, NativeTextValueFailure};

/// Bounded source policy, independent of target or whitespace condensation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTextValuePolicyError {
    RootExtent,
    Leading,
    Empty,
    RawWhitespace,
    DecodedWhitespace,
    Control,
    ScriptOrStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeFileTextValueState {
    Pending,
    Attached(NodeId),
    Refused(NativeTemplateIssueKind),
}

/// Whole results remain normally owned on failure and caught unwind.
/// This row alone grants no File completion or native output profile.
/// ```compile_fail
/// use vize_l2::file::NativeFileTextValue;
/// fn clone<T: Clone>() {}
/// clone::<NativeFileTextValue<'static>>();
/// ```
/// ```compile_fail
/// use vize_l2::file::{NativeFileTextValue, NativeFileTextValueState};
/// let _ = NativeFileTextValue {
///     observation: panic!(), state: NativeFileTextValueState::Pending, text: None,
/// };
/// ```
pub struct NativeFileTextValue<'a> {
    pub(crate) observation: Result<NativeTextValue<'a>, NativeTextValueFailure<'a>>,
    pub(crate) state: NativeFileTextValueState,
    pub(crate) text: Option<NonNull<TextOp<'a>>>,
}
impl<'a> NativeFileTextValue<'a> {
    #[must_use]
    pub fn observation(&self) -> Option<&NativeTextValue<'a>> {
        self.observation.as_ref().ok()
    }
    #[must_use]
    pub fn failure(&self) -> Option<&NativeTextValueFailure<'a>> {
        self.observation.as_ref().err()
    }
    #[must_use]
    pub const fn state(&self) -> NativeFileTextValueState {
        self.state
    }
}

/// Same actual File, NodeId and arena Text allocation, with whole preparation.
/// Equal bytes, a numeric node or a copied Text cannot construct this view.
/// ```compile_fail
/// use vize_l2::{file::{FileArtifact, FileTextValue}, op::TextOp};
/// use vize_l0::id::NodeId;
/// fn forge<'f, 'a>(file: &'f FileArtifact<'a>, text: &'f TextOp<'a>) {
///     let _ = FileTextValue { file, text, node: NodeId::FIRST };
/// }
/// ```
#[derive(Clone, Copy)]
pub struct FileTextValue<'f, 'a> {
    file: &'f FileArtifact<'a>,
    text: &'f TextOp<'a>,
    node: NodeId,
}
impl<'a> FileArtifact<'a> {
    #[must_use]
    pub fn native_text_values(&self) -> &[NativeFileTextValue<'a>] {
        &self.facts.native_text_values
    }
    /// Attached diagnostic prefixes remain readable even in incomplete Files.
    /// Product completion must independently require the actual whole owner.
    #[must_use]
    pub fn native_text_value_for<'f>(
        &'f self,
        node: NodeId,
        text: &'f TextOp<'a>,
    ) -> Option<FileTextValue<'f, 'a>> {
        let [record] = self.facts.native_text_values.as_slice() else {
            return None;
        };
        let observation = record.observation()?;
        let source = observation.source();
        if record.state != NativeFileTextValueState::Attached(node)
            || record.text != Some(NonNull::from(text))
            || !self.artifact().contains_node(node)
            || !core::ptr::eq(source.authored_root(), self.artifact().source())
            || source.span() != text.span
            || !core::ptr::eq(source.text(), text.content)
        {
            return None;
        }
        Some(FileTextValue {
            file: self,
            text,
            node,
        })
    }
}
impl<'a> RejectedFile<'a> {
    #[must_use]
    pub fn native_text_values(&self) -> &[NativeFileTextValue<'a>] {
        &self.facts.native_text_values
    }
}
impl<'f, 'a> FileTextValue<'f, 'a> {
    #[must_use]
    pub const fn file(self) -> &'f FileArtifact<'a> {
        self.file
    }
    #[must_use]
    pub const fn text(self) -> &'f TextOp<'a> {
        self.text
    }
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.node
    }
    #[must_use]
    pub fn observation(self) -> Option<&'f NativeTextValue<'a>> {
        self.file.native_text_values().first()?.observation()
    }
}

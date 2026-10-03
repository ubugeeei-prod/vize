//! Original handler custody sealed with its actual attached File node.

use super::{BindingRef, FileArtifact, ScopeId};
use crate::lang::js::{NativeHandlerInput, RejectedNativeHandlerInput};
use crate::op::{HandlerId, OnHandlerRef, OnOp};
use crate::resolution::{HandlerResolution, ResolutionError};
use alloc::boxed::Box;
use vize_l1::markup::NativeAttributeHandlerFailure;

pub(crate) struct HandlerRecord<'a> {
    pub original: core::ptr::NonNull<OnOp<'a>>,
    pub scope: ScopeId,
    pub resolution: HandlerResolution<'a>,
}

pub(crate) struct PendingHandler<'a> {
    pub input: Option<NativeHandlerInput<'a>>,
    pub resolution: Option<HandlerResolution<'a>>,
    pub scope: ScopeId,
    pub span: vize_l0::Span,
}

/// Complete original observations remain readable after a header refusal.
#[derive(Debug)]
pub enum RejectedFileHandler<'a> {
    Observation(NativeAttributeHandlerFailure<'a>),
    Syntax(Box<RejectedNativeHandlerInput<'a>>),
    Resolution {
        input: NativeHandlerInput<'a>,
        error: ResolutionError,
    },
}

/// A short query borrow retains the actual File; an equal node ID is insufficient.
/// ```compile_fail
/// use vize_l2::file::{FileArtifact, FileHandler};
/// use vize_l2::op::HandlerId;
/// fn forge<'f,'a>(file: &'f FileArtifact<'a>, id: HandlerId) {
///     let _ = FileHandler { file, id };
/// }
/// ```
#[derive(Clone, Copy)]
pub struct FileHandler<'f, 'a> {
    file: &'f FileArtifact<'a>,
    id: HandlerId,
}

impl<'a> FileArtifact<'a> {
    /// Numeric file-local lookup only; use `handler_for` to join an actual On.
    #[must_use]
    pub fn handler(&self, id: HandlerId) -> Option<FileHandler<'_, 'a>> {
        self.facts.handlers.get(id.node())?;
        Some(FileHandler { file: self, id })
    }
    /// Join the exact checked attached On allocation with this actual File.
    /// Equal IDs, spans, fields or an On copied from another File do not join.
    #[must_use]
    pub fn handler_for(&self, on: &OnOp<'a>) -> Option<FileHandler<'_, 'a>> {
        let id = on.handler.and_then(OnHandlerRef::body)?;
        let handler = self.handler(id)?;
        handler.accepts_on(on).then_some(handler)
    }
    #[must_use]
    pub fn rejected_handlers(&self) -> &[RejectedFileHandler<'a>] {
        &self.facts.rejected_handlers
    }
    /// Original observations parked before a fallible header/handler walk.
    /// Their presence grants no attached node or native completion.
    pub fn unattached_handlers(&self) -> impl Iterator<Item = &NativeHandlerInput<'a>> {
        self.facts.pending_handlers.iter().filter_map(|handler| {
            handler
                .input
                .as_ref()
                .or_else(|| handler.resolution.as_ref().map(HandlerResolution::input))
        })
    }
}

impl<'f, 'a> FileHandler<'f, 'a> {
    #[must_use]
    pub fn accepts_on(self, on: &OnOp<'a>) -> bool {
        self.file
            .facts
            .handlers
            .get(self.id.node())
            .is_some_and(|record| core::ptr::eq(record.original.as_ptr(), on))
    }
    #[must_use]
    pub const fn file(self) -> &'f FileArtifact<'a> {
        self.file
    }
    #[must_use]
    pub const fn id(self) -> HandlerId {
        self.id
    }
    #[must_use]
    pub fn resolution(self) -> Option<&'f HandlerResolution<'a>> {
        Some(&self.file.facts.handlers.get(self.id.node())?.resolution)
    }
    #[must_use]
    pub fn scope(self) -> Option<ScopeId> {
        Some(self.file.facts.handlers.get(self.id.node())?.scope)
    }
    #[must_use]
    pub fn accepts(self, binding: BindingRef<'_, '_>) -> bool {
        core::ptr::eq(self.file, binding.file())
    }
    #[must_use]
    pub fn same_owner(self, other: FileHandler<'_, '_>) -> bool {
        core::ptr::eq(self.file, other.file)
    }
}

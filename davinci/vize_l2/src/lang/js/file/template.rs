//! Selected ownership with Text/Comment/empty and zero-header HTML bodies.

use super::FileProducer;
use crate::artifact::ArtifactError;
use crate::file::region::template::FileTemplateWalk;
use crate::file::{FileArtifact, FileIssueKind, RejectedFile, ScriptUnitId};
use vize_l0::Span;
use vize_l1::{embed::syntax::EmbedSyntax, markup::NativeTemplateComponent};

// The implementation is included from its compatibility module to retain the
// existing type identity. Keep an ordinary, disabled declaration here so
// rustfmt still discovers the language-owned source file.
#[cfg(any())]
mod header;
mod program;
mod scoped;
mod setup;
pub use scoped::{
    ScopedTemplateIssue, ScopedTemplateIssueKind, ScopedTemplateView,
};
pub use setup::{SelectedSetup, SelectedSetupIssue, SelectedSetupIssueKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateRouteIssueKind {
    Interpolation(super::super::InterpolationInputError),
    InterpolationPreparation {
        span: Span,
        kind: vize_l1::markup::InterpolationError,
    },
    AttributeValue {
        span: Span,
        kind: vize_l1::markup::AttributeOperandError,
    },
    TextValuePreparation {
        span: Span,
        kind: vize_l1::markup::TextValueError,
    },
    TextValuePolicy(crate::file::TextValuePolicyError),
    Artifact(ArtifactError),
    Program(FileIssueKind),
    SetupSyntax(vize_l1::embed::syntax::EmbedHole),
    SetupSource(vize_l1::embed::SourceError),
    SetupPolicy(SelectedSetupIssueKind),
    UnsupportedOrdinaryScript,
    UnsupportedStyle,
    MissingProgram,
    DuplicateProgram,
    InvalidProfile,
    InvalidEvent,
    IncompleteChildren,
    UnsupportedChild,
    Interrupted,
    UnsupportedInvocation,
    UnsupportedExport,
    ReservedBinding,
    Handler {
        span: Span,
        kind: FileIssueKind,
    },
    For {
        span: Span,
        kind: FileIssueKind,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemplateRouteIssue {
    pub span: Span,
    pub kind: TemplateRouteIssueKind,
}
#[derive(Clone, Copy)]
pub(crate) enum TemplateRouteState {
    Scripts,
    Walking,
    Complete,
    Refused(TemplateRouteIssue),
    Interrupted,
}

/// Consume original Descriptor-selected ownership before any fallible walk.
/// The File producer is private and created internally, never paired by caller.
/// ```compile_fail
/// use vize_l2::lang::js::{FileProducer, TemplateOwner};
/// fn replace<'a>(owner: &mut TemplateOwner<'a>, producer: FileProducer<'a>) {
///     owner.producer = producer;
/// }
/// ```
pub struct TemplateOwner<'a> {
    selected: NativeTemplateComponent<'a>,
    producer: FileProducer<'a>,
    ordinary: Option<ScriptUnitId>,
    setup: Option<ScriptUnitId>,
    retained_setup: Option<alloc::boxed::Box<EmbedSyntax<'a>>>,
    state: TemplateRouteState,
}
pub struct RejectedTemplateOwner<'a> {
    selected: NativeTemplateComponent<'a>,
    error: ArtifactError,
}
impl<'a> RejectedTemplateOwner<'a> {
    #[must_use]
    pub fn selected(&self) -> &NativeTemplateComponent<'a> {
        &self.selected
    }
    #[must_use]
    pub fn error(&self) -> ArtifactError {
        self.error
    }
}

impl<'a> TemplateOwner<'a> {
    pub fn new(
        selected: NativeTemplateComponent<'a>,
    ) -> Result<Self, alloc::boxed::Box<RejectedTemplateOwner<'a>>> {
        let producer = match FileProducer::new(
            selected.component().allocator(),
            selected.component().block().root_source(),
        ) {
            Ok(producer) => producer,
            Err(error) => {
                return Err(alloc::boxed::Box::new(RejectedTemplateOwner {
                    selected,
                    error,
                }));
            }
        };
        Ok(Self {
            selected,
            producer,
            ordinary: None,
            setup: None,
            retained_setup: None,
            state: TemplateRouteState::Scripts,
        })
    }
    #[must_use]
    pub fn selected(&self) -> &NativeTemplateComponent<'a> {
        &self.selected
    }
    pub fn begin(&mut self) -> Result<FileTemplateWalk<'_, 'a>, TemplateRouteIssue> {
        let span = self.selected.component().block().span();
        if !matches!(self.state, TemplateRouteState::Scripts) {
            return self.refuse(span, TemplateRouteIssueKind::Interrupted);
        }
        if self.selected.ordinary().is_some() != self.ordinary.is_some()
            || self.selected.setup().is_some() != self.setup.is_some()
        {
            return self.refuse(span, TemplateRouteIssueKind::MissingProgram);
        }
        if !self.producer.builder.facts.issues.is_empty()
            || !self
                .producer
                .builder
                .facts
                .units
                .iter()
                .all(crate::file::ScriptUnit::walk_completed)
        {
            return self.refuse(
                span,
                TemplateRouteIssueKind::Program(FileIssueKind::UnsupportedSyntax),
            );
        }
        let carrier = self.selected.component().carrier();
        let ordinary_empty = self.setup.is_none()
            && self
                .producer
                .ordinary_empty_script()
                .is_some_and(|receipt| Some(receipt.unit()) == self.ordinary);
        // The same Program walk records these bits even without an observer.
        // This private two-selected-unit preflight does not enumerate the AST.
        for unit in &self.producer.builder.facts.units {
            let kind = if unit.origin.has_call {
                Some(TemplateRouteIssueKind::UnsupportedInvocation)
            } else if unit.origin.has_export && !(ordinary_empty && Some(unit.id) == self.ordinary)
            {
                Some(TemplateRouteIssueKind::UnsupportedExport)
            } else if unit.origin.reserved_binding {
                Some(TemplateRouteIssueKind::ReservedBinding)
            } else {
                None
            };
            if let Some(kind) = kind {
                let span = unit.span;
                return self.refuse(span, kind);
            }
        }
        if !carrier.errors.is_empty() || !carrier.unsupported.is_empty() {
            return self.refuse(span, TemplateRouteIssueKind::UnsupportedChild);
        }
        FileTemplateWalk::new(
            &self.selected,
            &mut self.producer.builder,
            &mut self.state,
            self.setup,
        )
    }
    #[must_use]
    pub fn finish(self) -> TemplateFile<'a> {
        TemplateFile {
            selected: self.selected,
            outcome: self.producer.finish(),
            state: self.state,
            retained_setup: self.retained_setup,
        }
    }
}

/// Keep original Component and canonical File together across movement/failure.
/// ```compile_fail
/// use vize_l2::{file::FileArtifact, lang::js::TemplateFile};
/// fn replace<'a>(owner: &mut TemplateFile<'a>, file: FileArtifact<'a>) {
///     owner.outcome = Ok(file);
/// }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::TemplateFile;
/// fn move_after_view(owner: TemplateFile<'_>) {
///     if let Ok(view) = owner.view() { drop(owner); let _ = view.file(); }
/// }
/// ```
pub struct TemplateFile<'a> {
    selected: NativeTemplateComponent<'a>,
    outcome: Result<FileArtifact<'a>, RejectedFile<'a>>,
    state: TemplateRouteState,
    retained_setup: Option<alloc::boxed::Box<EmbedSyntax<'a>>>,
}
impl<'a> TemplateFile<'a> {
    #[must_use]
    pub fn selected(&self) -> &NativeTemplateComponent<'a> {
        &self.selected
    }
    #[must_use]
    pub fn file(&self) -> Option<&FileArtifact<'a>> {
        self.outcome.as_ref().ok()
    }
    #[must_use]
    pub fn rejected_file(&self) -> Option<&RejectedFile<'a>> {
        self.outcome.as_ref().err()
    }
    pub fn view(&self) -> Result<TemplateFileView<'_, 'a>, TemplateRouteIssue> {
        if matches!(self.state, TemplateRouteState::Complete)
            && self.file().is_some_and(FileArtifact::is_complete)
        {
            return Ok(TemplateFileView { owner: self });
        }
        let issue = match self.state {
            TemplateRouteState::Refused(issue) => issue,
            _ => TemplateRouteIssue {
                span: self.selected.component().block().span(),
                kind: TemplateRouteIssueKind::Interrupted,
            },
        };
        Err(issue)
    }
}
/// Constructor private: no external File+Component pair or numeric registrar.
/// ```compile_fail
/// use vize_l2::lang::js::TemplateFileView;
/// fn copy(view: TemplateFileView<'_, '_>) { let _ = view.clone(); }
/// ```
pub struct TemplateFileView<'f, 'a> {
    owner: &'f TemplateFile<'a>,
}
impl<'f, 'a> TemplateFileView<'f, 'a> {
    #[must_use]
    pub fn owner(&self) -> &'f TemplateFile<'a> {
        self.owner
    }
    #[must_use]
    pub fn file(&self) -> Option<&'f FileArtifact<'a>> {
        self.owner.file()
    }
}

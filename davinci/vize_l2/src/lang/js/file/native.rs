//! Selected ownership with Text/Comment/empty and zero-header HTML bodies.

use super::L2IrProducer;
use crate::artifact::L2TemplateError;
use crate::file::region::native::NativeTemplateWalk;
use crate::file::{L2Ir, L2IrIssueKind, RejectedL2Ir, ScriptUnitId};
use vize_l0::Span;
use vize_l1::{embed::syntax::EmbedSyntax, markup::SelectedTemplateComponent};

// The implementation is included from its compatibility module to retain the
// existing type identity. Keep an ordinary, disabled declaration here so
// rustfmt still discovers the language-owned source file.
#[cfg(any())]
mod header;
mod program;
mod scoped;
mod setup;
pub use scoped::{
    NativeScopedTemplateIssue, NativeScopedTemplateIssueKind, NativeScopedTemplateView,
};
pub use setup::{NativeSelectedSetup, NativeSetupIssue, NativeSetupIssueKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTemplateIssueKind {
    Interpolation(super::super::NativeInterpolationInputError),
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
    TextValuePolicy(crate::file::NativeTextValuePolicyError),
    Artifact(L2TemplateError),
    Program(L2IrIssueKind),
    SetupSyntax(vize_l1::embed::syntax::EmbedHole),
    SetupSource(vize_l1::embed::SourceError),
    SetupPolicy(NativeSetupIssueKind),
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
        kind: L2IrIssueKind,
    },
    For {
        span: Span,
        kind: L2IrIssueKind,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeTemplateIssue {
    pub span: Span,
    pub kind: NativeTemplateIssueKind,
}
#[derive(Clone, Copy)]
pub(crate) enum NativeRouteState {
    Scripts,
    Walking,
    Complete,
    Refused(NativeTemplateIssue),
    Interrupted,
}

/// Consume original Descriptor-selected ownership before any fallible walk.
/// The File producer is private and created internally, never paired by caller.
/// ```compile_fail
/// use vize_l2::lang::js::{L2IrProducer, NativeTemplateOwner};
/// fn replace<'a>(owner: &mut NativeTemplateOwner<'a>, producer: L2IrProducer<'a>) {
///     owner.producer = producer;
/// }
/// ```
pub struct NativeTemplateOwner<'a> {
    selected: SelectedTemplateComponent<'a>,
    producer: L2IrProducer<'a>,
    ordinary: Option<ScriptUnitId>,
    setup: Option<ScriptUnitId>,
    retained_setup: Option<alloc::boxed::Box<EmbedSyntax<'a>>>,
    state: NativeRouteState,
}
pub struct RejectedNativeTemplateOwner<'a> {
    selected: SelectedTemplateComponent<'a>,
    error: L2TemplateError,
}
impl<'a> RejectedNativeTemplateOwner<'a> {
    #[must_use]
    pub fn selected(&self) -> &SelectedTemplateComponent<'a> {
        &self.selected
    }
    #[must_use]
    pub fn error(&self) -> L2TemplateError {
        self.error
    }
}

impl<'a> NativeTemplateOwner<'a> {
    pub fn new(
        selected: SelectedTemplateComponent<'a>,
    ) -> Result<Self, alloc::boxed::Box<RejectedNativeTemplateOwner<'a>>> {
        let producer = match L2IrProducer::new(
            selected.component().allocator(),
            selected.component().block().root_source(),
        ) {
            Ok(producer) => producer,
            Err(error) => {
                return Err(alloc::boxed::Box::new(RejectedNativeTemplateOwner {
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
            state: NativeRouteState::Scripts,
        })
    }
    #[must_use]
    pub fn selected(&self) -> &SelectedTemplateComponent<'a> {
        &self.selected
    }
    pub fn begin(&mut self) -> Result<NativeTemplateWalk<'_, 'a>, NativeTemplateIssue> {
        let span = self.selected.component().block().span();
        if !matches!(self.state, NativeRouteState::Scripts) {
            return self.refuse(span, NativeTemplateIssueKind::Interrupted);
        }
        if self.selected.ordinary().is_some() != self.ordinary.is_some()
            || self.selected.setup().is_some() != self.setup.is_some()
        {
            return self.refuse(span, NativeTemplateIssueKind::MissingProgram);
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
                NativeTemplateIssueKind::Program(L2IrIssueKind::UnsupportedSyntax),
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
                Some(NativeTemplateIssueKind::UnsupportedInvocation)
            } else if unit.origin.has_export && !(ordinary_empty && Some(unit.id) == self.ordinary)
            {
                Some(NativeTemplateIssueKind::UnsupportedExport)
            } else if unit.origin.reserved_binding {
                Some(NativeTemplateIssueKind::ReservedBinding)
            } else {
                None
            };
            if let Some(kind) = kind {
                let span = unit.span;
                return self.refuse(span, kind);
            }
        }
        if !carrier.errors.is_empty() || !carrier.unsupported.is_empty() {
            return self.refuse(span, NativeTemplateIssueKind::UnsupportedChild);
        }
        NativeTemplateWalk::new(
            &self.selected,
            &mut self.producer.builder,
            &mut self.state,
            self.setup,
        )
    }
    #[must_use]
    pub fn finish(self) -> NativeTemplateL2Ir<'a> {
        NativeTemplateL2Ir {
            selected: self.selected,
            outcome: self.producer.finish(),
            state: self.state,
            retained_setup: self.retained_setup,
        }
    }
}

/// Keep original Component and canonical File together across movement/failure.
/// ```compile_fail
/// use vize_l2::{file::L2Ir, lang::js::NativeTemplateL2Ir};
/// fn replace<'a>(owner: &mut NativeTemplateL2Ir<'a>, l2ir: L2Ir<'a>) {
///     owner.outcome = Ok(l2ir);
/// }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::NativeTemplateL2Ir;
/// fn move_after_view(owner: NativeTemplateL2Ir<'_>) {
///     if let Ok(view) = owner.view() { drop(owner); let _ = view.file(); }
/// }
/// ```
pub struct NativeTemplateL2Ir<'a> {
    selected: SelectedTemplateComponent<'a>,
    outcome: Result<L2Ir<'a>, RejectedL2Ir<'a>>,
    state: NativeRouteState,
    retained_setup: Option<alloc::boxed::Box<EmbedSyntax<'a>>>,
}
impl<'a> NativeTemplateL2Ir<'a> {
    #[must_use]
    pub fn selected(&self) -> &SelectedTemplateComponent<'a> {
        &self.selected
    }
    #[must_use]
    pub fn file(&self) -> Option<&L2Ir<'a>> {
        self.outcome.as_ref().ok()
    }
    #[must_use]
    pub fn rejected_file(&self) -> Option<&RejectedL2Ir<'a>> {
        self.outcome.as_ref().err()
    }
    pub fn view(&self) -> Result<NativeTemplateView<'_, 'a>, NativeTemplateIssue> {
        if matches!(self.state, NativeRouteState::Complete)
            && self.file().is_some_and(L2Ir::is_complete)
        {
            return Ok(NativeTemplateView { owner: self });
        }
        let issue = match self.state {
            NativeRouteState::Refused(issue) => issue,
            _ => NativeTemplateIssue {
                span: self.selected.component().block().span(),
                kind: NativeTemplateIssueKind::Interrupted,
            },
        };
        Err(issue)
    }
}
/// Constructor private: no external File+Component pair or numeric registrar.
/// ```compile_fail
/// use vize_l2::lang::js::NativeTemplateView;
/// fn copy(view: NativeTemplateView<'_, '_>) { let _ = view.clone(); }
/// ```
pub struct NativeTemplateView<'f, 'a> {
    owner: &'f NativeTemplateL2Ir<'a>,
}
impl<'f, 'a> NativeTemplateView<'f, 'a> {
    #[must_use]
    pub fn owner(&self) -> &'f NativeTemplateL2Ir<'a> {
        self.owner
    }
    #[must_use]
    pub fn file(&self) -> Option<&'f L2Ir<'a>> {
        self.owner.file()
    }
}

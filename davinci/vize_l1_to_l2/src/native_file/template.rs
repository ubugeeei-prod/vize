use super::{NativeSfcIssue, NativeSfcIssueKind};
use crate::native::{
    NativeComponent, NativeEmbed, NativeHole, NativeProduced, NativeVueConstructionError,
    NativeVueProduced, PendingNativeComponent, RejectedNativeComponent,
};
use crate::vue_file::{VueFileIssue, VueFileProducer};
use alloc::{boxed::Box, vec::Vec};
use vize_l0::{Allocator, SourceBlock, diag::Diagnostic};
use vize_l1::{
    container::vue::TemplateView,
    embed::{Lang, syntax::NativeSyntax},
    markup::ComponentSourceError,
};

#[derive(Debug)]
pub enum NativeTemplateOutcome<'a> {
    /// Original observations stay in the private pending owner, without authority.
    ConstructionPending,
    ConstructionRefused(NativeVueConstructionError),
    Produced(NativeVueProduced<'a>),
    FactoryRefused {
        component: NativeComponent<'a>,
        issue: VueFileIssue,
    },
    ComponentRejected(Box<RejectedNativeComponent<'a>>),
    FileUnavailable(NativeComponent<'a>),
    SourceRefused(ComponentSourceError),
}

/// Actual selected template and the whole original parse/construction outcome.
///
/// ```compile_fail
/// use vize_l1_to_l2::native_file::NativeTemplateObservation;
/// fn discard(owner: &mut NativeTemplateObservation<'_>) {
///     let _ = owner.pending.take();
/// }
/// ```
///
/// ```compile_fail
/// use vize_l1_to_l2::native_file::NativeTemplateObservation;
/// fn rewrite(owner: &NativeTemplateObservation<'_>) {
///     if let Some(original) = owner.produced() {
///         original.embeds.clear();
///     }
/// }
/// ```
pub struct NativeTemplateObservation<'a> {
    container_index: usize,
    block: SourceBlock<'a>,
    lang: Lang,
    pending: Option<PendingNativeComponent<'a>>,
    outcome: NativeTemplateOutcome<'a>,
}
impl core::fmt::Debug for NativeTemplateObservation<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("NativeTemplateObservation")
            .field("container_index", &self.container_index)
            .field("block", &self.block)
            .field("lang", &self.lang)
            .field("pending", &self.pending)
            .field("outcome", &self.outcome)
            .finish_non_exhaustive()
    }
}
impl<'a> NativeTemplateObservation<'a> {
    #[must_use]
    pub fn container_index(&self) -> usize {
        self.container_index
    }
    #[must_use]
    pub fn block(&self) -> SourceBlock<'a> {
        self.block
    }
    /// Descriptor-derived script language, independent of the File's full profile.
    #[must_use]
    pub fn lang(&self) -> Lang {
        self.lang
    }
    #[must_use]
    pub fn outcome(&self) -> &NativeTemplateOutcome<'a> {
        &self.outcome
    }
    /// Read the actual original owner even before a genuine native walk can run.
    #[must_use]
    pub fn component(&self) -> Option<&NativeComponent<'a>> {
        if let Some(pending) = &self.pending {
            return Some(pending.component());
        }
        match &self.outcome {
            NativeTemplateOutcome::Produced(produced) => Some(&produced.produced().component),
            NativeTemplateOutcome::FactoryRefused { component, .. }
            | NativeTemplateOutcome::FileUnavailable(component) => Some(component),
            NativeTemplateOutcome::ComponentRejected(rejected) => Some(&rejected.component),
            NativeTemplateOutcome::ConstructionPending
            | NativeTemplateOutcome::ConstructionRefused(_)
            | NativeTemplateOutcome::SourceRefused(_) => None,
        }
    }
    #[must_use]
    pub fn produced(&self) -> Option<&NativeProduced<'a>> {
        match &self.outcome {
            NativeTemplateOutcome::Produced(produced) => Some(produced.produced()),
            _ => None,
        }
    }
    /// Original observations remain readable while construction is unfinished.
    #[must_use]
    pub fn holes(&self) -> &[NativeHole] {
        if let Some(pending) = &self.pending {
            return pending.observations().holes();
        }
        self.produced().map_or(&[], |produced| &produced.holes)
    }
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        if let Some(pending) = &self.pending {
            return pending.observations().diagnostics();
        }
        self.produced()
            .map_or(&[], |produced| &produced.diagnostics)
    }
    #[must_use]
    pub fn embeds(&self) -> &[NativeEmbed<'a>] {
        if let Some(pending) = &self.pending {
            return pending.observations().embeds();
        }
        self.produced().map_or(&[], |produced| &produced.embeds)
    }
    #[must_use]
    pub fn rejected_syntax(&self) -> &[NativeSyntax<'a>] {
        if let Some(pending) = &self.pending {
            return pending.observations().rejected_syntax();
        }
        self.produced()
            .map_or(&[], |produced| &produced.rejected_syntax)
    }
    pub(super) fn matches(&self, selected: TemplateView<'_, 'a>, lang: Lang) -> bool {
        self.container_index == selected.container_index()
            && self.block.span() == selected.block().span()
            && core::ptr::eq(self.block.source(), selected.block().source())
            && core::ptr::eq(self.block.root_source(), selected.source())
            && self.lang == lang
            && self.pending.is_none()
            && self.produced().is_some_and(|produced| {
                produced.is_supported()
                    && produced.component.block().span() == self.block.span()
                    && core::ptr::eq(produced.component.block().source(), self.block.source())
                    && core::ptr::eq(produced.component.block().root_source(), selected.source())
            })
    }
    fn reject(&self, issues: &mut Vec<NativeSfcIssue>, kind: NativeSfcIssueKind) {
        issues.push(NativeSfcIssue {
            container_index: Some(self.container_index),
            span: Some(self.block.span()),
            kind,
        });
    }
    /// Transfer a pre-driver refusal only; a started owner remains parked intact.
    fn unconstructed(&mut self) -> Option<NativeComponent<'a>> {
        let pending = self.pending.take()?;
        match pending.into_unconstructed_component() {
            Ok(component) => Some(component),
            Err(pending) => {
                self.pending = Some(*pending);
                None
            }
        }
    }
    pub(super) fn construct(
        &mut self,
        producer: Option<&mut VueFileProducer<'a>>,
        issues: &mut Vec<NativeSfcIssue>,
    ) {
        if self.pending.is_none() {
            return;
        }
        let Some(producer) = producer else {
            if let Some(component) = self.unconstructed() {
                self.outcome = NativeTemplateOutcome::FileUnavailable(component);
            }
            self.reject(issues, NativeSfcIssueKind::FileRejected);
            return;
        };
        let result = {
            let mut region = match producer.template_region() {
                Ok(region) => region,
                Err(issue) => {
                    if let Some(component) = self.unconstructed() {
                        self.outcome = NativeTemplateOutcome::FactoryRefused { component, issue };
                    }
                    self.reject(issues, NativeSfcIssueKind::TemplateFactory(issue));
                    return;
                }
            };
            let Some(pending) = self.pending.as_mut() else {
                self.reject(issues, NativeSfcIssueKind::TemplateConstructionPending);
                return;
            };
            // The genuine entry consumes its lower normal-end guard in this live
            // region and retains its private receipt on this same parked owner.
            pending.construct_vue_file_in(&mut region)
        };
        if let Err(error) = result {
            self.outcome = NativeTemplateOutcome::ConstructionRefused(error);
            self.reject(issues, NativeSfcIssueKind::TemplateConstruction(error));
            return;
        }
        // The fallible driver and this producer's region borrow have ended.
        // Only the private genuine receipt permits this normal owner transfer.
        let Some(pending) = self.pending.take() else {
            self.reject(issues, NativeSfcIssueKind::TemplateConstructionPending);
            return;
        };
        match pending.into_vue_produced() {
            Ok(produced) => {
                if !produced.produced().is_supported() {
                    self.reject(issues, NativeSfcIssueKind::TemplateUnsupported);
                }
                self.outcome = NativeTemplateOutcome::Produced(produced);
            }
            Err(pending) => {
                self.pending = Some(*pending);
                self.reject(issues, NativeSfcIssueKind::TemplateConstructionPending);
            }
        }
    }
}

pub(super) fn prepare<'a>(
    allocator: &'a Allocator,
    selected: TemplateView<'_, 'a>,
    lang: Lang,
    issues: &mut Vec<NativeSfcIssue>,
) -> NativeTemplateObservation<'a> {
    let block = selected.block();
    let reject = |issues: &mut Vec<NativeSfcIssue>, kind| {
        issues.push(NativeSfcIssue {
            container_index: Some(selected.container_index()),
            span: Some(block.span()),
            kind,
        });
    };
    let (pending, outcome) = match NativeComponent::parse_in(allocator, block) {
        Ok(component) => (
            Some(PendingNativeComponent::from_component(component)),
            NativeTemplateOutcome::ConstructionPending,
        ),
        Err(error) => {
            reject(issues, NativeSfcIssueKind::TemplateSource(error));
            (None, NativeTemplateOutcome::SourceRefused(error))
        }
    };
    NativeTemplateObservation {
        container_index: selected.container_index(),
        block,
        lang,
        pending,
        outcome,
    }
}

#[cfg(test)]
mod tests;

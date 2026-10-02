//! Normally owned observations survive a caught construction unwind.

use super::block::construct_component;
use super::construction::Diagnostic as DiagnosticFactory;
use super::{NativeComponent, NativeEmbed, NativeHole, NativeProduced, RejectedNativeComponent};
use alloc::{boxed::Box, vec::Vec};
use vize_l0::diag::Diagnostic;
use vize_l1::embed::{Lang, syntax::NativeSyntax};
use vize_l2::artifact::ComponentFactory;

/// Original normal owners, borrowed by Context during its sole construction walk.
pub(crate) struct NativeObservations<'a> {
    pub(super) holes: Vec<NativeHole>,
    pub(super) diagnostics: Vec<Diagnostic>,
    pub(super) embeds: Vec<NativeEmbed<'a>>,
    pub(super) rejected_syntax: Vec<NativeSyntax<'a>>,
}

impl core::fmt::Debug for NativeObservations<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NativeObservations")
            .field("holes", &self.holes())
            .field("diagnostics", &self.diagnostics())
            .field("embeds", &self.embeds())
            .field("rejected_syntax", &self.rejected_syntax())
            .finish()
    }
}

impl<'a> NativeObservations<'a> {
    pub(super) fn new() -> Self {
        Self {
            holes: Vec::new(),
            diagnostics: Vec::new(),
            embeds: Vec::new(),
            rejected_syntax: Vec::new(),
        }
    }

    pub(crate) fn holes(&self) -> &[NativeHole] {
        &self.holes
    }

    pub(crate) fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub(crate) fn embeds(&self) -> &[NativeEmbed<'a>] {
        &self.embeds
    }

    pub(crate) fn rejected_syntax(&self) -> &[NativeSyntax<'a>] {
        &self.rejected_syntax
    }
}

/// Actual driver state; normal return grants no File semantic completeness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeWalkState {
    Ready,
    Walking,
    Refused,
    Interrupted,
    NormalEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PendingConstructionError {
    AlreadyStarted,
    ForeignSource,
}

/// Park this owner in the assembler's existing template row before construction.
pub(crate) struct PendingNativeComponent<'a> {
    component: NativeComponent<'a>,
    observations: NativeObservations<'a>,
    state: NativeWalkState,
}

impl core::fmt::Debug for PendingNativeComponent<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PendingNativeComponent")
            .field("component", self.component())
            .field("observations", self.observations())
            .field("state", &self.state())
            .finish()
    }
}

impl<'a> PendingNativeComponent<'a> {
    pub(crate) fn from_component(component: NativeComponent<'a>) -> Self {
        Self {
            component,
            observations: NativeObservations::new(),
            state: NativeWalkState::Ready,
        }
    }

    pub(crate) fn component(&self) -> &NativeComponent<'a> {
        &self.component
    }

    pub(crate) fn observations(&self) -> &NativeObservations<'a> {
        &self.observations
    }

    pub(crate) fn state(&self) -> NativeWalkState {
        self.state
    }

    /// The unchanged diagnostic capability cannot issue a native File receipt.
    pub(crate) fn construct_diagnostic_in<R: ComponentFactory<'a>>(
        &mut self,
        region: &mut R,
        lang: Lang,
    ) -> Result<(), PendingConstructionError> {
        if self.state != NativeWalkState::Ready {
            return Err(PendingConstructionError::AlreadyStarted);
        }
        let guard = NativeWalkGuard::begin(&mut self.state);
        let expected = self.component.block().root_source();
        let mut region = DiagnosticFactory::new(region);
        let actual = super::ConstructionFactory::source(&region);
        if expected.as_ptr() != actual.as_ptr() || expected.len() != actual.len() {
            guard.refuse();
            return Err(PendingConstructionError::ForeignSource);
        }
        construct_component(&self.component, &mut self.observations, &mut region, lang);
        guard.normal_end();
        Ok(())
    }

    pub(crate) fn into_produced(self) -> Result<NativeProduced<'a>, Box<Self>> {
        if self.state != NativeWalkState::NormalEnd {
            return Err(Box::new(self));
        }
        let Self {
            component,
            observations,
            state: _,
        } = self;
        let NativeObservations {
            holes,
            diagnostics,
            embeds,
            rejected_syntax,
        } = observations;
        Ok(NativeProduced {
            component,
            holes,
            diagnostics,
            embeds,
            rejected_syntax,
        })
    }

    pub(super) fn source_rejection(self) -> Box<RejectedNativeComponent<'a>> {
        Box::new(RejectedNativeComponent {
            component: self.component,
        })
    }
}

/// Drop retains all owners and changes only this inline state.
struct NativeWalkGuard<'s> {
    state: &'s mut NativeWalkState,
}

impl<'s> NativeWalkGuard<'s> {
    fn begin(state: &'s mut NativeWalkState) -> Self {
        *state = NativeWalkState::Walking;
        Self { state }
    }

    fn refuse(self) {
        *self.state = NativeWalkState::Refused;
    }

    fn normal_end(self) {
        *self.state = NativeWalkState::NormalEnd;
    }
}

impl Drop for NativeWalkGuard<'_> {
    fn drop(&mut self) {
        if *self.state == NativeWalkState::Walking {
            *self.state = NativeWalkState::Interrupted;
        }
    }
}

#[cfg(test)]
mod tests;

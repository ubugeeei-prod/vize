//! Genuine native custody is retained on the same normally owned pending row.

use super::{NativeObservations, NativeWalkGuard, NativeWalkState, PendingNativeComponent};
use crate::native::block::construct_component;
use crate::native::{
    ConstructionFactory, NativeComponent, NativeEmbed, NativeHole, NativeProduced,
};
use crate::vue_file::{NativeTemplateProfile, VueFileRegion};
use alloc::boxed::Box;
use vize_l0::{SourceBlock, diag::Diagnostic};
use vize_l1::embed::syntax::NativeSyntax;
use vize_l2::artifact::ArtifactError;
use vize_l2::file::TemplateIssue;

/// Source, interruption and repeat refusal retain the original pending owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeVueConstructionError {
    AlreadyStarted,
    ForeignSource,
    NativeAuthorityUnavailable,
    Artifact(ArtifactError),
    Interrupted(TemplateIssue),
}

/// Only the private same-region normal-end route can construct this receipt.
pub(super) struct NativeConstructionReceipt<'a> {
    block: SourceBlock<'a>,
    profile: NativeTemplateProfile,
}

impl NativeConstructionReceipt<'_> {
    fn matches(&self, block: SourceBlock<'_>) -> bool {
        self.block.root_source().as_ptr() == block.root_source().as_ptr()
            && self.block.root_source().len() == block.root_source().len()
            && self.block.source().as_ptr() == block.source().as_ptr()
            && self.block.source().len() == block.source().len()
            && self.block.span() == block.span()
    }
}

/// The same original observations plus private uninterrupted native custody.
///
/// File/Program/Vue semantic completeness remains an independent obligation.
/// No caller constructor, mutable extraction, Clone or Copy is provided.
pub struct NativeVueProduced<'a> {
    produced: NativeProduced<'a>,
    receipt: NativeConstructionReceipt<'a>,
}

impl core::fmt::Debug for NativeVueProduced<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NativeVueProduced")
            .field("produced", self.produced())
            .field("native_language", &self.receipt.profile.lang())
            .finish_non_exhaustive()
    }
}

impl<'a> NativeVueProduced<'a> {
    #[must_use]
    pub fn produced(&self) -> &NativeProduced<'a> {
        &self.produced
    }
}

/// A consuming public refusal keeps the same Component and partial observations.
#[derive(Debug)]
pub struct RejectedNativeVueComponent<'a> {
    pending: PendingNativeComponent<'a>,
    error: NativeVueConstructionError,
}

impl<'a> RejectedNativeVueComponent<'a> {
    #[must_use]
    pub fn error(&self) -> NativeVueConstructionError {
        self.error
    }
    #[must_use]
    pub fn component(&self) -> &NativeComponent<'a> {
        self.pending.component()
    }
    #[must_use]
    pub fn holes(&self) -> &[NativeHole] {
        self.pending.observations().holes()
    }
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        self.pending.observations().diagnostics()
    }
    #[must_use]
    pub fn embeds(&self) -> &[NativeEmbed<'a>] {
        self.pending.observations().embeds()
    }
    #[must_use]
    pub fn rejected_syntax(&self) -> &[NativeSyntax<'a>] {
        self.pending.observations().rejected_syntax()
    }
}

impl<'a> PendingNativeComponent<'a> {
    /// Borrow the sole row; its Component and observations never leave it here.
    /// The intrinsic language comes exclusively from this real File region.
    pub(crate) fn construct_vue_file_in(
        &mut self,
        region: &mut VueFileRegion<'_, 'a>,
    ) -> Result<(), NativeVueConstructionError> {
        if self.state != NativeWalkState::Ready {
            return Err(NativeVueConstructionError::AlreadyStarted);
        }
        let guard = NativeWalkGuard::begin(&mut self.state);
        let block = self.component.block();
        let mut walk = match region.begin_native_walk(block.span()) {
            Ok(walk) => walk,
            Err(error) => {
                guard.refuse();
                return Err(NativeVueConstructionError::Artifact(error));
            }
        };
        let profile = walk.profile();
        {
            #[cfg(not(test))]
            let mut factory = walk.factory();
            #[cfg(test)]
            let mut inner = walk.factory();
            #[cfg(test)]
            let mut factory = testing::Factory::new(&mut inner, self.component.test_allocator());
            let actual = factory.source();
            let expected = block.root_source();
            if expected.as_ptr() != actual.as_ptr() || expected.len() != actual.len() {
                guard.refuse();
                return Err(NativeVueConstructionError::ForeignSource);
            }
            construct_component(
                &self.component,
                &mut self.observations,
                &mut factory,
                profile.lang(),
            );
        }
        // Consume the real guard while borrowing this same region. A caught
        // lower interruption must refuse this route, even after normal return.
        walk.complete()
            .map_err(NativeVueConstructionError::Interrupted)?;
        self.native = Some(NativeConstructionReceipt { block, profile });
        guard.normal_end();
        Ok(())
    }

    pub(crate) fn into_vue_produced(self) -> Result<NativeVueProduced<'a>, Box<Self>> {
        if self.state != NativeWalkState::NormalEnd
            || self
                .native
                .as_ref()
                .is_none_or(|receipt| !receipt.matches(self.component.block()))
        {
            return Err(Box::new(self));
        }
        let Self {
            component,
            observations,
            state: _,
            native,
        } = self;
        let Some(receipt) = native else {
            return Err(Box::new(Self {
                component,
                observations,
                state: NativeWalkState::NormalEnd,
                native: None,
            }));
        };
        let NativeObservations {
            holes,
            diagnostics,
            embeds,
            rejected_syntax,
        } = observations;
        Ok(NativeVueProduced {
            produced: NativeProduced {
                component,
                holes,
                diagnostics,
                embeds,
                rejected_syntax,
            },
            receipt,
        })
    }
}

#[cfg(test)]
mod testing;
#[cfg(test)]
pub(crate) use testing::{Point, Probe};
#[cfg(test)]
mod tests;

impl<'a> NativeComponent<'a> {
    /// The whole-owner native route has no caller language or generic factory.
    /// Caught inner unwinds require the assembler's parked borrowed-row route.
    pub fn construct_vue_file_in(
        self,
        region: &mut VueFileRegion<'_, 'a>,
    ) -> Result<NativeVueProduced<'a>, Box<RejectedNativeVueComponent<'a>>> {
        let mut pending = PendingNativeComponent::from_component(self);
        if let Err(error) = pending.construct_vue_file_in(region) {
            return Err(Box::new(RejectedNativeVueComponent { pending, error }));
        }
        pending.into_vue_produced().map_err(|pending| {
            Box::new(RejectedNativeVueComponent {
                pending: *pending,
                error: NativeVueConstructionError::NativeAuthorityUnavailable,
            })
        })
    }
}

//! Presence-only SDK facts from one authentic supplied direct child.
//!
//! This artifact does not walk siblings, parse expressions, or certify that a
//! whole body has no markers. Its sealed constructor retains original custody.

use vize_l0::{
    diag::{
        WitnessChain, WitnessLink,
        verify::{WitnessError, verify_chain},
    },
    fact::{FactConsumer, FactError, FactManager, FactTable, FactView},
};
use vize_l1::markup::NativeChild;

use super::{
    NativeLintRefusal,
    header_facts::{NativeHeaderFact, NativeHeaderFacts, NativeLintHeaders},
};

mod groups;
mod marker;
mod observations;

use groups::{CHECKS, REGISTRY};
pub use groups::{NativeDirectInterpolations, NativeTextareaMustache, TextareaMustacheDemand};
pub use observations::{NativeInterpolationFact, NativeTextareaMustacheFact};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeChildFactError {
    ForeignChild,
    WrongParent,
    RawInterpolation,
    UnexpectedChild,
    Admission(NativeLintRefusal),
    Fact(FactError),
}
impl From<NativeLintRefusal> for NativeChildFactError {
    fn from(error: NativeLintRefusal) -> Self {
        Self::Admission(error)
    }
}
impl From<FactError> for NativeChildFactError {
    fn from(error: FactError) -> Self {
        Self::Fact(error)
    }
}

struct Evidence {
    header_key: u32,
    header: NativeHeaderFact,
    marker: Option<(u32, NativeInterpolationFact)>,
}

/// Non-cloneable, owner/parent/ordinal-bound facts for exactly one supplied child.
/// Empty tables say only that this child has no admitted interpolation marker.
///
/// ```compile_fail
/// use vize_patina::native::child_facts::NativeChildFacts;
/// fn cloneable<T: Clone>() {}
/// cloneable::<NativeChildFacts<'static, 'static, 'static>>();
/// ```
pub struct NativeChildFacts<'s, 'o, 'a> {
    header: &'s NativeHeaderFacts<'o, 'a>,
    original: NativeChild<'s, 'a>,
    manager: FactManager<'static, Evidence>,
}

impl<'o, 'a> NativeHeaderFacts<'o, 'a> {
    /// Observe one original direct child at the caller's existing traversal.
    /// No caller key, source, span, category or arbitrary body tree is accepted.
    pub fn child_facts<'s>(
        &'s self,
        child: NativeChild<'s, 'a>,
    ) -> Result<NativeChildFacts<'s, 'o, 'a>, NativeChildFactError> {
        if !core::ptr::eq(self.owner().component(), child.component()) {
            return Err(NativeChildFactError::ForeignChild);
        }
        if child
            .parent_element()
            .is_none_or(|parent| !core::ptr::eq(parent, self.original()))
        {
            return Err(NativeChildFactError::WrongParent);
        }
        if self
            .original()
            .children
            .get(child.ordinal())
            .is_none_or(|original| !core::ptr::eq(original, child.surface()))
        {
            return Err(NativeLintRefusal::SourceMismatch.into());
        }
        let inputs = self.facts::<groups::HeaderDemand>();
        let (header_key, header) = inputs
            .get::<NativeLintHeaders>()?
            .iter()
            .next()
            .ok_or(NativeLintRefusal::SourceMismatch)?;
        let marker = marker::observe(self, &child, *header_key)?;
        let evidence = Evidence {
            header_key: *header_key,
            header: header.clone(),
            marker,
        };
        let mut manager = FactManager::new(&REGISTRY);
        manager.prepare::<TextareaMustacheDemand>(&evidence)?;
        Ok(NativeChildFacts {
            header: self,
            original: child,
            manager,
        })
    }
}

impl<'s, 'o, 'a> NativeChildFacts<'s, 'o, 'a> {
    #[must_use]
    pub fn header(&self) -> &'s NativeHeaderFacts<'o, 'a> {
        self.header
    }
    #[must_use]
    pub fn original(&self) -> NativeChild<'_, 'a> {
        self.original.reborrow()
    }
    #[must_use]
    pub fn facts<C: FactConsumer>(&self) -> FactView<'_> {
        self.manager.view::<C>()
    }
    pub fn markers(&self) -> Result<&FactTable<NativeDirectInterpolations>, FactError> {
        self.facts::<TextareaMustacheDemand>()
            .get::<NativeDirectInterpolations>()
    }
    pub fn textarea_mustache(&self) -> Result<&FactTable<NativeTextareaMustache>, FactError> {
        self.facts::<TextareaMustacheDemand>()
            .get::<NativeTextareaMustache>()
    }
    /// The actual original header, marker and derived counterexample ranges.
    pub fn textarea_mustache_chain(&self, child: u32) -> Result<Option<WitnessChain>, FactError> {
        let facts = self.facts::<TextareaMustacheDemand>();
        let Some(counterexample) = facts.get::<NativeTextareaMustache>()?.get(&child) else {
            return Ok(None);
        };
        let Some(header) = facts
            .get::<NativeLintHeaders>()?
            .get(&counterexample.header)
        else {
            return Ok(None);
        };
        let Some(marker) = facts.get::<NativeDirectInterpolations>()?.get(&child) else {
            return Ok(None);
        };
        Ok(Some(
            WitnessChain::new(WitnessLink::of::<NativeLintHeaders>(
                &counterexample.header,
                header.span(),
            ))
            .then(WitnessLink::of::<NativeDirectInterpolations>(
                &child,
                marker.span,
            ))
            .then(WitnessLink::of::<NativeTextareaMustache>(
                &child,
                counterexample.span,
            )),
        ))
    }
    pub fn verify(&self, chain: &WitnessChain) -> Result<(), WitnessError> {
        verify_chain(chain, &self.facts::<TextareaMustacheDemand>(), &CHECKS)
    }
}

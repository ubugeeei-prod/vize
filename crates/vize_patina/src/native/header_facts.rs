//! SDK facts from one original selected authored header.
//!
//! The evidence constructor is sealed and borrows the authentic selected owner.
//! Exact authored lint grammar is retained; DOM membership and runtime component
//! resolution are not inferred. This provider emits no diagnostic.

use vize_l0::{
    Span, String,
    diag::{
        WitnessChain, WitnessLink,
        verify::{WitnessError, verify_chain},
    },
    fact::{FactConsumer, FactError, FactManager, FactTable, FactView},
};
use vize_l1::{
    Element,
    markup::{NativeElement, NativeTemplateComponent},
};

use super::{NativeLintRefusal, header, header_rules};

mod groups;
mod observations;

pub use groups::{
    NativeLintAttributes, NativeLintHeaders, NativeUnsupportedAria, UnsupportedAriaDemand,
};
pub use observations::{
    NativeAttributeFact, NativeBindingKind, NativeHeaderFact, NativeUnsupportedAriaFact,
};

use groups::{CHECKS, REGISTRY};

/// Header admission and SDK computation preserve their distinct exact errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeHeaderFactError {
    Header(NativeLintRefusal),
    Fact(FactError),
}

impl From<NativeLintRefusal> for NativeHeaderFactError {
    fn from(error: NativeLintRefusal) -> Self {
        Self::Header(error)
    }
}
impl From<FactError> for NativeHeaderFactError {
    fn from(error: FactError) -> Self {
        Self::Fact(error)
    }
}

// No public constructor can populate this artifact, its keys or observations.
struct Evidence {
    header_key: u32,
    header: NativeHeaderFact,
    attributes: Vec<(u32, NativeAttributeFact)>,
}

/// Non-cloneable, original-owner-bound SDK tables for one completely checked
/// authored header. The source, profile and actual element remain borrowed.
///
/// ```compile_fail
/// use vize_patina::native::header_facts::NativeHeaderFacts;
/// fn cloneable<T: Clone>() {}
/// cloneable::<NativeHeaderFacts<'static, 'static>>();
/// ```
pub struct NativeHeaderFacts<'o, 'a> {
    owner: &'o NativeTemplateComponent<'a>,
    original: &'o Element<'a>,
    manager: FactManager<'static, Evidence>,
}

impl<'o, 'a> NativeHeaderFacts<'o, 'a> {
    pub(super) fn new(
        owner: &'o NativeTemplateComponent<'a>,
        element: &NativeElement<'o, 'a>,
    ) -> Result<Self, NativeHeaderFactError> {
        let header_key =
            u32::try_from(element.ordinal()).map_err(|_| NativeLintRefusal::SourceMismatch)?;
        let mut attributes = Vec::with_capacity(element.attributes().len());
        let mut bad_ordinal = false;
        let (receipt, opening) = header_rules::inspect_attributes(element, |original, binding| {
            let Ok(key) = u32::try_from(original.ordinal()) else {
                bad_ordinal = true;
                return;
            };
            let (kind, name, span) = match binding {
                header::Binding::Static { name, range, .. } => {
                    (NativeBindingKind::Static, name, range)
                }
                header::Binding::Bind { name, range } => (NativeBindingKind::Bind, name, range),
                header::Binding::Other { range } => (
                    NativeBindingKind::Other,
                    original.surface().name.text,
                    range,
                ),
            };
            attributes.push((
                key,
                NativeAttributeFact {
                    name: String::new(name),
                    span,
                    kind,
                },
            ));
        })?;
        if bad_ordinal {
            return Err(NativeLintRefusal::SourceMismatch.into());
        }
        let evidence = Evidence {
            header_key,
            header: NativeHeaderFact {
                tag: String::new(element.surface().tag()),
                span: receipt.span(),
                opening,
                kind: receipt.kind(),
                literal: receipt.header_is_literal(),
            },
            attributes,
        };
        let mut manager = FactManager::new(&REGISTRY);
        manager.prepare::<UnsupportedAriaDemand>(&evidence)?;
        Ok(Self {
            owner,
            original: element.surface(),
            manager,
        })
    }

    #[must_use]
    pub fn owner(&self) -> &'o NativeTemplateComponent<'a> {
        self.owner
    }

    #[must_use]
    pub fn original(&self) -> &'o Element<'a> {
        self.original
    }

    /// Every read uses the caller's actual declared SDK demand.
    #[must_use]
    pub fn facts<C: FactConsumer>(&self) -> FactView<'_> {
        self.manager.view::<C>()
    }

    pub fn unsupported_aria(&self) -> Result<&FactTable<NativeUnsupportedAria>, FactError> {
        self.facts::<UnsupportedAriaDemand>()
            .get::<NativeUnsupportedAria>()
    }

    /// Build a causal chain only for a stored counterexample, using the actual
    /// original tag and full attribute ranges. A nonmatching key returns None.
    pub fn unsupported_aria_chain(
        &self,
        attribute: u32,
    ) -> Result<Option<WitnessChain>, FactError> {
        let facts = self.facts::<UnsupportedAriaDemand>();
        let Some(counterexample) = facts.get::<NativeUnsupportedAria>()?.get(&attribute) else {
            return Ok(None);
        };
        let Some(header) = facts
            .get::<NativeLintHeaders>()?
            .get(&counterexample.header)
        else {
            return Ok(None);
        };
        let Some(original) = facts.get::<NativeLintAttributes>()?.get(&attribute) else {
            return Ok(None);
        };
        Ok(Some(
            WitnessChain::new(WitnessLink::of::<NativeLintHeaders>(
                &counterexample.header,
                header.span,
            ))
            .then(WitnessLink::of::<NativeLintAttributes>(
                &attribute,
                original.span,
            ))
            .then(WitnessLink::of::<NativeUnsupportedAria>(
                &attribute,
                counterexample.span,
            )),
        ))
    }

    /// Re-read each exact key and span through the real SDK witness verifier.
    pub fn verify(&self, chain: &WitnessChain) -> Result<(), WitnessError> {
        verify_chain(chain, &self.facts::<UnsupportedAriaDemand>(), &CHECKS)
    }
}

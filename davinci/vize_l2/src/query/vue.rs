//! Vue meaning attached to the File that observed it. A consumer may request
//! bindings, component contracts or instance options without a second AST walk.

use super::{BindingRef, Model};
use crate::file::{L2Ir, Namespace, ScopeId, ScriptUnitId};
pub use crate::framework::vue::{
    CallContract, ComponentContract, ComponentOptions, ComponentShape, ContractSource, Emit,
    EmitsContract, MacroCall, MacroKind, ModelContract, ModelName, OptionKey, OptionMember,
    OptionMemberKind, OptionsFactsIncomplete, OrdinaryFactsIncomplete, Prop, PropsContract,
    ReactiveSource, SetupBinding, SetupBindingKind, SetupFactsIncomplete, SlotsContract,
};
use crate::framework::vue::{OptionsFacts, OrdinaryFacts, SetupFacts};
use alloc::boxed::Box;
use core::ops::Deref;
use vize_l0::Span;

mod producer;
#[cfg(test)]
mod tests;
pub use producer::{OrdinaryDemand, VueDemand, VueFileProducer, VueProgramError};

/// A fact family a consumer can request from the original walks.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VueFamily {
    BindingKind,
    Setup,
    OrdinaryBindings,
    Options,
}

/// Failure to certify a Vue answer for this File and requested family.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VueQueryError {
    IncompleteFile,
    ForeignFile,
    NotObserved(VueFamily),
    SetupIncomplete(SetupFactsIncomplete),
    OrdinaryIncomplete(OrdinaryFactsIncomplete),
    OptionsIncomplete(OptionsFactsIncomplete),
    /// The producer retained an authored pattern it could not classify.
    Unclassified(Span),
    /// More than one classification covers the original declaration.
    Ambiguous(Span),
}

struct Observed<F> {
    unit: ScriptUnitId,
    scope: ScopeId,
    facts: F,
}
#[derive(Default)]
struct VueFacts {
    setup: Option<Box<Observed<Result<SetupFacts, SetupFactsIncomplete>>>>,
    ordinary: Option<Box<Observed<OrdinaryFamilies>>>,
}
struct OrdinaryFamilies {
    bindings: Option<Result<OrdinaryFacts, OrdinaryFactsIncomplete>>,
    options: Option<Result<OptionsFacts, OptionsFactsIncomplete>>,
}

/// An immutable File plus the fact families requested from its own producer.
pub struct VueFile<'a> {
    file: L2Ir<'a>,
    facts: VueFacts,
}
impl<'a> VueFile<'a> {
    #[must_use]
    pub fn file(&self) -> &L2Ir<'a> {
        &self.file
    }
    #[must_use]
    pub fn model(&self) -> VueModel<'_, 'a> {
        VueModel {
            model: Model::new(&self.file),
            facts: Some(&self.facts),
        }
    }
    #[must_use]
    pub fn into_file(self) -> L2Ir<'a> {
        self.file
    }
}

/// The borrowed Vue and ordinary semantic queries for one File.
#[derive(Clone, Copy)]
pub struct VueModel<'f, 'a> {
    model: Model<'f, 'a>,
    facts: Option<&'f VueFacts>,
}

impl<'f, 'a> Deref for VueModel<'f, 'a> {
    type Target = Model<'f, 'a>;
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl<'f, 'a> Model<'f, 'a> {
    /// A File for which no framework observers were requested.
    #[must_use]
    pub fn vue(self) -> VueModel<'f, 'a> {
        VueModel {
            model: self,
            facts: None,
        }
    }
    /// Attach only an observation belonging to this exact immutable File.
    pub fn with_vue(self, owner: &'f VueFile<'a>) -> Result<VueModel<'f, 'a>, VueQueryError> {
        if !core::ptr::eq(self.file(), owner.file()) {
            return Err(VueQueryError::ForeignFile);
        }
        Ok(owner.model())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VueBinding {
    Setup(SetupBinding),
    Ordinary(SetupBinding),
}

impl<'f, 'a> VueModel<'f, 'a> {
    fn complete(self) -> Result<(), VueQueryError> {
        if self.model.file().is_complete() {
            Ok(())
        } else {
            Err(VueQueryError::IncompleteFile)
        }
    }

    fn setup(self) -> Result<&'f SetupFacts, VueQueryError> {
        self.complete()?;
        let observed = self
            .facts
            .and_then(|facts| facts.setup.as_ref())
            .ok_or(VueQueryError::NotObserved(VueFamily::Setup))?;
        observed
            .facts
            .as_ref()
            .map_err(|reason| VueQueryError::SetupIncomplete(*reason))
    }

    /// Macro calls in source order, including the inner call of withDefaults.
    pub fn macro_calls(self) -> Result<&'f [MacroCall], VueQueryError> {
        Ok(self.setup()?.macro_calls())
    }

    /// Props, emits, slots, models and expose belong to the original setup Program.
    pub fn contract(self) -> Result<&'f ComponentContract, VueQueryError> {
        Ok(self.setup()?.contract())
    }

    /// Instance members carry their actual declaring option, class or mixin.
    pub fn options(self) -> Result<Option<&'f ComponentOptions>, VueQueryError> {
        self.complete()?;
        let facts = self
            .facts
            .and_then(|f| f.ordinary.as_ref())
            .and_then(|o| o.facts.options.as_ref())
            .ok_or(VueQueryError::NotObserved(VueFamily::Options))?;
        Ok(facts
            .as_ref()
            .map_err(|reason| VueQueryError::OptionsIncomplete(*reason))?
            .component())
    }

    /// Classify the exact lexical declaration identified by an owner-bound
    /// binding. Type names, inner scopes and template aliases have no top-level
    /// script classification. A missing observer is a typed refusal.
    pub fn binding_kind(
        self,
        binding: BindingRef<'_, '_>,
    ) -> Result<Option<VueBinding>, VueQueryError> {
        if !core::ptr::eq(self.model.file(), binding.file()) {
            return Err(VueQueryError::ForeignFile);
        }
        self.complete()?;
        let Some(decl) = binding.declaration() else {
            return Ok(None);
        };
        if decl.namespace != Namespace::Value {
            return Ok(None);
        }
        let setup = self.facts.and_then(|f| f.setup.as_ref());
        if let Some(observed) = setup.filter(|o| o.unit == decl.unit) {
            if observed.scope != decl.scope {
                return Ok(None);
            }
            let facts = observed
                .facts
                .as_ref()
                .map_err(|reason| VueQueryError::SetupIncomplete(*reason))?;
            if let Some(span) = facts
                .unclassified()
                .iter()
                .find(|s| s.start <= decl.span.start && decl.span.end <= s.end)
            {
                return Err(VueQueryError::Unclassified(*span));
            }
            return original(facts.bindings(), decl.span).map(|r| r.map(VueBinding::Setup));
        }
        let ordinary = self
            .facts
            .and_then(|f| f.ordinary.as_ref())
            .filter(|o| o.unit == decl.unit)
            .ok_or(VueQueryError::NotObserved(VueFamily::BindingKind))?;
        if ordinary.scope != decl.scope {
            return Ok(None);
        }
        let facts = ordinary
            .facts
            .bindings
            .as_ref()
            .ok_or(VueQueryError::NotObserved(VueFamily::OrdinaryBindings))?;
        let facts = facts
            .as_ref()
            .map_err(|reason| VueQueryError::OrdinaryIncomplete(*reason))?;
        original(facts.bindings(), decl.span).map(|r| r.map(VueBinding::Ordinary))
    }
}

fn original(rows: &[SetupBinding], span: Span) -> Result<Option<SetupBinding>, VueQueryError> {
    // The observed rows are in authored order. Multiple namespace declarations
    // and aliases are resolved by the original File before this exact-span join.
    let start = rows.partition_point(|row| row.name.start < span.start);
    let mut candidates = rows
        .get(start..)
        .unwrap_or_default()
        .iter()
        .take_while(|r| r.name.start == span.start)
        .filter(|r| r.name == span);
    let found = candidates.next().copied();
    if candidates.next().is_some() {
        Err(VueQueryError::Ambiguous(span))
    } else {
        Ok(found)
    }
}

#[cfg(feature = "script-queries")]
mod script_programs;

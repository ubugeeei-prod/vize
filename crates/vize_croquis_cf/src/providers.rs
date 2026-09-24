//! App-level fact providers — the P4-10a provider contract.
//!
//! Not everything a Vue project means lives inside one file: the route tree a
//! `createRouter({ routes })` call declares, `definePageMeta`, i18n catalogs.
//! A **provider** turns such a convention into ordinary fact groups on the
//! P4-1 fact API ([`vize_davinci::fact`]), so a consumer demands "the route
//! params of `user-posts`" exactly the way it demands bindings or scopes.
//!
//! # The contract (`docs/davinci/open-questions.md`, settled here)
//!
//! A provider is **neither a new plug-in kind nor a fact consumer with write
//! access**. It is a *project-level population pass* in the demand graph:
//!
//! - it declares its **ambient inputs** ([`Provider::INPUTS`]) — the modules,
//!   globs and config it reads. That list is exactly what P5-1's key manifest
//!   folds into the artifact key, so a provider's facts invalidate when, and
//!   only when, something it reads changes;
//! - it **exclusively owns** its output groups ([`Provider::OUTPUTS`]). Every
//!   group has one writer: [`ProviderRegistry::new`] rejects a second
//!   provider claiming a group with [`ProviderError::SecondWriter`] — in a
//!   `const` item, at compile time;
//! - its groups are registered producers over the project artifact
//!   ([`ProjectSources`]) in [`PROJECT_FACTS`], so consumers declare them in a
//!   [`FactConsumer::DEMAND`](vize_davinci::fact::FactConsumer::DEMAND) and
//!   the TS-35 detector polices every read.
//!
//! In-tree providers compile in behind cargo features (charter #15's first
//! tier: `provider-vue-router`); external providers reach the same interface
//! through charter #29's "custom fact providers" hook family (P6-7).
//!
//! # Identity
//!
//! Project-level groups take ids from the block reserved in [`ids`], disjoint
//! from the per-artifact groups the P4-3 waves register, so a witness link's
//! group id names one fact group across the whole program.

pub mod project;
#[cfg(feature = "provider-vue-router")]
pub mod vue_router;

pub use project::{
    ModuleId, ModuleKind, ProjectModule, ProjectSources, ScriptBlock, TemplateBlock,
};

use vize_davinci::fact::{Demand, FactRegistry, ProducerEntry};
use vize_davinci::pass::AnalysisId;

/// Fact-group ids of the project-level (provider-owned) groups.
///
/// `40..48` is reserved for providers; per-artifact groups (P4-3) allocate
/// from the bottom of the id space.
pub mod ids {
    use vize_davinci::pass::AnalysisId;

    /// `vue-router` — one statically extracted route tree per router.
    pub const ROUTE_TREE: AnalysisId = AnalysisId::new(40);
    /// `vue-router` — the params every named route takes.
    pub const ROUTE_PARAMS: AnalysisId = AnalysisId::new(41);
}

/// Something a provider reads besides its own code: the P5-1 key-manifest
/// entry for its groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmbientInput {
    /// Every script module the host supplies (the `.ts`/`.js` family) and
    /// every SFC `<script>` block.
    ScriptModules,
    /// Every SFC `<template>` block.
    SfcTemplates,
    /// The files matching a project-relative glob.
    Glob(&'static str),
    /// One project configuration file, by project-relative path.
    Config(&'static str),
}

/// A provider: a project-level population pass that exclusively owns the
/// fact groups it writes.
pub trait Provider {
    /// The provider's stable name, used in registry errors.
    const NAME: &'static str;
    /// Everything the provider reads — never empty (a provider that reads
    /// nothing would never invalidate).
    const INPUTS: &'static [AmbientInput];
    /// The groups the provider writes, and no other provider may.
    const OUTPUTS: Demand;

    /// This provider as const registry data.
    const DESC: ProviderDesc = ProviderDesc {
        name: Self::NAME,
        inputs: Self::INPUTS,
        outputs: Self::OUTPUTS,
    };
}

/// One provider as const data — what the single-writer check reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderDesc {
    /// The provider's stable name.
    pub name: &'static str,
    /// Its ambient inputs.
    pub inputs: &'static [AmbientInput],
    /// The groups it owns.
    pub outputs: Demand,
}

/// Why a set of providers is not a valid registry. Every variant names the
/// provider(s) and group, so a test compares the exact value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderError {
    /// A provider declares no ambient inputs.
    NoInputs { provider: &'static str },
    /// A provider declares no output groups.
    NoOutputs { provider: &'static str },
    /// A provider claims a group no registered producer computes.
    UnproducedOutput {
        provider: &'static str,
        group: AnalysisId,
    },
    /// Two providers claim the same group — the single-writer rule.
    SecondWriter {
        group: AnalysisId,
        owner: &'static str,
        writer: &'static str,
    },
}

/// The single-writer check: every provider reads something, writes
/// something, writes only registered groups, and shares no group with an
/// earlier provider.
///
/// A `const fn`, so [`ProviderRegistry::new`] runs it at compile time;
/// exposed on its own so the exact error of a rejected set can be asserted.
///
/// # Errors
///
/// The first violation in registration order.
pub const fn check_providers(
    providers: &[ProviderDesc],
    registered: Demand,
) -> Result<(), ProviderError> {
    let mut rest = providers;
    let mut i = 0;
    while let [provider, tail @ ..] = rest {
        if provider.inputs.is_empty() {
            return Err(ProviderError::NoInputs {
                provider: provider.name,
            });
        }
        if provider.outputs.is_empty() {
            return Err(ProviderError::NoOutputs {
                provider: provider.name,
            });
        }
        let unproduced = provider.outputs.minus(registered);
        if !unproduced.is_empty() {
            return Err(ProviderError::UnproducedOutput {
                provider: provider.name,
                group: first(unproduced),
            });
        }
        let mut earlier = providers;
        let mut j = 0;
        while j < i
            && let [owner, more @ ..] = earlier
        {
            let shared = owner.outputs.intersect(provider.outputs);
            if !shared.is_empty() {
                return Err(ProviderError::SecondWriter {
                    group: first(shared),
                    owner: owner.name,
                    writer: provider.name,
                });
            }
            earlier = more;
            j += 1;
        }
        rest = tail;
        i += 1;
    }
    Ok(())
}

const fn first(set: Demand) -> AnalysisId {
    AnalysisId::new(set.to_bits().trailing_zeros() as u8)
}

/// The groups `producers` compute, in `const` context.
#[must_use]
pub const fn produced<A: ?Sized>(producers: &[ProducerEntry<A>]) -> Demand {
    let mut set = Demand::NONE;
    let mut rest = producers;
    while let [producer, tail @ ..] = rest {
        set = set.with(producer.desc.id);
        rest = tail;
    }
    set
}

/// Every provider over artifacts of type `A`, checked for single writers
/// against the fact registry that runs their producers.
///
/// Build it in a `const` item that unwraps [`ProviderRegistry::new`] with a
/// `panic!` on `Err`: in `const` evaluation that is a compile error, never a
/// runtime abort. A second writer to the route tree does not compile:
///
/// ```compile_fail,E0080
/// use vize_croquis_cf::providers::{
///     AmbientInput, PROJECT_FACTS, ProjectSources, Provider, ProviderRegistry,
///     vue_router::{RouteTree, VueRouterProvider},
/// };
/// use vize_davinci::fact::{Demand, FactGroup};
///
/// struct FileRoutes;
/// impl Provider for FileRoutes {
///     const NAME: &'static str = "file-routes";
///     const INPUTS: &'static [AmbientInput] = &[AmbientInput::Glob("src/pages/**/*.vue")];
///     const OUTPUTS: Demand = Demand::NONE.with(RouteTree::ID);
/// }
/// const REGISTRY: ProviderRegistry<ProjectSources> =
///     match ProviderRegistry::new(&[VueRouterProvider::DESC, FileRoutes::DESC], &PROJECT_FACTS) {
///         Ok(registry) => registry,
///         Err(_) => panic!("single writer"),
///     };
/// ```
///
/// Its twin, the same registry without the second writer, builds — which
/// pins the failure above to the shared group:
///
/// ```
/// use vize_croquis_cf::providers::{
///     PROJECT_FACTS, ProjectSources, Provider, ProviderRegistry, vue_router::VueRouterProvider,
/// };
///
/// const REGISTRY: ProviderRegistry<ProjectSources> =
///     match ProviderRegistry::new(&[VueRouterProvider::DESC], &PROJECT_FACTS) {
///         Ok(registry) => registry,
///         Err(_) => panic!("single writer"),
///     };
/// assert_eq!(REGISTRY.providers().len(), 1);
/// ```
pub struct ProviderRegistry<A: ?Sized + 'static> {
    providers: &'static [ProviderDesc],
    facts: &'static FactRegistry<A>,
}

impl<A: ?Sized + 'static> ProviderRegistry<A> {
    /// A registry of `providers` whose groups `facts` produces.
    ///
    /// Unwrap it in a `const` item so a rejected set is a compile error.
    ///
    /// # Errors
    ///
    /// The first [`ProviderError`] of [`check_providers`].
    pub const fn new(
        providers: &'static [ProviderDesc],
        facts: &'static FactRegistry<A>,
    ) -> Result<Self, ProviderError> {
        match check_providers(providers, produced(facts.producers())) {
            Ok(()) => Ok(Self { providers, facts }),
            Err(error) => Err(error),
        }
    }

    /// The registered providers, in registration order.
    #[must_use]
    pub const fn providers(&self) -> &'static [ProviderDesc] {
        self.providers
    }

    /// The fact registry that runs the providers' producers.
    #[must_use]
    pub const fn facts(&self) -> &'static FactRegistry<A> {
        self.facts
    }

    /// The provider that owns `group`, if any.
    #[must_use]
    pub fn owner(&self, group: AnalysisId) -> Option<&'static ProviderDesc> {
        self.providers
            .iter()
            .find(|provider| provider.outputs.contains(group))
    }

    /// The ambient inputs a run demanding `demand` reads: every input of
    /// every provider owning a group in the demand's closure, in provider
    /// order, without repeats — the P5-1 key manifest of that demand.
    #[must_use]
    pub fn inputs(&self, demand: Demand) -> Vec<AmbientInput> {
        let closure = self.facts.closure(demand);
        let mut inputs = Vec::new();
        for provider in self.providers {
            if provider.outputs.intersect(closure).is_empty() {
                continue;
            }
            for input in provider.inputs {
                if !inputs.contains(input) {
                    inputs.push(*input);
                }
            }
        }
        inputs
    }
}

/// Every project-level fact group, as registered producers over
/// [`ProjectSources`].
pub const PROJECT_FACTS: FactRegistry<ProjectSources> = FactRegistry::new(&[
    #[cfg(feature = "provider-vue-router")]
    ProducerEntry::of::<vue_router::RouteTree>(),
    #[cfg(feature = "provider-vue-router")]
    ProducerEntry::of::<vue_router::RouteParams>(),
]);

/// Every in-tree provider, single-writer checked against [`PROJECT_FACTS`].
pub const PROVIDERS: ProviderRegistry<ProjectSources> = match ProviderRegistry::new(
    &[
        #[cfg(feature = "provider-vue-router")]
        <vue_router::VueRouterProvider as Provider>::DESC,
    ],
    &PROJECT_FACTS,
) {
    Ok(registry) => registry,
    Err(_) => panic!("provider registry: the in-tree providers violate single writer"),
};

#[cfg(test)]
mod tests;

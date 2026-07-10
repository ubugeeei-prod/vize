//! Source storage, dependency planning, execution, memoization, and invalidation.

mod execution;
mod snapshot;

pub use snapshot::CompilationSnapshot;

use vize_carton::{FxHashMap, FxHashSet};

use crate::{
    ArtifactCache, CompilationInput, CompilationInputError, CompilationInputs, ExecutionCounters,
    InputId, Plan, PlanError, Product, ProductId, ProductRequest, Provider, ProviderId,
    RegisterProviderError, Shared, SourceError, SourceId, SourceRange, SourceSnapshot, SourceStore,
    invalidation::{InputInvalidationReport, InvalidationReport},
    provider::{ErasedProvider, ProviderAdapter},
};

#[derive(Clone)]
pub(crate) struct ProviderEntry {
    pub(crate) id: ProviderId,
    pub(crate) provider: Shared<dyn ErasedProvider>,
}

/// One independent artifact graph, source store, provider registry, and cache.
#[derive(Default)]
pub struct Compilation {
    sources: SourceStore,
    providers: FxHashMap<ProductId, Vec<ProviderEntry>>,
    provider_generation: u64,
    inputs: CompilationInputs,
    cache: ArtifactCache,
    counters: ExecutionCounters,
}

impl Compilation {
    /// Build an empty compilation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register one independently applicable provider for its typed product.
    ///
    /// Multiple concrete provider types may target the same product. Planning
    /// selects exactly one through [`Provider::supports`]. Registering the same
    /// concrete provider type twice is rejected.
    pub fn register_provider<T: Provider>(
        &mut self,
        provider: T,
    ) -> Result<(), RegisterProviderError> {
        let product = ProductId::of::<T::Product>();
        let provider_id = ProviderId::of::<T>();
        if self
            .providers
            .get(&product)
            .is_some_and(|entries| entries.iter().any(|entry| entry.id == provider_id))
        {
            return Err(RegisterProviderError::DuplicateProvider {
                provider: provider_id,
                product,
            });
        }
        let next_generation = self
            .provider_generation
            .checked_add(1)
            .ok_or(RegisterProviderError::ProviderGenerationExhausted)?;
        self.providers
            .entry(product)
            .or_default()
            .push(ProviderEntry {
                id: provider_id,
                provider: Shared::new(ProviderAdapter::new(provider)),
            });
        self.provider_generation = next_generation;
        Ok(())
    }

    /// Whether a provider for `P` is registered.
    pub fn has_provider<P: Product>(&self) -> bool {
        self.providers
            .get(&ProductId::of::<P>())
            .is_some_and(|entries| !entries.is_empty())
    }

    /// Install or replace an open typed compilation input.
    pub fn set_input<I: CompilationInput>(
        &mut self,
        value: I::Value,
    ) -> Result<InputInvalidationReport, CompilationInputError> {
        let input = InputId::of::<I>();
        let replaced = self.inputs.insert::<I>(value)?;
        let evicted = self.cache.evict_input(input);
        Ok(InputInvalidationReport::new(input, replaced, evicted))
    }

    pub const fn inputs(&self) -> &CompilationInputs {
        &self.inputs
    }

    pub fn input<I: CompilationInput>(&self) -> Option<&I::Value> {
        self.inputs.get::<I>()
    }

    /// Inspect the revision-keyed artifact cache.
    pub const fn cache(&self) -> &ArtifactCache {
        &self.cache
    }

    /// Add an independently supplied source.
    pub fn add_source(
        &mut self,
        name: impl Into<Shared<str>>,
        text: impl Into<Shared<str>>,
    ) -> Result<SourceId, SourceError> {
        self.sources.add(name, text)
    }

    /// Add a source with a precise provenance edge into an existing parent.
    pub fn add_embedded_source(
        &mut self,
        parent: SourceId,
        range: SourceRange,
        name: impl Into<Shared<str>>,
        text: impl Into<Shared<str>>,
    ) -> Result<SourceId, SourceError> {
        self.sources.add_embedded(parent, range, name, text)
    }

    pub const fn sources(&self) -> &SourceStore {
        &self.sources
    }

    pub fn source(&self, source: SourceId) -> Option<&SourceSnapshot> {
        self.sources.get(source)
    }

    /// Replace source text while preserving its identity.
    pub fn update_source(
        &mut self,
        source: SourceId,
        text: impl Into<Shared<str>>,
    ) -> Result<InvalidationReport, SourceError> {
        self.update_source_inner(source, text.into(), None)
    }

    /// Refresh an embedded source's text and provenance range together.
    pub fn update_embedded_source(
        &mut self,
        source: SourceId,
        range: SourceRange,
        text: impl Into<Shared<str>>,
    ) -> Result<InvalidationReport, SourceError> {
        self.update_source_inner(source, text.into(), Some(range))
    }

    fn update_source_inner(
        &mut self,
        source: SourceId,
        text: Shared<str>,
        range: Option<SourceRange>,
    ) -> Result<InvalidationReport, SourceError> {
        let mutation = self.sources.update(source, text, range)?;
        let affected: FxHashSet<_> = mutation
            .changes
            .iter()
            .map(|change| change.source)
            .collect();
        let evicted = self.cache.evict_sources(&affected);
        Ok(InvalidationReport::new(source, mutation.changes, evicted))
    }

    /// Build a plan for one typed product without executing any provider.
    pub fn plan_for<P: Product>(&self, source: SourceId) -> Result<Plan, PlanError> {
        self.plan(source, [ProductId::of::<P>()])
    }

    /// Plan exactly the transitive closure of same-source roots.
    pub fn plan(
        &self,
        source: SourceId,
        roots: impl IntoIterator<Item = ProductId>,
    ) -> Result<Plan, PlanError> {
        self.plan_requests(
            roots
                .into_iter()
                .map(|product| ProductRequest::new(source, product)),
        )
    }

    /// Plan the transitive closure of roots spanning one or more sources.
    pub fn plan_requests(
        &self,
        roots: impl IntoIterator<Item = ProductRequest>,
    ) -> Result<Plan, PlanError> {
        crate::planner::build_plan(
            &self.sources,
            &self.providers,
            &self.inputs,
            self.provider_generation,
            roots,
        )
    }

    /// Aggregate observations since this compilation was created.
    pub const fn counters(&self) -> &ExecutionCounters {
        &self.counters
    }
}

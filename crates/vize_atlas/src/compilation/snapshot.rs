//! Immutable compilation snapshots and isolated forks.

use vize_carton::FxHashMap;

use super::{Compilation, ProviderEntry};
use crate::{
    ArtifactCache, CompilationInputs, ExecutionCounters, Plan, PlanError, Product, ProductId,
    ProductRequest, Shared, SourceId, SourceSnapshot, SourceStore,
};

/// Cheaply cloned immutable view of a compilation's query state.
///
/// Creating a snapshot copies the current maps once. Cloning the resulting
/// value is an atomic reference-count increment. [`CompilationSnapshot::fork`]
/// creates an isolated mutable compilation that can be queried or edited
/// without changing the snapshot or the compilation from which it came.
#[derive(Clone)]
pub struct CompilationSnapshot {
    state: Shared<CompilationSnapshotState>,
}

struct CompilationSnapshotState {
    sources: SourceStore,
    providers: FxHashMap<ProductId, Vec<ProviderEntry>>,
    provider_generation: u64,
    inputs: CompilationInputs,
    cache: ArtifactCache,
}

impl CompilationSnapshot {
    /// Sources and exact revisions captured by this snapshot.
    pub fn sources(&self) -> &SourceStore {
        &self.state.sources
    }

    /// Look up one captured source, including embedded-source provenance.
    pub fn source(&self, source: SourceId) -> Option<&SourceSnapshot> {
        self.state.sources.get(source)
    }

    /// Typed compilation inputs captured by this snapshot.
    pub fn inputs(&self) -> &CompilationInputs {
        &self.state.inputs
    }

    /// Cached artifacts captured by this snapshot.
    pub fn cache(&self) -> &ArtifactCache {
        &self.state.cache
    }

    /// Plan a typed request against the immutable snapshot.
    pub fn plan_for<P: Product>(&self, source: SourceId) -> Result<Plan, PlanError> {
        self.plan(source, [ProductId::of::<P>()])
    }

    /// Plan same-source products against the immutable snapshot.
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

    /// Plan complete source/product requests against the snapshot.
    pub fn plan_requests(
        &self,
        roots: impl IntoIterator<Item = ProductRequest>,
    ) -> Result<Plan, PlanError> {
        crate::planner::build_plan(
            &self.state.sources,
            &self.state.providers,
            &self.state.inputs,
            self.state.provider_generation,
            roots,
        )
    }

    /// Create an isolated mutable compilation from this immutable state.
    pub fn fork(&self) -> Compilation {
        Compilation {
            sources: self.state.sources.clone(),
            providers: self.state.providers.clone(),
            provider_generation: self.state.provider_generation,
            inputs: self.state.inputs.clone(),
            cache: self.state.cache.clone(),
            counters: ExecutionCounters::default(),
        }
    }
}

impl Compilation {
    /// Capture sources, inputs, providers, and cache as immutable query state.
    pub fn snapshot(&self) -> CompilationSnapshot {
        CompilationSnapshot {
            state: Shared::new(CompilationSnapshotState {
                sources: self.sources.clone(),
                providers: self.providers.clone(),
                provider_generation: self.provider_generation,
                inputs: self.inputs.clone(),
                cache: self.cache.clone(),
            }),
        }
    }
}

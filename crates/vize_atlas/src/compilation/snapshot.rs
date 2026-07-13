//! Immutable compilation snapshots, shared query sessions, and isolated forks.

use vize_carton::FxHashMap;

use super::{Compilation, ProviderEntry};
use crate::{
    ArtifactCache, CompilationInputs, ExecutionCounters, ExecutionOutcome, Plan, PlanError,
    Product, ProductId, ProductRequest, QueryError, QueryOutcome, Shared, SourceId, SourceSnapshot,
    SourceStore,
};

/// Cheaply cloned immutable view of a compilation's query state.
///
/// Creating a snapshot copies the current maps once. Cloning the resulting
/// value is an atomic reference-count increment. [`CompilationSnapshot::fork`]
/// creates an isolated mutable compilation that can be queried or edited
/// without changing the snapshot or the compilation from which it came.
/// [`CompilationSnapshot::query_session`] instead creates a read-only worker
/// that shares memoized products with the snapshot's other query sessions.
#[derive(Clone)]
pub struct CompilationSnapshot {
    state: Shared<CompilationSnapshotState>,
}

struct CompilationSnapshotState {
    sources: SourceStore,
    providers: FxHashMap<ProductId, Vec<ProviderEntry>>,
    provider_generation: u64,
    inputs: CompilationInputs,
    cache: Shared<ArtifactCache>,
}

/// One query-only worker over an immutable [`CompilationSnapshot`].
///
/// Sessions have independent counters and can be moved to different threads.
/// Their source, input, and provider state cannot be mutated through this API,
/// while successful memoized products immediately become visible to sibling
/// sessions created from the same snapshot.
pub struct QuerySession {
    compilation: Compilation,
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

    /// Create a query-only worker sharing this snapshot's memoization cache.
    pub fn query_session(&self) -> QuerySession {
        QuerySession {
            compilation: Compilation {
                sources: self.state.sources.clone(),
                providers: self.state.providers.clone(),
                provider_generation: self.state.provider_generation,
                inputs: self.state.inputs.clone(),
                cache: Shared::clone(&self.state.cache),
                counters: ExecutionCounters::default(),
            },
        }
    }

    /// Create an isolated mutable compilation from this immutable state.
    pub fn fork(&self) -> Compilation {
        Compilation {
            sources: self.state.sources.clone(),
            providers: self.state.providers.clone(),
            provider_generation: self.state.provider_generation,
            inputs: self.state.inputs.clone(),
            cache: Shared::new(self.state.cache.as_ref().clone()),
            counters: ExecutionCounters::default(),
        }
    }
}

impl QuerySession {
    /// Sources and exact revisions captured for this query worker.
    pub fn sources(&self) -> &SourceStore {
        self.compilation.sources()
    }

    /// Look up one captured source, including embedded-source provenance.
    pub fn source(&self, source: SourceId) -> Option<&SourceSnapshot> {
        self.compilation.source(source)
    }

    /// Typed compilation inputs captured for this query worker.
    pub fn inputs(&self) -> &CompilationInputs {
        self.compilation.inputs()
    }

    /// Inspect the cache shared with sibling sessions from the same snapshot.
    pub fn cache(&self) -> &ArtifactCache {
        self.compilation.cache()
    }

    /// Plan one typed request against the immutable captured state.
    pub fn plan_for<P: Product>(&self, source: SourceId) -> Result<Plan, PlanError> {
        self.compilation.plan_for::<P>(source)
    }

    /// Plan same-source products against the immutable captured state.
    pub fn plan(
        &self,
        source: SourceId,
        roots: impl IntoIterator<Item = ProductId>,
    ) -> Result<Plan, PlanError> {
        self.compilation.plan(source, roots)
    }

    /// Plan complete source/product requests against the captured state.
    pub fn plan_requests(
        &self,
        roots: impl IntoIterator<Item = ProductRequest>,
    ) -> Result<Plan, PlanError> {
        self.compilation.plan_requests(roots)
    }

    /// Execute a plan without exposing any source, input, or provider mutation.
    pub fn execute(&mut self, plan: Plan) -> Result<ExecutionOutcome, QueryError> {
        self.compilation.execute(plan)
    }

    /// Plan, execute, and return one strongly typed root product.
    pub fn query<P: Product>(&mut self, source: SourceId) -> Result<QueryOutcome<P>, QueryError> {
        self.compilation.query::<P>(source)
    }

    /// Counters local to this session, excluding work done by its siblings.
    pub fn counters(&self) -> &ExecutionCounters {
        self.compilation.counters()
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
                cache: Shared::new(self.cache.as_ref().clone()),
            }),
        }
    }
}

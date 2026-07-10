//! Observable query counters and per-execution traces.

use vize_carton::FxHashMap;

use crate::{Product, ProductId, ProductRequest, ProviderId, SourceId};

/// Aggregate counters for one product since a compilation was created.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct ProductCounters {
    queries: u64,
    executions: u64,
    cache_hits: u64,
}

impl ProductCounters {
    /// Root queries plus typed dependency reads.
    pub const fn queries(self) -> u64 {
        self.queries
    }

    /// Provider invocations, including invocations that returned an error.
    pub const fn executions(self) -> u64 {
        self.executions
    }

    /// Values reused from the compilation cache.
    pub const fn cache_hits(self) -> u64 {
        self.cache_hits
    }
}

/// Aggregate counters for all products in one compilation.
#[derive(Debug, Clone, Default)]
pub struct ExecutionCounters {
    products: FxHashMap<ProductId, ProductCounters>,
}

impl ExecutionCounters {
    /// Read counters for typed product `P`.
    pub fn for_product<P: Product>(&self) -> ProductCounters {
        self.for_id(ProductId::of::<P>())
    }

    /// Read counters for a runtime product identity.
    pub fn for_id(&self, product: ProductId) -> ProductCounters {
        self.products.get(&product).copied().unwrap_or_default()
    }

    pub(crate) fn record_query(&mut self, product: ProductId) {
        let counters = self.products.entry(product).or_default();
        counters.queries = counters.queries.saturating_add(1);
    }

    pub(crate) fn record_execution(&mut self, product: ProductId) {
        let counters = self.products.entry(product).or_default();
        counters.executions = counters.executions.saturating_add(1);
    }

    pub(crate) fn record_cache_hit(&mut self, product: ProductId) {
        let counters = self.products.entry(product).or_default();
        counters.cache_hits = counters.cache_hits.saturating_add(1);
    }
}

/// One observable event from executing a dependency plan.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TraceEvent {
    /// A user-facing root was queried.
    RootQueried { product: ProductId },
    /// A provider read one of its declared typed dependencies.
    DependencyQueried {
        provider: ProviderId,
        dependency: ProductId,
    },
    /// A memoized value was reused.
    CacheHit {
        product: ProductId,
        provider: ProviderId,
    },
    /// A provider completed and its value entered the cache.
    ProviderExecuted {
        product: ProductId,
        provider: ProviderId,
    },
    /// A root with an explicit source identity was queried.
    RequestRootQueried { request: ProductRequest },
    /// A provider read a declared cross-source dependency.
    RequestDependencyQueried {
        provider: ProviderId,
        dependency: ProductRequest,
    },
    /// A memoized cross-source request was reused.
    RequestCacheHit {
        request: ProductRequest,
        provider: ProviderId,
    },
    /// A provider completed for an explicitly identified request.
    RequestProviderExecuted {
        request: ProductRequest,
        provider: ProviderId,
    },
}

/// Ordered events for one plan execution.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct ExecutionTrace {
    events: Vec<TraceEvent>,
}

impl ExecutionTrace {
    /// Inspect events in execution order.
    pub fn events(&self) -> &[TraceEvent] {
        &self.events
    }

    /// Whether a provider for `P` executed in this outcome.
    pub fn executed<P: Product>(&self) -> bool {
        let product = ProductId::of::<P>();
        self.events.iter().any(|event| match event {
            TraceEvent::ProviderExecuted {
                product: executed, ..
            } => *executed == product,
            TraceEvent::RequestProviderExecuted { request, .. } => request.product() == product,
            _ => false,
        })
    }

    /// Whether `P` was served from cache in this outcome.
    pub fn cache_hit<P: Product>(&self) -> bool {
        let product = ProductId::of::<P>();
        self.events.iter().any(|event| match event {
            TraceEvent::CacheHit {
                product: cached, ..
            } => *cached == product,
            TraceEvent::RequestCacheHit { request, .. } => request.product() == product,
            _ => false,
        })
    }

    /// Whether a provider for `P` executed for the specified source.
    pub fn executed_for_source<P: Product>(&self, source: SourceId) -> bool {
        let expected = ProductRequest::for_product::<P>(source);
        self.events.iter().any(|event| {
            matches!(
                event,
                TraceEvent::RequestProviderExecuted { request, .. } if *request == expected
            )
        })
    }

    /// Whether `P` was served from cache for the specified source.
    pub fn cache_hit_for_source<P: Product>(&self, source: SourceId) -> bool {
        let expected = ProductRequest::for_product::<P>(source);
        self.events.iter().any(|event| {
            matches!(
                event,
                TraceEvent::RequestCacheHit { request, .. } if *request == expected
            )
        })
    }

    pub(crate) fn push(&mut self, event: TraceEvent) {
        self.events.push(event);
    }
}

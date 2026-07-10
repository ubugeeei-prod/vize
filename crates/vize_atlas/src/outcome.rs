//! Typed query and multi-root execution outcomes.

use std::marker::PhantomData;
use vize_carton::FxHashMap;

use crate::{
    ExecutionTrace, Plan, Product, ProductId, ProductRequest, ProductView, ProviderObservation,
    QueryError, Shared, SourceId, SourceRevision, provider::ErasedValue,
};

/// How one product value was obtained during an execution.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProductStatus {
    Executed,
    CacheHit,
    /// Planned dependency skipped because every consumer was already cached.
    Pruned,
}

/// Values and observations produced by executing one dependency plan.
pub struct ExecutionOutcome {
    plan: Plan,
    values: FxHashMap<ProductRequest, ErasedValue>,
    statuses: FxHashMap<ProductRequest, ProductStatus>,
    observations: Vec<ProviderObservation>,
    trace: ExecutionTrace,
}

impl ExecutionOutcome {
    pub(crate) fn new(
        plan: Plan,
        values: FxHashMap<ProductRequest, ErasedValue>,
        statuses: FxHashMap<ProductRequest, ProductStatus>,
        observations: Vec<ProviderObservation>,
        trace: ExecutionTrace,
    ) -> Self {
        Self {
            plan,
            values,
            statuses,
            observations,
            trace,
        }
    }

    pub fn plan(&self) -> &Plan {
        &self.plan
    }

    pub const fn source(&self) -> SourceId {
        self.plan.source()
    }

    pub const fn source_revision(&self) -> SourceRevision {
        self.plan.source_revision()
    }

    /// Every source revision captured by the executed plan.
    pub fn source_revisions(&self) -> &[(SourceId, SourceRevision)] {
        self.plan.source_revisions()
    }

    pub const fn trace(&self) -> &ExecutionTrace {
        &self.trace
    }

    /// Structured side outcomes from executed or cached providers.
    pub fn observations(&self) -> &[ProviderObservation] {
        &self.observations
    }

    /// Observations emitted by the provider for one complete request.
    pub fn observations_for_request(
        &self,
        request: ProductRequest,
    ) -> impl Iterator<Item = &ProviderObservation> {
        self.observations
            .iter()
            .filter(move |observation| observation.request() == request)
    }

    /// Return how `product` was handled, or `None` when it was outside the plan.
    pub fn status(&self, product: ProductId) -> Option<ProductStatus> {
        self.status_for_request(ProductRequest::new(self.source(), product))
    }

    /// Return how one complete product request was obtained.
    pub fn status_for_request(&self, request: ProductRequest) -> Option<ProductStatus> {
        self.statuses.get(&request).copied()
    }

    /// Clone a typed product value from this outcome.
    pub fn get<P: Product>(&self) -> Result<Option<Shared<P::Value>>, QueryError> {
        let product = ProductId::of::<P>();
        let request = ProductRequest::new(self.source(), product);
        let Some(value) = self.values.get(&request) else {
            return Ok(None);
        };
        Shared::downcast::<P::Value>(Shared::clone(value))
            .map(Some)
            .map_err(|_| QueryError::ProductTypeMismatch(product))
    }

    /// Clone a typed product value for an explicit source.
    pub fn get_for_source<P: Product>(
        &self,
        source: SourceId,
    ) -> Result<Option<Shared<P::Value>>, QueryError> {
        if source == self.source() {
            return self.get::<P>();
        }
        let request = ProductRequest::for_product::<P>(source);
        let Some(value) = self.values.get(&request) else {
            return Ok(None);
        };
        Shared::downcast::<P::Value>(Shared::clone(value))
            .map(Some)
            .map_err(|_| QueryError::RequestTypeMismatch(request))
    }

    /// Borrow a consumer view of a product for this outcome's root source.
    ///
    /// The returned view cannot outlive this outcome because it projects the
    /// owned storage retained by the execution result and artifact cache.
    pub fn view<P: ProductView>(&self) -> Result<Option<P::View<'_>>, QueryError> {
        let product = ProductId::of::<P>();
        let request = ProductRequest::new(self.source(), product);
        let Some(value) = self.values.get(&request) else {
            return Ok(None);
        };
        let storage = value
            .as_ref()
            .downcast_ref::<P::Value>()
            .ok_or(QueryError::ProductTypeMismatch(product))?;
        Ok(Some(P::view(storage)))
    }

    /// Borrow a consumer view of a product for an explicit source.
    pub fn view_for_source<P: ProductView>(
        &self,
        source: SourceId,
    ) -> Result<Option<P::View<'_>>, QueryError> {
        if source == self.source() {
            return self.view::<P>();
        }
        let request = ProductRequest::for_product::<P>(source);
        let Some(value) = self.values.get(&request) else {
            return Ok(None);
        };
        let storage = value
            .as_ref()
            .downcast_ref::<P::Value>()
            .ok_or(QueryError::RequestTypeMismatch(request))?;
        Ok(Some(P::view(storage)))
    }
}

/// Strongly typed convenience result for one requested root.
pub struct QueryOutcome<P: Product> {
    value: Shared<P::Value>,
    status: ProductStatus,
    execution: ExecutionOutcome,
    _product: PhantomData<fn() -> P>,
}

impl<P: Product> QueryOutcome<P> {
    pub(crate) const fn new(
        value: Shared<P::Value>,
        status: ProductStatus,
        execution: ExecutionOutcome,
    ) -> Self {
        Self {
            value,
            status,
            execution,
            _product: PhantomData,
        }
    }

    /// Borrow the typed value.
    pub fn value(&self) -> &P::Value {
        &self.value
    }

    /// Clone the reference-counted typed value.
    pub fn shared(&self) -> Shared<P::Value> {
        Shared::clone(&self.value)
    }

    /// Whether the root executed or was memoized.
    pub const fn status(&self) -> ProductStatus {
        self.status
    }

    pub const fn execution(&self) -> &ExecutionOutcome {
        &self.execution
    }

    pub const fn trace(&self) -> &ExecutionTrace {
        self.execution.trace()
    }

    pub fn plan(&self) -> &Plan {
        self.execution.plan()
    }
}

impl<P: ProductView> QueryOutcome<P> {
    /// Borrow the product's consumer view over its shared cached storage.
    pub fn view(&self) -> P::View<'_> {
        P::view(self.value.as_ref())
    }
}

//! Object-safe provider bridge used by the open registry.

use crate::{
    CachePolicy, InputId, Product, ProductRequest, Provider, ProviderError, Shared, SourceInputId,
};

use super::{ErasedValue, PlanningContext, ProviderContext};

pub(crate) trait ErasedProvider: Send + Sync {
    fn cache_policy(&self) -> CachePolicy;
    fn input_dependencies(&self) -> Vec<InputId>;
    fn source_input_dependencies(&self) -> Vec<SourceInputId>;
    fn source_dependencies(&self, context: &PlanningContext<'_>) -> Vec<crate::SourceId>;
    fn supports(&self, context: &PlanningContext<'_>) -> bool;
    fn dependency_requests(&self, context: &PlanningContext<'_>) -> Vec<ProductRequest>;
    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<ErasedValue, ProviderError>;
}

pub(crate) struct ProviderAdapter<T: Provider> {
    provider: T,
}

impl<T: Provider> ProviderAdapter<T> {
    pub(crate) const fn new(provider: T) -> Self {
        Self { provider }
    }
}

impl<T: Provider> ErasedProvider for ProviderAdapter<T> {
    fn cache_policy(&self) -> CachePolicy {
        T::Product::CACHE_POLICY
    }

    fn input_dependencies(&self) -> Vec<InputId> {
        self.provider.input_dependencies()
    }

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        self.provider.source_input_dependencies()
    }

    fn source_dependencies(&self, context: &PlanningContext<'_>) -> Vec<crate::SourceId> {
        self.provider.source_dependencies(context)
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        self.provider.supports(context)
    }

    fn dependency_requests(&self, context: &PlanningContext<'_>) -> Vec<ProductRequest> {
        self.provider.dependency_requests(context)
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<ErasedValue, ProviderError> {
        self.provider
            .provide(context)
            .map(|value| Shared::new(value) as ErasedValue)
    }
}

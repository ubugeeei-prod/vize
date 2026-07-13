//! Typed providers and their constrained execution context.

mod erased;
mod observations;

pub(crate) use erased::{ErasedProvider, ProviderAdapter};

use std::{
    any::{Any, TypeId, type_name},
    fmt,
};
use vize_carton::FxHashMap;

use crate::{
    CompilationInput, CompilationInputs, ExecutionCounters, ExecutionTrace, InputId, Product,
    ProductId, ProductRequest, ProviderError, ProviderObservation, Shared, SourceId, SourceInput,
    SourceInputId, SourceSnapshot, SourceStore, TraceEvent,
};

pub(crate) type ErasedValue = Shared<dyn Any + Send + Sync>;

/// Runtime identity of one concrete [`Provider`] implementation.
///
/// Provider identity is open in the same way as [`ProductId`]: a frontend or
/// backend introduces a provider by defining an ordinary Rust type in its own
/// crate. Atlas does not maintain a closed provider-kind enum.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ProviderId {
    type_id: TypeId,
    name: &'static str,
}

impl ProviderId {
    /// Return the identity of concrete provider type `T`.
    pub fn of<T: Provider>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            name: type_name::<T>(),
        }
    }

    /// Return the concrete provider type name used in diagnostics.
    pub const fn name(self) -> &'static str {
        self.name
    }
}

impl fmt::Debug for ProviderId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ProviderId")
            .field(&self.name)
            .finish()
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name)
    }
}

/// Read-only source and compilation inputs used to shape a dependency plan.
pub struct PlanningContext<'a> {
    source: &'a SourceSnapshot,
    sources: &'a SourceStore,
    inputs: &'a CompilationInputs,
}

impl<'a> PlanningContext<'a> {
    pub(crate) const fn new(
        source: &'a SourceSnapshot,
        sources: &'a SourceStore,
        inputs: &'a CompilationInputs,
    ) -> Self {
        Self {
            source,
            sources,
            inputs,
        }
    }

    /// Source for the product currently being planned.
    pub const fn source(&self) -> &SourceSnapshot {
        self.source
    }

    /// All immutable source snapshots available to this compilation.
    pub const fn sources(&self) -> &SourceStore {
        self.sources
    }

    /// Look up another source while constructing cross-source dependencies.
    pub fn source_by_id(&self, source: SourceId) -> Option<&SourceSnapshot> {
        self.sources.get(source)
    }

    /// Read an open typed dialect, target, or capability input.
    pub fn input<I: CompilationInput>(&self) -> Option<&I::Value> {
        self.inputs.get::<I>()
    }

    /// Read an open typed option scoped to the source being planned.
    pub fn source_input<I: SourceInput>(&self) -> Option<&I::Value> {
        self.inputs.get_source::<I>(self.source.id())
    }
}

/// Open contract for constructing one typed product.
///
/// Dependencies are declared before planning. During execution the provider
/// may read the current source and may query only those declared products.
pub trait Provider: Send + Sync + 'static {
    type Product: Product;

    /// Typed compilation inputs that can affect applicability, dependencies,
    /// or output from this provider.
    ///
    /// Atlas uses this declaration to stale plans and evict cached products
    /// selectively. It must include every input read through either planning
    /// or execution context. The declaration itself must not depend on input
    /// values.
    fn input_dependencies(&self) -> Vec<InputId> {
        Vec::new()
    }

    /// Source-scoped inputs that can affect this provider.
    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        Vec::new()
    }

    /// Additional raw source revisions read without requesting a product.
    fn source_dependencies(&self, _context: &PlanningContext<'_>) -> Vec<SourceId> {
        Vec::new()
    }

    /// Whether this provider applies to the current source and typed inputs.
    ///
    /// Multiple crates may register providers for the same product. Planning
    /// succeeds only when exactly one registered provider supports the current
    /// context. An unselected provider's dependency hook is never called.
    fn supports(&self, _context: &PlanningContext<'_>) -> bool {
        true
    }

    /// Direct product dependencies. The planner follows these transitively.
    ///
    /// This hook is source- and input-aware: a composite frontend can choose
    /// an SFC closure for `.vue` and a JSX closure for `.tsx` without forcing
    /// both representations into every plan.
    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        Vec::new()
    }

    /// Direct product requests, including optional cross-source dependencies.
    ///
    /// The default preserves the single-source contract by attaching every
    /// value returned from [`Provider::dependencies`] to the current source.
    /// Providers that aggregate a project or follow imports override this hook
    /// and return complete [`ProductRequest`] identities instead.
    fn dependency_requests(&self, context: &PlanningContext<'_>) -> Vec<ProductRequest> {
        let source = context.source().id();
        self.dependencies(context)
            .into_iter()
            .map(|product| ProductRequest::new(source, product))
            .collect()
    }

    /// Construct the product for the context's source.
    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<<Self::Product as Product>::Value, ProviderError>;
}

/// Source and already-resolved dependencies visible to one provider.
pub struct ProviderContext<'a> {
    request: ProductRequest,
    source: &'a SourceSnapshot,
    sources: &'a SourceStore,
    provider: ProviderId,
    declared: &'a [ProductRequest],
    resolved: &'a FxHashMap<ProductRequest, ErasedValue>,
    inputs: &'a CompilationInputs,
    counters: &'a mut ExecutionCounters,
    trace: &'a mut ExecutionTrace,
    observations: &'a mut Vec<ProviderObservation>,
}

pub(crate) struct ProviderExecution<'a> {
    pub(crate) resolved: &'a FxHashMap<ProductRequest, ErasedValue>,
    pub(crate) inputs: &'a CompilationInputs,
    pub(crate) counters: &'a mut ExecutionCounters,
    pub(crate) trace: &'a mut ExecutionTrace,
    pub(crate) observations: &'a mut Vec<ProviderObservation>,
}

impl<'a> ProviderContext<'a> {
    pub(crate) fn new(
        request: ProductRequest,
        source: &'a SourceSnapshot,
        sources: &'a SourceStore,
        provider: ProviderId,
        declared: &'a [ProductRequest],
        execution: ProviderExecution<'a>,
    ) -> Self {
        Self {
            request,
            source,
            sources,
            provider,
            declared,
            resolved: execution.resolved,
            inputs: execution.inputs,
            counters: execution.counters,
            trace: execution.trace,
            observations: execution.observations,
        }
    }

    /// Source snapshot for which this provider is executing.
    pub const fn source(&self) -> &SourceSnapshot {
        self.source
    }

    /// All source snapshots captured for this provider execution.
    pub const fn sources(&self) -> &SourceStore {
        self.sources
    }

    /// Look up another source by stable identity.
    pub fn source_by_id(&self, source: SourceId) -> Option<&SourceSnapshot> {
        self.sources.get(source)
    }

    /// Identity of the concrete provider selected by the plan.
    pub const fn provider(&self) -> ProviderId {
        self.provider
    }

    /// Complete product request currently being provided.
    pub const fn request(&self) -> ProductRequest {
        self.request
    }

    /// Read the same typed compilation input used during planning.
    pub fn input<I: CompilationInput>(&self) -> Option<&I::Value> {
        self.inputs.get::<I>()
    }

    /// Read the typed option attached to the current source identity.
    pub fn source_input<I: SourceInput>(&self) -> Option<&I::Value> {
        self.inputs.get_source::<I>(self.source.id())
    }

    /// Query a declared, already-planned typed dependency.
    pub fn get<P: Product>(&mut self) -> Result<Shared<P::Value>, ProviderError> {
        let dependency = ProductId::of::<P>();
        let request = ProductRequest::new(self.source.id(), dependency);
        self.counters.record_query(dependency);
        self.trace.push(TraceEvent::DependencyQueried {
            provider: self.provider,
            dependency,
        });
        if !self.declared.contains(&request) {
            return Err(ProviderError::UndeclaredDependency {
                provider: self.provider,
                dependency,
            });
        }
        let value =
            self.resolved
                .get(&request)
                .cloned()
                .ok_or(ProviderError::DependencyUnavailable {
                    provider: self.provider,
                    dependency,
                })?;
        Shared::downcast::<P::Value>(value)
            .map_err(|_| ProviderError::DependencyTypeMismatch(dependency))
    }

    /// Query a declared typed dependency belonging to another source.
    ///
    /// Passing the current source is equivalent to [`ProviderContext::get`]
    /// and retains the original single-source error and trace forms.
    pub fn get_for_source<P: Product>(
        &mut self,
        source: SourceId,
    ) -> Result<Shared<P::Value>, ProviderError> {
        if source == self.source.id() {
            return self.get::<P>();
        }
        let request = ProductRequest::for_product::<P>(source);
        self.counters.record_query(request.product());
        self.trace.push(TraceEvent::RequestDependencyQueried {
            provider: self.provider,
            dependency: request,
        });
        if !self.declared.contains(&request) {
            return Err(ProviderError::UndeclaredRequest {
                provider: self.provider,
                dependency: request,
            });
        }
        let value =
            self.resolved
                .get(&request)
                .cloned()
                .ok_or(ProviderError::RequestUnavailable {
                    provider: self.provider,
                    dependency: request,
                })?;
        Shared::downcast::<P::Value>(value).map_err(|_| ProviderError::RequestTypeMismatch(request))
    }
}

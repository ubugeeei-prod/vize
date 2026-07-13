//! Request-keyed plan execution and typed query convenience.

use vize_carton::{FxHashMap, FxHashSet};

use super::Compilation;
use crate::{
    CachePolicy, ExecutionOutcome, ExecutionTrace, Plan, PlanError, Product, ProductRequest,
    ProductStatus, ProviderContext, ProviderObservation, QueryError, QueryOutcome, Shared,
    SourceId, TraceEvent, cache::CachedArtifact, provider::ProviderExecution,
};

impl Compilation {
    /// Execute a previously built plan in dependency order.
    pub fn execute(&mut self, plan: Plan) -> Result<ExecutionOutcome, QueryError> {
        self.validate_plan(&plan)?;
        let request_traces = plan.source_revisions.len() > 1;
        let mut trace = ExecutionTrace::default();
        for root in &plan.root_requests {
            self.counters.record_query(root.product());
            if request_traces {
                trace.push(TraceEvent::RequestRootQueried { request: *root });
            } else {
                trace.push(TraceEvent::RootQueried {
                    product: root.product(),
                });
            }
        }
        let mut values = FxHashMap::default();
        let mut statuses = FxHashMap::default();
        let mut observation_closures = FxHashMap::default();
        let (required, mut cached_values) = self.required_requests(&plan)?;
        for request in plan
            .requests
            .iter()
            .filter(|request| !required.contains(request))
        {
            statuses.insert(*request, ProductStatus::Pruned);
        }
        for request in &plan.requests {
            if !required.contains(request) {
                continue;
            }
            let source = self
                .sources
                .get(request.source())
                .cloned()
                .ok_or(PlanError::SourceNotFound(request.source()))?;
            let selected = plan
                .providers
                .get(request)
                .copied()
                .ok_or(QueryError::MissingRequest(*request))?;
            let entry = self
                .providers
                .get(&request.product())
                .and_then(|entries| entries.iter().find(|entry| entry.id == selected))
                .cloned()
                .ok_or_else(|| request_missing_provider(*request))?;
            let cache_policy = entry.provider.cache_policy();
            if let Some(cached) = cached_values.remove(request) {
                values.insert(*request, cached.value);
                observation_closures.insert(*request, cached.observation_closure);
                statuses.insert(*request, ProductStatus::CacheHit);
                self.counters.record_cache_hit(request.product());
                push_cache_trace(&mut trace, *request, selected, request_traces);
                continue;
            }

            let declared = plan.dependencies.get(request).cloned().unwrap_or_default();
            let source_dependencies = plan
                .source_dependencies
                .get(request)
                .map_or(&[][..], Vec::as_slice);
            self.counters.record_execution(request.product());
            let mut provider_observations = Vec::new();
            let value = {
                let mut context = ProviderContext::new(
                    *request,
                    &source,
                    &self.sources,
                    selected,
                    &declared,
                    ProviderExecution {
                        resolved: &values,
                        inputs: &self.inputs,
                        counters: &mut self.counters,
                        trace: &mut trace,
                        observations: &mut provider_observations,
                    },
                );
                entry.provider.provide(&mut context).map_err(|error| {
                    QueryError::ProviderFailed {
                        source: request.source(),
                        product: request.product(),
                        provider: selected,
                        error: Box::new(error),
                    }
                })?
            };
            let observation_closure = build_observation_closure(
                &declared,
                &observation_closures,
                &provider_observations,
            )?;
            if cache_policy == CachePolicy::Memoized {
                self.cache.insert(
                    *request,
                    source.revision(),
                    selected,
                    crate::cache::CacheDependencies {
                        inputs: plan
                            .input_dependencies
                            .get(request)
                            .map_or(&[], Vec::as_slice),
                        source_inputs: plan
                            .source_input_dependencies
                            .get(request)
                            .map_or(&[], Vec::as_slice),
                        sources: source_dependencies,
                    },
                    CachedArtifact {
                        value: Shared::clone(&value),
                        observation_closure: observation_closure.clone(),
                    },
                );
            }
            observation_closures.insert(*request, observation_closure);
            values.insert(*request, value);
            statuses.insert(*request, ProductStatus::Executed);
            push_execution_trace(&mut trace, *request, selected, request_traces);
        }
        let observations =
            build_observation_closure(&plan.root_requests, &observation_closures, &[])?;
        Ok(ExecutionOutcome::new(
            plan,
            values,
            statuses,
            observations,
            trace,
        ))
    }

    fn required_requests(
        &self,
        plan: &Plan,
    ) -> Result<
        (
            FxHashSet<ProductRequest>,
            FxHashMap<ProductRequest, CachedArtifact>,
        ),
        QueryError,
    > {
        let mut required = FxHashSet::default();
        let mut cached = FxHashMap::default();
        for root in &plan.root_requests {
            self.require_request(plan, *root, &mut required, &mut cached)?;
        }
        Ok((required, cached))
    }

    fn require_request(
        &self,
        plan: &Plan,
        request: ProductRequest,
        required: &mut FxHashSet<ProductRequest>,
        cached: &mut FxHashMap<ProductRequest, CachedArtifact>,
    ) -> Result<(), QueryError> {
        if required.contains(&request) {
            return Ok(());
        }
        let source = self
            .sources
            .get(request.source())
            .ok_or(PlanError::SourceNotFound(request.source()))?;
        let selected = plan
            .providers
            .get(&request)
            .copied()
            .ok_or(QueryError::MissingRequest(request))?;
        let entry = self
            .providers
            .get(&request.product())
            .and_then(|entries| entries.iter().find(|entry| entry.id == selected))
            .ok_or_else(|| request_missing_provider(request))?;
        let source_dependencies = plan
            .source_dependencies
            .get(&request)
            .map_or(&[][..], Vec::as_slice);
        if entry.provider.cache_policy() == CachePolicy::Memoized
            && let Some(artifact) =
                self.cache
                    .get(request, source.revision(), selected, source_dependencies)
        {
            required.insert(request);
            cached.insert(request, artifact);
            return Ok(());
        }
        if let Some(dependencies) = plan.dependencies.get(&request) {
            for dependency in dependencies {
                self.require_request(plan, *dependency, required, cached)?;
            }
        }
        required.insert(request);
        Ok(())
    }

    fn validate_plan(&self, plan: &Plan) -> Result<(), QueryError> {
        for (source, planned) in &plan.source_revisions {
            let current = self
                .sources
                .get(*source)
                .ok_or(PlanError::SourceNotFound(*source))?
                .revision();
            if current != *planned {
                return Err(QueryError::StaleSourcePlan {
                    source: *source,
                    planned: *planned,
                    current,
                });
            }
        }
        if self.provider_generation != plan.provider_generation {
            return Err(QueryError::StaleProviderPlan {
                planned: plan.provider_generation,
                current: self.provider_generation,
            });
        }
        for (input, planned) in &plan.input_revisions {
            let current = self.inputs.revision(*input);
            if current != *planned {
                return Err(QueryError::StaleInputPlan {
                    input: *input,
                    planned: *planned,
                    current,
                });
            }
        }
        for (source, input, planned) in &plan.source_input_revisions {
            let current = self.inputs.source_revision(*source, *input);
            if current != *planned {
                return Err(QueryError::StaleSourceInputPlan {
                    source: *source,
                    input: *input,
                    planned: *planned,
                    current,
                });
            }
        }
        Ok(())
    }

    /// Plan, execute, and return one strongly typed root product.
    pub fn query<P: Product>(&mut self, source: SourceId) -> Result<QueryOutcome<P>, QueryError> {
        let plan = self.plan_for::<P>(source)?;
        let execution = self.execute(plan)?;
        let request = ProductRequest::for_product::<P>(source);
        let value = execution
            .get_for_source::<P>(source)?
            .ok_or(QueryError::MissingRequest(request))?;
        let status = execution
            .status_for_request(request)
            .ok_or(QueryError::MissingRequest(request))?;
        Ok(QueryOutcome::new(value, status, execution))
    }
}

fn push_cache_trace(
    trace: &mut ExecutionTrace,
    request: ProductRequest,
    provider: crate::ProviderId,
    request_traces: bool,
) {
    if request_traces {
        trace.push(TraceEvent::RequestCacheHit { request, provider });
    } else {
        trace.push(TraceEvent::CacheHit {
            product: request.product(),
            provider,
        });
    }
}

fn push_execution_trace(
    trace: &mut ExecutionTrace,
    request: ProductRequest,
    provider: crate::ProviderId,
    request_traces: bool,
) {
    if request_traces {
        trace.push(TraceEvent::RequestProviderExecuted { request, provider });
    } else {
        trace.push(TraceEvent::ProviderExecuted {
            product: request.product(),
            provider,
        });
    }
}

fn request_missing_provider(request: ProductRequest) -> PlanError {
    PlanError::MissingRequestProvider {
        request,
        required_by: None,
    }
}

fn build_observation_closure(
    requests: &[ProductRequest],
    closures: &FxHashMap<ProductRequest, Vec<ProviderObservation>>,
    local: &[ProviderObservation],
) -> Result<Vec<ProviderObservation>, QueryError> {
    let mut observations = Vec::new();
    let mut included = FxHashSet::default();
    for request in requests {
        let closure = closures
            .get(request)
            .ok_or(QueryError::MissingRequest(*request))?;
        extend_unique_requests(&mut observations, &mut included, closure);
    }
    observations.extend(local.iter().cloned());
    Ok(observations)
}

fn extend_unique_requests(
    target: &mut Vec<ProviderObservation>,
    included: &mut FxHashSet<ProductRequest>,
    observations: &[ProviderObservation],
) {
    let mut start = 0;
    while start < observations.len() {
        let request = observations[start].request();
        let end = observations[start..]
            .iter()
            .position(|observation| observation.request() != request)
            .map_or(observations.len(), |offset| start + offset);
        if included.insert(request) {
            target.extend(observations[start..end].iter().cloned());
        }
        start = end;
    }
}

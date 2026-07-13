//! Source-aware dependency closure planning.

use vize_carton::{FxHashMap, FxHashSet};

use crate::{
    CompilationInputs, InputId, Plan, PlanError, PlanningContext, ProductId, ProductRequest,
    ProviderId, SourceId, SourceInputId, SourceRevision, SourceStore, compilation::ProviderEntry,
};

pub(crate) fn build_plan(
    sources: &SourceStore,
    providers: &FxHashMap<ProductId, Vec<ProviderEntry>>,
    inputs: &CompilationInputs,
    provider_generation: u64,
    roots: impl IntoIterator<Item = ProductRequest>,
) -> Result<Plan, PlanError> {
    let mut seen_roots = FxHashSet::default();
    let roots: Vec<_> = roots
        .into_iter()
        .filter(|root| seen_roots.insert(*root))
        .collect();
    let Some(first_root) = roots.first().copied() else {
        return Err(PlanError::NoRoots);
    };
    let first_snapshot = validate_source(sources, first_root.source())?;

    let mut traversal = Traversal::new(sources, providers, inputs, first_root.source());
    for root in &roots {
        traversal.visit(*root, None)?;
    }

    let mut seen_inputs = FxHashSet::default();
    let mut input_revisions = Vec::new();
    for request in &traversal.requests {
        if let Some(request_inputs) = traversal.input_dependencies.get(request) {
            for input in request_inputs {
                if seen_inputs.insert(*input) {
                    input_revisions.push((*input, inputs.revision(*input)));
                }
            }
        }
    }

    let mut source_revisions: Vec<_> = traversal
        .requests
        .iter()
        .filter_map(|request| {
            sources
                .get(request.source())
                .map(|source| (source.id(), source.revision()))
        })
        .collect();
    source_revisions.sort_unstable_by_key(|(source, _)| *source);
    source_revisions.dedup_by_key(|(source, _)| *source);

    let mut seen_source_inputs = FxHashSet::default();
    let mut source_input_revisions = Vec::new();
    for request in &traversal.requests {
        if let Some(request_inputs) = traversal.source_input_dependencies.get(request) {
            for &(source, input) in request_inputs {
                if seen_source_inputs.insert((source, input)) {
                    source_input_revisions.push((
                        source,
                        input,
                        inputs.source_revision(source, input),
                    ));
                }
            }
        }
    }

    let root_products = roots.iter().map(|request| request.product()).collect();
    let products = traversal
        .requests
        .iter()
        .map(|request| request.product())
        .collect();
    let product_dependencies = traversal
        .dependencies
        .iter()
        .filter(|(request, _)| request.source() == first_root.source())
        .map(|(request, dependencies)| {
            (
                request.product(),
                dependencies
                    .iter()
                    .map(|dependency| dependency.product())
                    .collect(),
            )
        })
        .collect();

    Ok(Plan {
        source: first_root.source(),
        source_revision: first_snapshot.revision(),
        source_revisions,
        provider_generation,
        input_revisions,
        source_input_revisions,
        roots: root_products,
        products,
        root_requests: roots,
        requests: traversal.requests,
        product_dependencies,
        dependencies: traversal.dependencies,
        providers: traversal.selected,
        input_dependencies: traversal.input_dependencies,
        source_input_dependencies: traversal.source_input_dependencies,
        source_dependencies: traversal.source_dependencies,
    })
}

fn validate_source(
    sources: &SourceStore,
    source: SourceId,
) -> Result<&crate::SourceSnapshot, PlanError> {
    let snapshot = sources
        .get(source)
        .ok_or(PlanError::SourceNotFound(source))?;
    if let Some(stale) = sources.stale_edge(source) {
        return Err(PlanError::StaleEmbeddedSource {
            source: stale.source,
            parent: stale.parent,
            recorded: stale.recorded,
            current: stale.current,
        });
    }
    Ok(snapshot)
}

#[derive(Clone, Copy)]
enum VisitState {
    Visiting,
    Complete,
}

struct Traversal<'a> {
    sources: &'a SourceStore,
    providers: &'a FxHashMap<ProductId, Vec<ProviderEntry>>,
    inputs: &'a CompilationInputs,
    compatibility_source: SourceId,
    states: FxHashMap<ProductRequest, VisitState>,
    stack: Vec<ProductRequest>,
    requests: Vec<ProductRequest>,
    dependencies: FxHashMap<ProductRequest, Vec<ProductRequest>>,
    selected: FxHashMap<ProductRequest, ProviderId>,
    input_dependencies: FxHashMap<ProductRequest, Vec<InputId>>,
    source_input_dependencies: FxHashMap<ProductRequest, Vec<(SourceId, SourceInputId)>>,
    source_dependencies: FxHashMap<ProductRequest, Vec<(SourceId, SourceRevision)>>,
}

impl<'a> Traversal<'a> {
    fn new(
        sources: &'a SourceStore,
        providers: &'a FxHashMap<ProductId, Vec<ProviderEntry>>,
        inputs: &'a CompilationInputs,
        compatibility_source: SourceId,
    ) -> Self {
        Self {
            sources,
            providers,
            inputs,
            compatibility_source,
            states: FxHashMap::default(),
            stack: Vec::new(),
            requests: Vec::new(),
            dependencies: FxHashMap::default(),
            selected: FxHashMap::default(),
            input_dependencies: FxHashMap::default(),
            source_input_dependencies: FxHashMap::default(),
            source_dependencies: FxHashMap::default(),
        }
    }

    fn visit(
        &mut self,
        request: ProductRequest,
        required_by: Option<ProductRequest>,
    ) -> Result<(), PlanError> {
        match self.states.get(&request) {
            Some(VisitState::Complete) => return Ok(()),
            Some(VisitState::Visiting) => return Err(self.cycle(request)),
            None => {}
        }
        let source = validate_source(self.sources, request.source())?;
        let context = PlanningContext::new(source, self.sources, self.inputs);
        let (entry, mut relevant_inputs, mut relevant_source_inputs) =
            self.select_provider(request, required_by, &context)?;
        self.states.insert(request, VisitState::Visiting);
        self.stack.push(request);
        let mut seen = FxHashSet::default();
        let mut direct = entry.provider.dependency_requests(&context);
        direct.retain(|dependency| seen.insert(*dependency));
        self.selected.insert(request, entry.id);
        self.dependencies.insert(request, direct.clone());

        let mut relevant_sources = vec![(request.source(), source.revision())];
        for source in entry.provider.source_dependencies(&context) {
            let source = validate_source(self.sources, source)?;
            relevant_sources.push((source.id(), source.revision()));
        }
        for dependency in &direct {
            self.visit(*dependency, Some(request))?;
            if let Some(dependency_inputs) = self.input_dependencies.get(dependency) {
                relevant_inputs.extend(dependency_inputs.iter().copied());
            }
            if let Some(dependency_inputs) = self.source_input_dependencies.get(dependency) {
                relevant_source_inputs.extend(dependency_inputs.iter().copied());
            }
            if let Some(dependency_sources) = self.source_dependencies.get(dependency) {
                relevant_sources.extend(dependency_sources.iter().copied());
            }
        }
        let mut seen_inputs = FxHashSet::default();
        relevant_inputs.retain(|input| seen_inputs.insert(*input));
        self.input_dependencies.insert(request, relevant_inputs);
        let mut seen_source_inputs = FxHashSet::default();
        relevant_source_inputs.retain(|input| seen_source_inputs.insert(*input));
        self.source_input_dependencies
            .insert(request, relevant_source_inputs);
        relevant_sources.sort_unstable_by_key(|(source, _)| *source);
        relevant_sources.dedup_by_key(|(source, _)| *source);
        self.source_dependencies.insert(request, relevant_sources);
        self.stack.pop();
        self.states.insert(request, VisitState::Complete);
        self.requests.push(request);
        Ok(())
    }

    fn select_provider(
        &self,
        request: ProductRequest,
        required_by: Option<ProductRequest>,
        context: &PlanningContext<'_>,
    ) -> Result<(ProviderEntry, Vec<InputId>, Vec<(SourceId, SourceInputId)>), PlanError> {
        let Some(registered) = self.providers.get(&request.product()) else {
            return Err(self.missing_provider(request, required_by));
        };
        let mut seen_inputs = FxHashSet::default();
        let mut relevant_inputs = Vec::new();
        let mut seen_source_inputs = FxHashSet::default();
        let mut relevant_source_inputs = Vec::new();
        for entry in registered {
            for input in entry.provider.input_dependencies() {
                if seen_inputs.insert(input) {
                    relevant_inputs.push(input);
                }
            }
            for input in entry.provider.source_input_dependencies() {
                let dependency = (request.source(), input);
                if seen_source_inputs.insert(dependency) {
                    relevant_source_inputs.push(dependency);
                }
            }
        }
        let applicable: Vec<_> = registered
            .iter()
            .filter(|entry| entry.provider.supports(context))
            .cloned()
            .collect();
        match applicable.as_slice() {
            [] if self.is_compatibility_request(request, required_by) => {
                Err(PlanError::NoApplicableProvider {
                    product: request.product(),
                    required_by: required_by.map(ProductRequest::product),
                    registered: registered.iter().map(|entry| entry.id).collect(),
                })
            }
            [] => Err(PlanError::NoApplicableRequestProvider {
                request,
                required_by,
                registered: registered.iter().map(|entry| entry.id).collect(),
            }),
            [entry] => Ok((entry.clone(), relevant_inputs, relevant_source_inputs)),
            entries if self.is_compatibility_request(request, required_by) => {
                Err(PlanError::AmbiguousProvider {
                    product: request.product(),
                    required_by: required_by.map(ProductRequest::product),
                    applicable: entries.iter().map(|entry| entry.id).collect(),
                })
            }
            entries => Err(PlanError::AmbiguousRequestProvider {
                request,
                required_by,
                applicable: entries.iter().map(|entry| entry.id).collect(),
            }),
        }
    }

    fn missing_provider(
        &self,
        request: ProductRequest,
        required_by: Option<ProductRequest>,
    ) -> PlanError {
        if self.is_compatibility_request(request, required_by) {
            PlanError::MissingProvider {
                product: request.product(),
                required_by: required_by.map(ProductRequest::product),
            }
        } else {
            PlanError::MissingRequestProvider {
                request,
                required_by,
            }
        }
    }

    fn is_compatibility_request(
        &self,
        request: ProductRequest,
        required_by: Option<ProductRequest>,
    ) -> bool {
        request.source() == self.compatibility_source
            && required_by.is_none_or(|parent| parent.source() == self.compatibility_source)
    }

    fn cycle(&self, request: ProductRequest) -> PlanError {
        let start = self
            .stack
            .iter()
            .position(|item| *item == request)
            .unwrap_or(0);
        let mut path = self.stack[start..].to_vec();
        path.push(request);
        if path
            .iter()
            .all(|entry| entry.source() == self.compatibility_source)
        {
            PlanError::DependencyCycle {
                path: path.into_iter().map(ProductRequest::product).collect(),
            }
        } else {
            PlanError::RequestDependencyCycle { path }
        }
    }
}

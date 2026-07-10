use vize_atlas::{
    Compilation, ObservationKind, PlanningContext, Product, ProductId, ProductRequest,
    ProductStatus, Provider, ProviderContext, ProviderError, ProviderId, SourceRange,
};

struct Observed;

impl Product for Observed {
    type Value = ();
    const NAME: &'static str = "test.observed";
}

struct ObservedParent;

impl Product for ObservedParent {
    type Value = ();
    const NAME: &'static str = "test.observed-parent";
}

struct ObservedParentProvider;

impl Provider for ObservedParentProvider {
    type Product = ObservedParent;

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<Observed>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<(), ProviderError> {
        context.get::<Observed>().map(|_| ())
    }
}

struct ObservedProvider;

impl Provider for ObservedProvider {
    type Product = Observed;

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<(), ProviderError> {
        context.observe(
            ObservationKind::Diagnostic,
            "test.syntax",
            "synthetic diagnostic",
            Some(SourceRange::new(1, 3)),
        );
        context.observe(
            ObservationKind::Fallback,
            "test.recovery",
            "used recovery path",
            None,
        );
        Ok(())
    }
}

#[test]
fn provider_observations_keep_request_provenance_across_cache_hits() {
    let mut compilation = Compilation::new();
    compilation.register_provider(ObservedProvider).unwrap();
    let source = compilation.add_source("observed.ts", "abc").unwrap();
    let request = ProductRequest::for_product::<Observed>(source);

    let first = compilation.query::<Observed>(source).unwrap();
    assert_eq!(first.execution().observations().len(), 2);
    let diagnostic = &first.execution().observations()[0];
    assert_eq!(diagnostic.request(), request);
    assert_eq!(diagnostic.provider(), ProviderId::of::<ObservedProvider>());
    assert_eq!(diagnostic.source(), source);
    assert_eq!(diagnostic.range(), Some(SourceRange::new(1, 3)));
    assert_eq!(diagnostic.kind(), ObservationKind::Diagnostic);
    assert_eq!(diagnostic.code(), "test.syntax");

    let second = compilation.query::<Observed>(source).unwrap();
    assert_eq!(second.status(), ProductStatus::CacheHit);
    assert_eq!(
        second.execution().observations(),
        first.execution().observations()
    );
    assert_eq!(
        second.execution().observations_for_request(request).count(),
        2
    );
}

#[test]
fn cached_parent_restores_observations_from_its_pruned_dependency_closure() {
    let mut compilation = Compilation::new();
    compilation.register_provider(ObservedProvider).unwrap();
    compilation
        .register_provider(ObservedParentProvider)
        .unwrap();
    let source = compilation.add_source("observed.ts", "abc").unwrap();
    let observed = ProductRequest::for_product::<Observed>(source);

    let first = compilation.query::<ObservedParent>(source).unwrap();
    assert_eq!(first.status(), ProductStatus::Executed);
    assert_eq!(first.execution().observations().len(), 2);
    assert_eq!(
        first.execution().observations_for_request(observed).count(),
        2
    );

    let cached = compilation.query::<ObservedParent>(source).unwrap();
    assert_eq!(cached.status(), ProductStatus::CacheHit);
    assert_eq!(
        cached.execution().status_for_request(observed),
        Some(ProductStatus::Pruned)
    );
    assert_eq!(
        cached.execution().observations(),
        first.execution().observations()
    );
    assert_eq!(
        cached
            .execution()
            .observations_for_request(observed)
            .count(),
        2
    );
}

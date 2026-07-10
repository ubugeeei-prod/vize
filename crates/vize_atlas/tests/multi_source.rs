use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

use vize_atlas::{
    Compilation, PlanningContext, Product, ProductRequest, ProductStatus, Provider,
    ProviderContext, ProviderError, ProviderId, Shared, SourceId, SourceProvenance, SourceRange,
};
use vize_carton::{String, cstr};

struct Semantic;

impl Product for Semantic {
    type Value = String;
    const NAME: &'static str = "multi.semantic";
}

#[derive(Clone, Default)]
struct SemanticExecutions(Shared<Mutex<Vec<SourceId>>>);

struct SemanticProvider(SemanticExecutions);

impl Provider for SemanticProvider {
    type Product = Semantic;

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<String, ProviderError> {
        self.0.0.lock().unwrap().push(context.source().id());
        Ok(cstr!(
            "{}={}",
            context.source().name(),
            context.source().text()
        ))
    }
}

struct ProjectSummary;

impl Product for ProjectSummary {
    type Value = String;
    const NAME: &'static str = "multi.project-summary";
}

struct ProjectProvider {
    left: SourceId,
    right: SourceId,
    executions: Shared<AtomicUsize>,
}

impl Provider for ProjectProvider {
    type Product = ProjectSummary;

    fn dependency_requests(&self, context: &PlanningContext<'_>) -> Vec<ProductRequest> {
        // Cross-file planning can inspect the complete immutable source store.
        assert!(context.sources().iter().count() >= 4);
        assert!(context.source_by_id(self.left).is_some());
        vec![
            ProductRequest::for_product::<Semantic>(self.left),
            ProductRequest::for_product::<Semantic>(self.right),
        ]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<String, ProviderError> {
        self.executions.fetch_add(1, Ordering::Relaxed);
        let left = context.get_for_source::<Semantic>(self.left)?;
        let right = context.get_for_source::<Semantic>(self.right)?;
        Ok(cstr!("{left};{right}"))
    }
}

struct Unused;

impl Product for Unused {
    type Value = ();
    const NAME: &'static str = "multi.unused";
}

struct UnusedProvider(Shared<AtomicUsize>);

impl Provider for UnusedProvider {
    type Product = Unused;

    fn provide(&self, _context: &mut ProviderContext<'_>) -> Result<(), ProviderError> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

struct Fixture {
    compilation: Compilation,
    project: SourceId,
    left: SourceId,
    right: SourceId,
    unrelated: SourceId,
    semantic_executions: SemanticExecutions,
    project_executions: Shared<AtomicUsize>,
    unused_executions: Shared<AtomicUsize>,
}

fn fixture() -> Fixture {
    let mut compilation = Compilation::new();
    let project = compilation.add_source("project.json", "{}").unwrap();
    let left = compilation.add_source("left.ts", "one").unwrap();
    let right = compilation.add_source("right.ts", "two").unwrap();
    let unrelated = compilation.add_source("unrelated.ts", "three").unwrap();
    let semantic_executions = SemanticExecutions::default();
    let project_executions = Shared::new(AtomicUsize::new(0));
    let unused_executions = Shared::new(AtomicUsize::new(0));
    compilation
        .register_provider(SemanticProvider(semantic_executions.clone()))
        .unwrap();
    compilation
        .register_provider(ProjectProvider {
            left,
            right,
            executions: Shared::clone(&project_executions),
        })
        .unwrap();
    compilation
        .register_provider(UnusedProvider(Shared::clone(&unused_executions)))
        .unwrap();
    Fixture {
        compilation,
        project,
        left,
        right,
        unrelated,
        semantic_executions,
        project_executions,
        unused_executions,
    }
}

#[test]
fn project_request_shares_cross_source_products_and_invalidates_transitively() {
    let mut fixture = fixture();

    // Prime an unrelated third source. Its tree must survive edits elsewhere.
    fixture
        .compilation
        .query::<Semantic>(fixture.unrelated)
        .unwrap();
    let plan = fixture
        .compilation
        .plan_requests([ProductRequest::for_product::<ProjectSummary>(
            fixture.project,
        )])
        .unwrap();
    assert_eq!(plan.source_revisions().len(), 3);
    assert_eq!(
        plan.provider_for_request(ProductRequest::for_product::<ProjectSummary>(
            fixture.project
        )),
        Some(ProviderId::of::<ProjectProvider>())
    );
    assert!(plan.contains_for_source::<Semantic>(fixture.left));
    assert!(plan.contains_for_source::<Semantic>(fixture.right));
    assert!(!plan.contains::<Unused>());

    let first = fixture.compilation.execute(plan).unwrap();
    assert_eq!(
        &*first
            .get_for_source::<ProjectSummary>(fixture.project)
            .unwrap()
            .unwrap(),
        "left.ts=one;right.ts=two"
    );
    assert_eq!(
        &*first
            .get_for_source::<Semantic>(fixture.left)
            .unwrap()
            .unwrap(),
        "left.ts=one"
    );
    assert_eq!(fixture.project_executions.load(Ordering::Relaxed), 1);
    assert_eq!(fixture.unused_executions.load(Ordering::Relaxed), 0);
    let initial_semantics = fixture.semantic_executions.0.lock().unwrap().clone();
    assert_eq!(
        initial_semantics
            .iter()
            .filter(|source| **source == fixture.left)
            .count(),
        1
    );
    assert_eq!(
        initial_semantics
            .iter()
            .filter(|source| **source == fixture.right)
            .count(),
        1
    );

    let invalidation = fixture
        .compilation
        .update_source(fixture.left, "changed")
        .unwrap();
    assert!(invalidation.evicted().iter().any(|entry| {
        entry.source == fixture.left && entry.product == vize_atlas::ProductId::of::<Semantic>()
    }));
    assert!(invalidation.evicted().iter().any(|entry| {
        entry.source == fixture.project
            && entry.product == vize_atlas::ProductId::of::<ProjectSummary>()
    }));
    assert!(
        fixture
            .compilation
            .cache()
            .contains::<Semantic>(fixture.right)
    );
    assert!(
        fixture
            .compilation
            .cache()
            .contains::<Semantic>(fixture.unrelated)
    );

    let second = fixture
        .compilation
        .query::<ProjectSummary>(fixture.project)
        .unwrap();
    assert_eq!(second.value(), "left.ts=changed;right.ts=two");
    assert_eq!(fixture.project_executions.load(Ordering::Relaxed), 2);
    let executions = fixture.semantic_executions.0.lock().unwrap();
    assert_eq!(
        executions
            .iter()
            .filter(|source| **source == fixture.left)
            .count(),
        2
    );
    assert_eq!(
        executions
            .iter()
            .filter(|source| **source == fixture.right)
            .count(),
        1
    );
    drop(executions);
    assert_eq!(
        fixture
            .compilation
            .query::<Semantic>(fixture.unrelated)
            .unwrap()
            .status(),
        ProductStatus::CacheHit
    );
    assert_eq!(fixture.unused_executions.load(Ordering::Relaxed), 0);
}

#[test]
fn immutable_snapshot_preserves_provenance_and_forks_independently() {
    let mut fixture = fixture();
    fixture
        .compilation
        .query::<ProjectSummary>(fixture.project)
        .unwrap();
    let embedded = fixture
        .compilation
        .add_embedded_source(fixture.left, SourceRange::new(0, 3), "left.ts?part", "one")
        .unwrap();
    let snapshot = fixture.compilation.snapshot();
    let cheap_clone = snapshot.clone();
    let captured_left_revision = snapshot.source(fixture.left).unwrap().revision();
    let captured_embedded = snapshot.source(embedded).unwrap().clone();

    fixture
        .compilation
        .update_source(fixture.left, "changed")
        .unwrap();
    assert_eq!(
        snapshot.source(fixture.left).unwrap().revision(),
        captured_left_revision
    );
    assert_eq!(cheap_clone.source(embedded), Some(&captured_embedded));
    assert!(matches!(
        captured_embedded.provenance(),
        SourceProvenance::Embedded { parent, .. } if *parent == fixture.left
    ));

    let mut fork = snapshot.fork();
    assert_eq!(
        fork.query::<ProjectSummary>(fixture.project)
            .unwrap()
            .status(),
        ProductStatus::CacheHit
    );
    fork.update_source(fixture.right, "forked").unwrap();
    assert_eq!(
        fork.query::<ProjectSummary>(fixture.project)
            .unwrap()
            .value(),
        "left.ts=one;right.ts=forked"
    );
    assert_eq!(snapshot.source(fixture.right).unwrap().text(), "two");
    assert_eq!(
        fixture.compilation.source(fixture.right).unwrap().text(),
        "two"
    );
}

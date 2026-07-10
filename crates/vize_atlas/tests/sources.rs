use vize_atlas::{
    Compilation, InvalidationPolicy, PlanError, Product, ProductId, ProductStatus, Provider,
    ProviderContext, ProviderError, QueryError, SourceProvenance, SourceRange, SourceRevision,
};

struct Length;

impl Product for Length {
    type Value = usize;
    const NAME: &'static str = "source.length";
}

struct LengthProvider;

impl Provider for LengthProvider {
    type Product = Length;

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<usize, ProviderError> {
        Ok(context.source().text().len())
    }
}

#[test]
fn source_identity_is_stable_while_revision_advances() {
    let mut compilation = Compilation::new();
    let source = compilation.add_source("a.ts", "one").unwrap();
    let initial = compilation.source(source).unwrap().revision();
    let report = compilation.update_source(source, "two").unwrap();

    assert_eq!(initial, SourceRevision::INITIAL);
    assert_eq!(compilation.source(source).unwrap().id(), source);
    assert_eq!(compilation.source(source).unwrap().revision().get(), 2);
    assert_eq!(report.updated(), source);
    assert_eq!(report.revisions().len(), 1);
    assert_eq!(report.revisions()[0].source, source);
    assert_eq!(report.revisions()[0].previous, initial);
    assert_eq!(report.revisions()[0].current.get(), 2);
}

#[test]
fn embedded_provenance_tracks_the_exact_parent_snapshot() {
    let mut compilation = Compilation::new();
    let parent = compilation
        .add_source("component.vue", "<script>one</script>")
        .unwrap();
    let range = SourceRange::new(8, 11);
    let embedded = compilation
        .add_embedded_source(parent, range, "component.vue?script", "one")
        .unwrap();

    assert_eq!(
        compilation.source(embedded).unwrap().provenance(),
        &SourceProvenance::Embedded {
            parent,
            parent_revision: SourceRevision::INITIAL,
            range,
        }
    );
}

#[test]
fn source_update_invalidates_only_its_provenance_tree_and_records_policy() {
    let mut compilation = Compilation::new();
    compilation.register_provider(LengthProvider).unwrap();
    let parent = compilation
        .add_source("component.vue", "<script>one</script>")
        .unwrap();
    let embedded = compilation
        .add_embedded_source(
            parent,
            SourceRange::new(8, 11),
            "component.vue?script",
            "one",
        )
        .unwrap();
    let unrelated = compilation.add_source("other.ts", "other").unwrap();

    compilation.query::<Length>(parent).unwrap();
    compilation.query::<Length>(embedded).unwrap();
    compilation.query::<Length>(unrelated).unwrap();
    let report = compilation
        .update_source(parent, "<script>four</script>")
        .unwrap();

    assert_eq!(
        report.policy(),
        InvalidationPolicy::SourceAndEmbeddedDescendants
    );
    assert_eq!(report.revisions().len(), 2);
    assert_eq!(report.revisions()[0].source, parent);
    assert_eq!(report.revisions()[1].source, embedded);
    assert_eq!(report.evicted().len(), 2);
    assert!(
        report
            .evicted()
            .iter()
            .any(|entry| { entry.source == parent && entry.product == ProductId::of::<Length>() })
    );
    assert!(
        report.evicted().iter().any(|entry| {
            entry.source == embedded && entry.product == ProductId::of::<Length>()
        })
    );

    let unrelated_outcome = compilation.query::<Length>(unrelated).unwrap();
    assert_eq!(unrelated_outcome.status(), ProductStatus::CacheHit);
    let parent_outcome = compilation.query::<Length>(parent).unwrap();
    assert_eq!(parent_outcome.status(), ProductStatus::Executed);
}

#[test]
fn parent_update_requires_embedded_source_to_refresh_its_provenance() {
    let mut compilation = Compilation::new();
    compilation.register_provider(LengthProvider).unwrap();
    let parent = compilation
        .add_source("component.vue", "<script>one</script>")
        .unwrap();
    let embedded = compilation
        .add_embedded_source(
            parent,
            SourceRange::new(8, 11),
            "component.vue?script",
            "one",
        )
        .unwrap();
    compilation
        .update_source(parent, "<script>four</script>")
        .unwrap();

    assert!(matches!(
        compilation.plan_for::<Length>(embedded),
        Err(PlanError::StaleEmbeddedSource { source, parent: owner, .. })
            if source == embedded && owner == parent
    ));

    compilation
        .update_embedded_source(embedded, SourceRange::new(8, 12), "four")
        .unwrap();
    let outcome = compilation.query::<Length>(embedded).unwrap();
    assert_eq!(*outcome.value(), 4);
    assert_eq!(
        compilation.source(embedded).unwrap().provenance(),
        &SourceProvenance::Embedded {
            parent,
            parent_revision: compilation.source(parent).unwrap().revision(),
            range: SourceRange::new(8, 12),
        }
    );
}

#[test]
fn source_change_stales_a_preexisting_plan() {
    let mut compilation = Compilation::new();
    compilation.register_provider(LengthProvider).unwrap();
    let source = compilation.add_source("a.ts", "one").unwrap();
    let plan = compilation.plan_for::<Length>(source).unwrap();
    compilation.update_source(source, "two").unwrap();

    assert!(matches!(
        compilation.execute(plan),
        Err(QueryError::StaleSourcePlan { source: stale, .. }) if stale == source
    ));
}

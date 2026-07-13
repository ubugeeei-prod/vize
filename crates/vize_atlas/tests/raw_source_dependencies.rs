use vize_atlas::{Compilation, PlanningContext, Product, Provider, ProviderContext, SourceId};

struct ProjectText;

impl Product for ProjectText {
    type Value = usize;
    const NAME: &'static str = "test.project-text";
}

struct ProjectTextProvider;

impl Provider for ProjectTextProvider {
    type Product = ProjectText;

    fn source_dependencies(&self, context: &PlanningContext<'_>) -> Vec<SourceId> {
        context.sources().iter().map(|source| source.id()).collect()
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<usize, vize_atlas::ProviderError> {
        Ok(context
            .sources()
            .iter()
            .map(|source| source.text().len())
            .sum())
    }
}

#[test]
fn raw_project_source_change_evicts_the_aggregate_root() {
    let mut compilation = Compilation::new();
    compilation.register_provider(ProjectTextProvider).unwrap();
    let anchor = compilation.add_source("anchor", "a").unwrap();
    let dependency = compilation.add_source("dependency", "bb").unwrap();
    let first = compilation.query::<ProjectText>(anchor).unwrap();
    assert_eq!(*first.value(), 3);
    assert!(compilation.cache().contains::<ProjectText>(anchor));

    let invalidation = compilation.update_source(dependency, "cccc").unwrap();
    assert!(
        invalidation
            .evicted()
            .iter()
            .any(|entry| entry.source == anchor)
    );
    assert_eq!(
        *compilation.query::<ProjectText>(anchor).unwrap().value(),
        5
    );
}

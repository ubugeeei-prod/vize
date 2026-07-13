use vize_atlas::{
    Compilation, Product, ProductId, ProductStatus, Provider, ProviderContext, SourceInput,
    SourceInputId,
};

struct PerFileOptions;

impl SourceInput for PerFileOptions {
    type Value = u32;
    const NAME: &'static str = "test.per-file-options";
}

struct Leaf;

impl Product for Leaf {
    type Value = u32;
    const NAME: &'static str = "test.source-input-leaf";
}

struct LeafProvider;

impl Provider for LeafProvider {
    type Product = Leaf;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<PerFileOptions>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<u32, vize_atlas::ProviderError> {
        Ok(context
            .source_input::<PerFileOptions>()
            .copied()
            .unwrap_or_default())
    }
}

struct Root;

impl Product for Root {
    type Value = u32;
    const NAME: &'static str = "test.source-input-root";
}

struct RootProvider;

impl Provider for RootProvider {
    type Product = Root;

    fn dependencies(&self, _context: &vize_atlas::PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<Leaf>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<u32, vize_atlas::ProviderError> {
        Ok(*context.get::<Leaf>()?)
    }
}

#[test]
fn one_source_option_change_preserves_unrelated_cache_and_plan() {
    let mut compilation = Compilation::new();
    compilation.register_provider(LeafProvider).unwrap();
    compilation.register_provider(RootProvider).unwrap();
    let left = compilation.add_source("Left.vue", "left").unwrap();
    let right = compilation.add_source("Right.vue", "right").unwrap();
    compilation
        .set_source_input::<PerFileOptions>(left, 1)
        .unwrap();
    compilation
        .set_source_input::<PerFileOptions>(right, 2)
        .unwrap();

    assert_eq!(*compilation.query::<Root>(left).unwrap().value(), 1);
    assert_eq!(*compilation.query::<Root>(right).unwrap().value(), 2);
    let left_plan = compilation.plan_for::<Root>(left).unwrap();
    let right_plan = compilation.plan_for::<Root>(right).unwrap();

    let invalidation = compilation
        .set_source_input::<PerFileOptions>(left, 3)
        .unwrap();
    assert!(
        invalidation
            .evicted()
            .iter()
            .all(|item| item.source == left)
    );
    assert!(!compilation.cache().contains::<Leaf>(left));
    assert!(!compilation.cache().contains::<Root>(left));
    assert!(compilation.cache().contains::<Leaf>(right));
    assert!(compilation.cache().contains::<Root>(right));

    assert!(matches!(
        compilation.execute(left_plan),
        Err(vize_atlas::QueryError::StaleSourceInputPlan { source, .. }) if source == left
    ));
    let right_outcome = compilation.execute(right_plan).unwrap();
    assert_eq!(
        right_outcome.status(ProductId::of::<Root>()),
        Some(ProductStatus::CacheHit)
    );
    assert_eq!(*right_outcome.get::<Root>().unwrap().unwrap(), 2);
    assert_eq!(*compilation.query::<Root>(left).unwrap().value(), 3);
}

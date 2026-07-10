use vize_atlas::{
    Compilation, CompilationInput, InputId, InvalidationPolicy, PlanningContext, Product,
    ProductId, Provider, ProviderContext, ProviderError, QueryError,
};

#[path = "inputs/relevance.rs"]
mod relevance;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum TargetMode {
    Standard,
    Vapor,
}

struct TargetCapability;

impl CompilationInput for TargetCapability {
    type Value = TargetMode;
    const NAME: &'static str = "test.target-capability";
}

struct StandardLowering;
struct VaporLowering;
struct Output;

impl Product for StandardLowering {
    type Value = &'static str;
    const NAME: &'static str = "input.standard-lowering";
}

impl Product for VaporLowering {
    type Value = &'static str;
    const NAME: &'static str = "input.vapor-lowering";
}

impl Product for Output {
    type Value = &'static str;
    const NAME: &'static str = "input.output";
}

struct StandardProvider;
struct VaporProvider;

impl Provider for StandardProvider {
    type Product = StandardLowering;

    fn provide(&self, _context: &mut ProviderContext<'_>) -> Result<&'static str, ProviderError> {
        Ok("standard")
    }
}

impl Provider for VaporProvider {
    type Product = VaporLowering;

    fn provide(&self, _context: &mut ProviderContext<'_>) -> Result<&'static str, ProviderError> {
        Ok("vapor")
    }
}

struct OutputProvider;

impl Provider for OutputProvider {
    type Product = Output;

    fn input_dependencies(&self) -> Vec<InputId> {
        vec![InputId::of::<TargetCapability>()]
    }

    fn dependencies(&self, context: &PlanningContext<'_>) -> Vec<ProductId> {
        match context.input::<TargetCapability>() {
            Some(TargetMode::Vapor) => vec![ProductId::of::<VaporLowering>()],
            _ => vec![ProductId::of::<StandardLowering>()],
        }
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<&'static str, ProviderError> {
        match context.input::<TargetCapability>() {
            Some(TargetMode::Vapor) => context.get::<VaporLowering>().map(|value| *value),
            _ => context.get::<StandardLowering>().map(|value| *value),
        }
    }
}

fn compilation() -> Compilation {
    let mut compilation = Compilation::new();
    compilation.register_provider(StandardProvider).unwrap();
    compilation.register_provider(VaporProvider).unwrap();
    compilation.register_provider(OutputProvider).unwrap();
    compilation
}

#[test]
fn typed_inputs_shape_plans_without_an_atlas_domain_enum() {
    let mut compilation = compilation();
    let initial = compilation
        .set_input::<TargetCapability>(TargetMode::Standard)
        .unwrap();
    assert_eq!(initial.input(), InputId::of::<TargetCapability>());
    assert!(!initial.replaced());
    assert!(initial.evicted().is_empty());
    assert_eq!(
        compilation.input::<TargetCapability>(),
        Some(&TargetMode::Standard)
    );
    let source = compilation.add_source("a.vue", "<div />").unwrap();

    let standard = compilation.plan_for::<Output>(source).unwrap();
    assert!(standard.contains::<StandardLowering>());
    assert!(!standard.contains::<VaporLowering>());
    assert_eq!(
        compilation.query::<Output>(source).unwrap().value(),
        &"standard"
    );

    let stale_plan = compilation.plan_for::<Output>(source).unwrap();
    let invalidation = compilation
        .set_input::<TargetCapability>(TargetMode::Vapor)
        .unwrap();
    assert!(invalidation.replaced());
    assert_eq!(
        invalidation.policy(),
        InvalidationPolicy::CompilationInputDependents
    );
    assert_eq!(invalidation.evicted().len(), 1);
    assert_eq!(invalidation.evicted()[0].product, ProductId::of::<Output>());
    assert!(matches!(
        compilation.execute(stale_plan),
        Err(QueryError::StaleInputPlan { .. })
    ));

    let vapor = compilation.plan_for::<Output>(source).unwrap();
    assert!(!vapor.contains::<StandardLowering>());
    assert!(vapor.contains::<VaporLowering>());
    assert_eq!(
        compilation.query::<Output>(source).unwrap().value(),
        &"vapor"
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<StandardLowering>()
            .executions(),
        1
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<VaporLowering>()
            .executions(),
        1
    );
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum DialectMode {
    Vue2,
    Vue3,
}

struct DialectCapability;

impl CompilationInput for DialectCapability {
    type Value = DialectMode;
    const NAME: &'static str = "test.dialect-capability";
}

struct DialectObservation;
struct TargetObservation;
struct DialectSummary;

impl Product for DialectObservation {
    type Value = &'static str;
    const NAME: &'static str = "input.dialect-observation";
}

impl Product for TargetObservation {
    type Value = &'static str;
    const NAME: &'static str = "input.target-observation";
}

impl Product for DialectSummary {
    type Value = &'static str;
    const NAME: &'static str = "input.dialect-summary";
}

struct DialectObservationProvider;
struct TargetObservationProvider;
struct DialectSummaryProvider;

impl Provider for DialectObservationProvider {
    type Product = DialectObservation;

    fn input_dependencies(&self) -> Vec<InputId> {
        vec![InputId::of::<DialectCapability>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<&'static str, ProviderError> {
        Ok(match context.input::<DialectCapability>() {
            Some(DialectMode::Vue2) => "vue2",
            _ => "vue3",
        })
    }
}

impl Provider for TargetObservationProvider {
    type Product = TargetObservation;

    fn input_dependencies(&self) -> Vec<InputId> {
        vec![InputId::of::<TargetCapability>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<&'static str, ProviderError> {
        Ok(match context.input::<TargetCapability>() {
            Some(TargetMode::Vapor) => "vapor",
            _ => "standard",
        })
    }
}

impl Provider for DialectSummaryProvider {
    type Product = DialectSummary;

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<DialectObservation>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<&'static str, ProviderError> {
        context.get::<DialectObservation>().map(|dialect| *dialect)
    }
}

#[test]
fn independent_inputs_preserve_unrelated_plans_and_cached_products() {
    let mut compilation = Compilation::new();
    compilation
        .register_provider(DialectObservationProvider)
        .unwrap();
    compilation
        .register_provider(TargetObservationProvider)
        .unwrap();
    compilation
        .set_input::<DialectCapability>(DialectMode::Vue2)
        .unwrap();
    compilation
        .set_input::<TargetCapability>(TargetMode::Standard)
        .unwrap();
    let source = compilation.add_source("component.vue", "").unwrap();

    assert_eq!(
        compilation
            .query::<DialectObservation>(source)
            .unwrap()
            .value(),
        &"vue2"
    );
    assert_eq!(
        compilation
            .query::<TargetObservation>(source)
            .unwrap()
            .value(),
        &"standard"
    );
    let stale_dialect = compilation.plan_for::<DialectObservation>(source).unwrap();
    let reusable_target = compilation.plan_for::<TargetObservation>(source).unwrap();

    let dialect_invalidation = compilation
        .set_input::<DialectCapability>(DialectMode::Vue3)
        .unwrap();
    assert_eq!(dialect_invalidation.evicted().len(), 1);
    assert_eq!(
        dialect_invalidation.evicted()[0].product,
        ProductId::of::<DialectObservation>()
    );
    assert!(!compilation.cache().contains::<DialectObservation>(source));
    assert!(compilation.cache().contains::<TargetObservation>(source));
    assert!(matches!(
        compilation.execute(stale_dialect),
        Err(QueryError::StaleInputPlan { input, .. })
            if input == InputId::of::<DialectCapability>()
    ));
    assert_eq!(
        compilation
            .execute(reusable_target)
            .unwrap()
            .status(ProductId::of::<TargetObservation>()),
        Some(vize_atlas::ProductStatus::CacheHit)
    );

    assert_eq!(
        compilation
            .query::<DialectObservation>(source)
            .unwrap()
            .value(),
        &"vue3"
    );
    let reusable_dialect = compilation.plan_for::<DialectObservation>(source).unwrap();
    let stale_target = compilation.plan_for::<TargetObservation>(source).unwrap();

    let target_invalidation = compilation
        .set_input::<TargetCapability>(TargetMode::Vapor)
        .unwrap();
    assert_eq!(target_invalidation.evicted().len(), 1);
    assert_eq!(
        target_invalidation.evicted()[0].product,
        ProductId::of::<TargetObservation>()
    );
    assert!(compilation.cache().contains::<DialectObservation>(source));
    assert!(!compilation.cache().contains::<TargetObservation>(source));
    assert_eq!(
        compilation
            .execute(reusable_dialect)
            .unwrap()
            .status(ProductId::of::<DialectObservation>()),
        Some(vize_atlas::ProductStatus::CacheHit)
    );
    assert!(matches!(
        compilation.execute(stale_target),
        Err(QueryError::StaleInputPlan { input, .. })
            if input == InputId::of::<TargetCapability>()
    ));
}

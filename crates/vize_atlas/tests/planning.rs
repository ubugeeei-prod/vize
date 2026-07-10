use vize_atlas::{
    Compilation, PlanError, PlanningContext, Product, ProductId, Provider, ProviderContext,
    ProviderError, ProviderId, QueryError, RegisterProviderError,
};

struct Leaf;

impl Product for Leaf {
    type Value = usize;
    const NAME: &'static str = "plan.leaf";
}

struct LeafProvider;

impl Provider for LeafProvider {
    type Product = Leaf;

    fn provide(&self, _context: &mut ProviderContext<'_>) -> Result<usize, ProviderError> {
        Ok(1)
    }
}

struct MissingRoot;

impl Product for MissingRoot {
    type Value = ();
    const NAME: &'static str = "plan.missing-root";
}

#[test]
fn missing_root_provider_is_reported_before_execution() {
    let mut compilation = Compilation::new();
    let source = compilation.add_source("a.vue", "").unwrap();

    assert_eq!(
        compilation.plan_for::<MissingRoot>(source),
        Err(PlanError::MissingProvider {
            product: ProductId::of::<MissingRoot>(),
            required_by: None,
        })
    );
}

struct NeedsMissing;

impl Product for NeedsMissing {
    type Value = ();
    const NAME: &'static str = "plan.needs-missing";
}

struct NeedsMissingProvider;

impl Provider for NeedsMissingProvider {
    type Product = NeedsMissing;

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<MissingRoot>()]
    }

    fn provide(&self, _context: &mut ProviderContext<'_>) -> Result<(), ProviderError> {
        Ok(())
    }
}

#[test]
fn missing_dependency_names_its_consumer() {
    let mut compilation = Compilation::new();
    compilation.register_provider(NeedsMissingProvider).unwrap();
    let source = compilation.add_source("a.vue", "").unwrap();

    assert_eq!(
        compilation.plan_for::<NeedsMissing>(source),
        Err(PlanError::MissingProvider {
            product: ProductId::of::<MissingRoot>(),
            required_by: Some(ProductId::of::<NeedsMissing>()),
        })
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<NeedsMissing>()
            .executions(),
        0
    );
}

struct CycleA;
struct CycleB;

impl Product for CycleA {
    type Value = ();
    const NAME: &'static str = "plan.cycle-a";
}

impl Product for CycleB {
    type Value = ();
    const NAME: &'static str = "plan.cycle-b";
}

struct CycleAProvider;
struct CycleBProvider;

impl Provider for CycleAProvider {
    type Product = CycleA;

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<CycleB>()]
    }

    fn provide(&self, _context: &mut ProviderContext<'_>) -> Result<(), ProviderError> {
        Ok(())
    }
}

impl Provider for CycleBProvider {
    type Product = CycleB;

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<CycleA>()]
    }

    fn provide(&self, _context: &mut ProviderContext<'_>) -> Result<(), ProviderError> {
        Ok(())
    }
}

#[test]
fn dependency_cycles_include_the_closed_path() {
    let mut compilation = Compilation::new();
    compilation.register_provider(CycleAProvider).unwrap();
    compilation.register_provider(CycleBProvider).unwrap();
    let source = compilation.add_source("a.vue", "").unwrap();

    assert_eq!(
        compilation.plan_for::<CycleA>(source),
        Err(PlanError::DependencyCycle {
            path: vec![
                ProductId::of::<CycleA>(),
                ProductId::of::<CycleB>(),
                ProductId::of::<CycleA>(),
            ],
        })
    );
}

#[test]
fn empty_root_set_is_rejected() {
    let mut compilation = Compilation::new();
    let source = compilation.add_source("a.vue", "").unwrap();
    assert_eq!(
        compilation.plan(source, std::iter::empty()),
        Err(PlanError::NoRoots)
    );
}

#[test]
fn planning_has_no_provider_execution_cost() {
    let mut compilation = Compilation::new();
    compilation.register_provider(LeafProvider).unwrap();
    let source = compilation.add_source("a.vue", "").unwrap();

    let plan = compilation.plan_for::<Leaf>(source).unwrap();
    assert_eq!(plan.products(), [ProductId::of::<Leaf>()]);
    assert_eq!(compilation.counters().for_product::<Leaf>().executions(), 0);
    assert_eq!(compilation.counters().for_product::<Leaf>().queries(), 0);
}

#[test]
fn provider_registry_change_stales_an_existing_plan() {
    let mut compilation = Compilation::new();
    compilation.register_provider(LeafProvider).unwrap();
    let source = compilation.add_source("a.vue", "").unwrap();
    let plan = compilation.plan_for::<Leaf>(source).unwrap();
    compilation.register_provider(CycleAProvider).unwrap();

    assert!(matches!(
        compilation.execute(plan),
        Err(QueryError::StaleProviderPlan { .. })
    ));
}

#[test]
fn duplicate_provider_is_rejected() {
    let mut compilation = Compilation::new();
    compilation.register_provider(LeafProvider).unwrap();
    assert_eq!(
        compilation.register_provider(LeafProvider),
        Err(RegisterProviderError::DuplicateProvider {
            provider: ProviderId::of::<LeafProvider>(),
            product: ProductId::of::<Leaf>(),
        })
    );
}

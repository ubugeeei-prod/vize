use vize_atlas::{
    Compilation, PlanningContext, Product, ProductId, ProductStatus, Provider, ProviderContext,
    ProviderError, ProviderId, QueryError,
};
use vize_carton::{String, cstr};

struct Words;

impl Product for Words {
    type Value = usize;
    const NAME: &'static str = "test.words";
}

struct WordsProvider;

impl Provider for WordsProvider {
    type Product = Words;

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<usize, ProviderError> {
        Ok(context.source().text().split_whitespace().count())
    }
}

struct Diagnostics;

impl Product for Diagnostics {
    type Value = String;
    const NAME: &'static str = "test.diagnostics";
}

struct DiagnosticsProvider;

impl Provider for DiagnosticsProvider {
    type Product = Diagnostics;

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<Words>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<String, ProviderError> {
        Ok(cstr!("{} words", context.get::<Words>()?))
    }
}

struct Emit;

impl Product for Emit {
    type Value = String;
    const NAME: &'static str = "test.emit";
}

struct EmitProvider;

impl Provider for EmitProvider {
    type Product = Emit;

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<Words>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<String, ProviderError> {
        Ok(cstr!("emit:{}", context.get::<Words>()?))
    }
}

struct Unused;

impl Product for Unused {
    type Value = ();
    const NAME: &'static str = "test.unused";
}

struct UnusedProvider;

impl Provider for UnusedProvider {
    type Product = Unused;

    fn provide(&self, _context: &mut ProviderContext<'_>) -> Result<(), ProviderError> {
        Ok(())
    }
}

fn compilation() -> Compilation {
    let mut compilation = Compilation::new();
    compilation.register_provider(WordsProvider).unwrap();
    compilation.register_provider(DiagnosticsProvider).unwrap();
    compilation.register_provider(EmitProvider).unwrap();
    compilation.register_provider(UnusedProvider).unwrap();
    compilation
}

#[test]
fn two_roots_share_one_dependency_and_unused_provider_costs_zero() {
    let mut compilation = compilation();
    let source = compilation
        .add_source("component.vue", "one two three")
        .unwrap();
    let plan = compilation
        .plan(
            source,
            [ProductId::of::<Diagnostics>(), ProductId::of::<Emit>()],
        )
        .unwrap();

    assert_eq!(
        plan.products(),
        [
            ProductId::of::<Words>(),
            ProductId::of::<Diagnostics>(),
            ProductId::of::<Emit>(),
        ]
    );
    assert!(!plan.contains::<Unused>());
    assert_eq!(
        plan.dependencies(ProductId::of::<Diagnostics>()).unwrap(),
        [ProductId::of::<Words>()]
    );

    let outcome = compilation.execute(plan).unwrap();
    assert_eq!(&*outcome.get::<Diagnostics>().unwrap().unwrap(), "3 words");
    assert_eq!(&*outcome.get::<Emit>().unwrap().unwrap(), "emit:3");
    assert_eq!(
        compilation.counters().for_product::<Words>().executions(),
        1
    );
    assert_eq!(compilation.counters().for_product::<Words>().queries(), 2);
    assert_eq!(
        compilation.counters().for_product::<Unused>().executions(),
        0
    );
    assert_eq!(compilation.counters().for_product::<Unused>().queries(), 0);
}

#[test]
fn typed_query_reuses_the_dependency_closure_from_cache() {
    let mut compilation = compilation();
    let source = compilation.add_source("a.vue", "one two").unwrap();

    let first = compilation.query::<Diagnostics>(source).unwrap();
    assert_eq!(first.value(), "2 words");
    assert_eq!(first.status(), ProductStatus::Executed);
    assert!(first.trace().executed::<Words>());
    assert!(first.trace().executed::<Diagnostics>());
    assert_eq!(compilation.cache().len(), 2);
    assert!(compilation.cache().contains::<Words>(source));
    assert!(compilation.cache().contains::<Diagnostics>(source));
    assert!(!compilation.cache().contains::<Unused>(source));

    let second = compilation.query::<Diagnostics>(source).unwrap();
    assert_eq!(second.value(), "2 words");
    assert_eq!(second.status(), ProductStatus::CacheHit);
    assert_eq!(
        second.execution().status(ProductId::of::<Words>()),
        Some(ProductStatus::Pruned)
    );
    assert!(!second.trace().cache_hit::<Words>());
    assert!(second.trace().cache_hit::<Diagnostics>());
    assert_eq!(
        compilation
            .counters()
            .for_product::<Diagnostics>()
            .queries(),
        2
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<Diagnostics>()
            .executions(),
        1
    );
    assert_eq!(
        compilation.counters().for_product::<Words>().executions(),
        1
    );
}

struct Rogue;

impl Product for Rogue {
    type Value = usize;
    const NAME: &'static str = "test.rogue";
}

struct RogueProvider;

impl Provider for RogueProvider {
    type Product = Rogue;

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<usize, ProviderError> {
        Ok(*context.get::<Words>()?)
    }
}

#[test]
fn provider_cannot_escape_its_declared_dependency_closure() {
    let mut compilation = Compilation::new();
    compilation.register_provider(WordsProvider).unwrap();
    compilation.register_provider(RogueProvider).unwrap();
    let source = compilation.add_source("a.ts", "one").unwrap();

    let error = match compilation.query::<Rogue>(source) {
        Ok(_) => panic!("undeclared dependency must fail"),
        Err(error) => error,
    };
    assert_eq!(
        error,
        QueryError::ProviderFailed {
            source,
            product: ProductId::of::<Rogue>(),
            provider: ProviderId::of::<RogueProvider>(),
            error: Box::new(ProviderError::UndeclaredDependency {
                provider: ProviderId::of::<RogueProvider>(),
                dependency: ProductId::of::<Words>(),
            }),
        }
    );
    assert_eq!(
        compilation.counters().for_product::<Words>().executions(),
        0
    );
    assert_eq!(
        compilation.counters().for_product::<Rogue>().executions(),
        1
    );
}

struct SfcSyntax;
struct JsxSyntax;
struct Frontend;

impl Product for SfcSyntax {
    type Value = &'static str;
    const NAME: &'static str = "test.sfc-syntax";
}

impl Product for JsxSyntax {
    type Value = &'static str;
    const NAME: &'static str = "test.jsx-syntax";
}

impl Product for Frontend {
    type Value = &'static str;
    const NAME: &'static str = "test.frontend";
}

struct StaticProvider<P>(std::marker::PhantomData<P>);

impl<P> StaticProvider<P> {
    const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl Provider for StaticProvider<SfcSyntax> {
    type Product = SfcSyntax;

    fn provide(&self, _context: &mut ProviderContext<'_>) -> Result<&'static str, ProviderError> {
        Ok("sfc")
    }
}

impl Provider for StaticProvider<JsxSyntax> {
    type Product = JsxSyntax;

    fn provide(&self, _context: &mut ProviderContext<'_>) -> Result<&'static str, ProviderError> {
        Ok("jsx")
    }
}

struct FrontendProvider;

impl Provider for FrontendProvider {
    type Product = Frontend;

    fn dependencies(&self, context: &PlanningContext<'_>) -> Vec<ProductId> {
        if context.source().name().ends_with(".vue") {
            vec![ProductId::of::<SfcSyntax>()]
        } else {
            vec![ProductId::of::<JsxSyntax>()]
        }
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<&'static str, ProviderError> {
        if context.source().name().ends_with(".vue") {
            context.get::<SfcSyntax>()
        } else {
            context.get::<JsxSyntax>()
        }
        .map(|syntax| *syntax)
    }
}

#[test]
fn dependency_planning_is_source_aware() {
    let mut compilation = Compilation::new();
    compilation
        .register_provider(StaticProvider::<SfcSyntax>::new())
        .unwrap();
    compilation
        .register_provider(StaticProvider::<JsxSyntax>::new())
        .unwrap();
    compilation.register_provider(FrontendProvider).unwrap();
    let vue = compilation.add_source("a.vue", "<div />").unwrap();
    let tsx = compilation.add_source("a.tsx", "<div />").unwrap();

    let vue_plan = compilation.plan_for::<Frontend>(vue).unwrap();
    assert!(vue_plan.contains::<SfcSyntax>());
    assert!(!vue_plan.contains::<JsxSyntax>());
    let tsx_plan = compilation.plan_for::<Frontend>(tsx).unwrap();
    assert!(!tsx_plan.contains::<SfcSyntax>());
    assert!(tsx_plan.contains::<JsxSyntax>());

    assert_eq!(compilation.query::<Frontend>(vue).unwrap().value(), &"sfc");
    assert_eq!(compilation.query::<Frontend>(tsx).unwrap().value(), &"jsx");
}

use std::sync::atomic::{AtomicUsize, Ordering};

use vize_atlas::{
    Compilation, PlanError, PlanningContext, Product, ProductId, Provider, ProviderContext,
    ProviderError, ProviderId, Shared,
};

struct SfcSyntax;
struct JsxSyntax;
struct FrontendArtifact;

impl Product for SfcSyntax {
    type Value = &'static str;
    const NAME: &'static str = "alternatives.sfc-syntax";
}

impl Product for JsxSyntax {
    type Value = &'static str;
    const NAME: &'static str = "alternatives.jsx-syntax";
}

impl Product for FrontendArtifact {
    type Value = &'static str;
    const NAME: &'static str = "alternatives.frontend-artifact";
}

struct SfcSyntaxProvider;
struct JsxSyntaxProvider;

impl Provider for SfcSyntaxProvider {
    type Product = SfcSyntax;

    fn provide(&self, _context: &mut ProviderContext<'_>) -> Result<&'static str, ProviderError> {
        Ok("parsed-sfc")
    }
}

impl Provider for JsxSyntaxProvider {
    type Product = JsxSyntax;

    fn provide(&self, _context: &mut ProviderContext<'_>) -> Result<&'static str, ProviderError> {
        Ok("parsed-jsx")
    }
}

#[derive(Clone, Default)]
struct ProviderCosts {
    sfc_dependency_calls: Shared<AtomicUsize>,
    jsx_dependency_calls: Shared<AtomicUsize>,
    sfc_executions: Shared<AtomicUsize>,
    jsx_executions: Shared<AtomicUsize>,
}

struct SfcFrontendProvider(ProviderCosts);
struct JsxFrontendProvider(ProviderCosts);

impl Provider for SfcFrontendProvider {
    type Product = FrontendArtifact;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        context.source().name().ends_with(".vue")
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        self.0.sfc_dependency_calls.fetch_add(1, Ordering::Relaxed);
        vec![ProductId::of::<SfcSyntax>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<&'static str, ProviderError> {
        self.0.sfc_executions.fetch_add(1, Ordering::Relaxed);
        context.get::<SfcSyntax>().map(|syntax| *syntax)
    }
}

impl Provider for JsxFrontendProvider {
    type Product = FrontendArtifact;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        context.source().name().ends_with(".tsx")
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        self.0.jsx_dependency_calls.fetch_add(1, Ordering::Relaxed);
        vec![ProductId::of::<JsxSyntax>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<&'static str, ProviderError> {
        self.0.jsx_executions.fetch_add(1, Ordering::Relaxed);
        context.get::<JsxSyntax>().map(|syntax| *syntax)
    }
}

fn frontend_compilation() -> (Compilation, ProviderCosts) {
    let costs = ProviderCosts::default();
    let mut compilation = Compilation::new();
    compilation.register_provider(SfcSyntaxProvider).unwrap();
    compilation.register_provider(JsxSyntaxProvider).unwrap();
    compilation
        .register_provider(SfcFrontendProvider(costs.clone()))
        .unwrap();
    compilation
        .register_provider(JsxFrontendProvider(costs.clone()))
        .unwrap();
    (compilation, costs)
}

#[test]
fn peer_frontends_select_independent_providers_for_one_product() {
    let (mut compilation, costs) = frontend_compilation();
    let vue = compilation.add_source("component.vue", "<div />").unwrap();
    let tsx = compilation
        .add_source("component.tsx", "export const C = () => <div />")
        .unwrap();

    let vue_plan = compilation.plan_for::<FrontendArtifact>(vue).unwrap();
    assert_eq!(
        vue_plan.provider_for::<FrontendArtifact>(),
        Some(ProviderId::of::<SfcFrontendProvider>())
    );
    assert!(vue_plan.contains::<SfcSyntax>());
    assert!(!vue_plan.contains::<JsxSyntax>());
    assert_eq!(costs.sfc_dependency_calls.load(Ordering::Relaxed), 1);
    assert_eq!(costs.jsx_dependency_calls.load(Ordering::Relaxed), 0);

    let vue_output = compilation.execute(vue_plan).unwrap();
    assert_eq!(
        *vue_output.get::<FrontendArtifact>().unwrap().unwrap(),
        "parsed-sfc"
    );
    assert_eq!(costs.sfc_executions.load(Ordering::Relaxed), 1);
    assert_eq!(costs.jsx_executions.load(Ordering::Relaxed), 0);
    assert_eq!(
        compilation.counters().for_product::<JsxSyntax>().queries(),
        0
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<JsxSyntax>()
            .executions(),
        0
    );

    let tsx_plan = compilation.plan_for::<FrontendArtifact>(tsx).unwrap();
    assert_eq!(
        tsx_plan.provider_for::<FrontendArtifact>(),
        Some(ProviderId::of::<JsxFrontendProvider>())
    );
    assert!(!tsx_plan.contains::<SfcSyntax>());
    assert!(tsx_plan.contains::<JsxSyntax>());
    assert_eq!(costs.sfc_dependency_calls.load(Ordering::Relaxed), 1);
    assert_eq!(costs.jsx_dependency_calls.load(Ordering::Relaxed), 1);

    let tsx_output = compilation.execute(tsx_plan).unwrap();
    assert_eq!(
        *tsx_output.get::<FrontendArtifact>().unwrap().unwrap(),
        "parsed-jsx"
    );
    assert_eq!(costs.sfc_executions.load(Ordering::Relaxed), 1);
    assert_eq!(costs.jsx_executions.load(Ordering::Relaxed), 1);
}

#[test]
fn no_applicable_provider_is_structured_and_costs_no_dependencies() {
    let (mut compilation, costs) = frontend_compilation();
    let source = compilation.add_source("component.css", ".a {}").unwrap();

    assert_eq!(
        compilation.plan_for::<FrontendArtifact>(source),
        Err(PlanError::NoApplicableProvider {
            product: ProductId::of::<FrontendArtifact>(),
            required_by: None,
            registered: vec![
                ProviderId::of::<SfcFrontendProvider>(),
                ProviderId::of::<JsxFrontendProvider>(),
            ],
        })
    );
    assert_eq!(costs.sfc_dependency_calls.load(Ordering::Relaxed), 0);
    assert_eq!(costs.jsx_dependency_calls.load(Ordering::Relaxed), 0);
    assert_eq!(costs.sfc_executions.load(Ordering::Relaxed), 0);
    assert_eq!(costs.jsx_executions.load(Ordering::Relaxed), 0);
}

struct FirstAlwaysProvider;
struct SecondAlwaysProvider;

impl Provider for FirstAlwaysProvider {
    type Product = FrontendArtifact;

    fn provide(&self, _context: &mut ProviderContext<'_>) -> Result<&'static str, ProviderError> {
        Ok("first")
    }
}

impl Provider for SecondAlwaysProvider {
    type Product = FrontendArtifact;

    fn provide(&self, _context: &mut ProviderContext<'_>) -> Result<&'static str, ProviderError> {
        Ok("second")
    }
}

#[test]
fn multiple_applicable_providers_are_reported_without_execution() {
    let mut compilation = Compilation::new();
    compilation.register_provider(FirstAlwaysProvider).unwrap();
    compilation.register_provider(SecondAlwaysProvider).unwrap();
    let source = compilation.add_source("component.vue", "").unwrap();

    assert_eq!(
        compilation.plan_for::<FrontendArtifact>(source),
        Err(PlanError::AmbiguousProvider {
            product: ProductId::of::<FrontendArtifact>(),
            required_by: None,
            applicable: vec![
                ProviderId::of::<FirstAlwaysProvider>(),
                ProviderId::of::<SecondAlwaysProvider>(),
            ],
        })
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<FrontendArtifact>()
            .queries(),
        0
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<FrontendArtifact>()
            .executions(),
        0
    );
}

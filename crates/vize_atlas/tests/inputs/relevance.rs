use super::{
    Compilation, DialectCapability, DialectMode, DialectObservation, DialectObservationProvider,
    DialectSummary, DialectSummaryProvider, InputId, ProductId, QueryError,
};

#[test]
fn input_relevance_propagates_through_product_dependencies() {
    let mut compilation = Compilation::new();
    compilation
        .register_provider(DialectObservationProvider)
        .unwrap();
    compilation
        .register_provider(DialectSummaryProvider)
        .unwrap();
    compilation
        .set_input::<DialectCapability>(DialectMode::Vue2)
        .unwrap();
    let source = compilation.add_source("component.vue", "").unwrap();

    assert_eq!(
        compilation.query::<DialectSummary>(source).unwrap().value(),
        &"vue2"
    );
    let plan = compilation.plan_for::<DialectSummary>(source).unwrap();
    assert_eq!(
        plan.input_dependencies(ProductId::of::<DialectSummary>()),
        Some([InputId::of::<DialectCapability>()].as_slice())
    );

    let invalidation = compilation
        .set_input::<DialectCapability>(DialectMode::Vue3)
        .unwrap();
    assert_eq!(invalidation.evicted().len(), 2);
    assert!(!compilation.cache().contains::<DialectObservation>(source));
    assert!(!compilation.cache().contains::<DialectSummary>(source));
    assert!(matches!(
        compilation.execute(plan),
        Err(QueryError::StaleInputPlan { input, .. })
            if input == InputId::of::<DialectCapability>()
    ));
}

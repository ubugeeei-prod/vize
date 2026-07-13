use super::transform_lazy_hydration_macros;

#[test]
fn transforms_define_lazy_hydration_component() {
    let content = r#"const LazyHydrationMyComponent = defineLazyHydrationComponent(
  'visible',
  () => import('./components/MyComponent.vue'),
)
"#;

    let transformed =
        transform_lazy_hydration_macros(content).expect("macro should be transformed");

    assert!(!transformed.code.contains("defineLazyHydrationComponent"));
    assert!(
        transformed
            .code
            .contains("__vizeCreateLazyVisibleComponent")
    );
    assert!(
        transformed
            .code
            .contains("\"./components/MyComponent.vue\", () => import")
    );
    assert!(
        transformed
            .preamble
            .contains("hydrateOnVisible as __vizeHydrateOnVisible")
    );
    assert!(
        transformed
            .preamble
            .contains("const __vizeCreateLazyVisibleComponent")
    );
}

#[test]
fn ignores_dynamic_strategy_or_loader() {
    let content = r#"const strategy = 'visible'
const LazyHydrationMyComponent = defineLazyHydrationComponent(
  strategy,
  source,
)
"#;

    assert!(transform_lazy_hydration_macros(content).is_none());
}

#[test]
fn transforms_exported_variable_declarations() {
    let content = r#"export const LazyHydrationMyComponent = defineLazyHydrationComponent(
  'time',
  () => import('./components/MyComponent.vue'),
)
"#;

    let transformed =
        transform_lazy_hydration_macros(content).expect("macro should be transformed");

    assert!(!transformed.code.contains("defineLazyHydrationComponent"));
    assert!(transformed.code.contains("__vizeCreateLazyTimeComponent"));
    assert!(
        transformed
            .preamble
            .contains("const __vizeCreateLazyTimeComponent")
    );
}

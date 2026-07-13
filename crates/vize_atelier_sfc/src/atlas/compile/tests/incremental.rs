use super::*;

#[test]
fn output_affecting_source_override_invalidates_the_cached_module() {
    let source_text = "<template><surface /></template>";
    let mut compilation = Compilation::new();
    crate::register_atlas_providers(&mut compilation).unwrap();
    let source = compilation.add_source("Renderer.vue", source_text).unwrap();
    let mut settings = SfcCompileSettings::default();
    settings.insert(
        source,
        SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard),
    );
    settings.install(&mut compilation).unwrap();
    let standard = compilation
        .query::<SfcCompileProduct>(source)
        .unwrap()
        .value()
        .code
        .clone();
    assert_eq!(
        compilation
            .query::<SfcCompileProduct>(source)
            .unwrap()
            .status(),
        ProductStatus::CacheHit
    );

    let mut custom = SfcCompileOptions::default();
    custom.template.custom_renderer = true;
    settings.insert(
        source,
        SfcCompileRequest::new(custom, TemplateSyntaxMode::Standard),
    );
    settings.install(&mut compilation).unwrap();
    let custom = compilation.query::<SfcCompileProduct>(source).unwrap();

    assert_eq!(custom.status(), ProductStatus::Executed);
    assert_ne!(custom.value().code, standard);
}

#[test]
fn source_compile_request_update_retains_other_source_artifacts() {
    let mut compilation = Compilation::new();
    crate::register_atlas_providers(&mut compilation).unwrap();
    let source_text = r#"<script setup>const message = 'ready'</script>
<template><main>{{ message }}</main></template>"#;
    let first = compilation.add_source("First.vue", source_text).unwrap();
    let second = compilation.add_source("Second.vue", source_text).unwrap();
    let mut settings = SfcCompileSettings::default();
    for source in [first, second] {
        settings.insert(
            source,
            SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard),
        );
    }
    settings.install(&mut compilation).unwrap();
    compilation.query::<SfcCompileProduct>(first).unwrap();
    compilation.query::<SfcCompileProduct>(second).unwrap();
    assert!(compilation.cache().contains::<ReliefProduct>(second));
    assert!(
        compilation
            .cache()
            .contains::<CroquisDocumentProduct>(second)
    );

    let mut changed = SfcCompileOptions::default();
    changed.template.custom_renderer = true;
    install_sfc_compile_request(
        &mut compilation,
        first,
        SfcCompileRequest::new(changed, TemplateSyntaxMode::Strict),
    )
    .unwrap();

    assert!(compilation.cache().contains::<SfcCompileProduct>(second));
    assert!(compilation.cache().contains::<ReliefProduct>(second));
    assert!(
        compilation
            .cache()
            .contains::<CroquisDocumentProduct>(second)
    );
    assert_eq!(
        compilation
            .query::<SfcCompileProduct>(second)
            .unwrap()
            .status(),
        ProductStatus::CacheHit
    );
    assert!(!compilation.cache().contains::<SfcCompileProduct>(first));
    assert!(!compilation.cache().contains::<ReliefProduct>(first));
}

#[test]
fn style_setting_changes_only_rerun_final_sfc_assembly() {
    let source_text = r#"<script setup>const message = 'ready'</script>
<template><main>{{ message }}</main></template>
<style>.ready { color: green }</style>"#;
    let mut compilation = Compilation::new();
    crate::register_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source("StyleOnly.vue", source_text)
        .unwrap();
    install_sfc_compile_request(
        &mut compilation,
        source,
        SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard),
    )
    .unwrap();
    compilation.query::<SfcCompileProduct>(source).unwrap();

    let descriptor = executions::<SfcDescriptorProduct>(&compilation);
    let relief = executions::<ReliefProduct>(&compilation);
    let transformed = executions::<TransformedReliefProduct>(&compilation);
    let croquis = executions::<CroquisDocumentProduct>(&compilation);
    let rendu = executions::<RenduProduct>(&compilation);
    let backend = executions::<DomOutputProduct>(&compilation);
    let assembly = executions::<SfcCompileProduct>(&compilation);

    let mut options = SfcCompileOptions::default();
    options.style.trim = true;
    install_sfc_compile_request(
        &mut compilation,
        source,
        SfcCompileRequest::new(options, TemplateSyntaxMode::Standard),
    )
    .unwrap();
    compilation.query::<SfcCompileProduct>(source).unwrap();

    assert_eq!(executions::<SfcDescriptorProduct>(&compilation), descriptor);
    assert_eq!(executions::<ReliefProduct>(&compilation), relief);
    assert_eq!(
        executions::<TransformedReliefProduct>(&compilation),
        transformed
    );
    assert_eq!(executions::<CroquisDocumentProduct>(&compilation), croquis);
    assert_eq!(executions::<RenduProduct>(&compilation), rendu);
    assert_eq!(executions::<DomOutputProduct>(&compilation), backend);
    assert_eq!(executions::<SfcCompileProduct>(&compilation), assembly + 1);
}

#[test]
fn backend_target_change_reuses_descriptor_and_template_frontend() {
    let source_text = r#"<script setup>const message = 'ready'</script>
<template><main>{{ message }}</main></template>"#;
    let mut compilation = Compilation::new();
    crate::register_atlas_providers(&mut compilation).unwrap();
    let source = compilation.add_source("Target.vue", source_text).unwrap();
    install_sfc_compile_request(
        &mut compilation,
        source,
        SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard),
    )
    .unwrap();
    compilation.query::<SfcCompileProduct>(source).unwrap();

    let descriptor = executions::<SfcDescriptorProduct>(&compilation);
    let relief = executions::<ReliefProduct>(&compilation);
    let transformed = executions::<TransformedReliefProduct>(&compilation);
    let croquis = executions::<CroquisDocumentProduct>(&compilation);
    let rendu = executions::<RenduProduct>(&compilation);
    let render_module = executions::<SfcRenderModuleProduct>(&compilation);

    let mut options = SfcCompileOptions::default();
    options.template.ssr = true;
    install_sfc_compile_request(
        &mut compilation,
        source,
        SfcCompileRequest::new(options, TemplateSyntaxMode::Standard),
    )
    .unwrap();
    let outcome = compilation.query::<SfcCompileProduct>(source).unwrap();

    assert!(outcome.value().code.contains("ssrRender"));
    assert_eq!(executions::<SfcDescriptorProduct>(&compilation), descriptor);
    assert_eq!(executions::<ReliefProduct>(&compilation), relief);
    assert_eq!(
        executions::<TransformedReliefProduct>(&compilation),
        transformed
    );
    assert_eq!(executions::<CroquisDocumentProduct>(&compilation), croquis);
    assert_eq!(executions::<RenduProduct>(&compilation), rendu);
    assert_eq!(executions::<DomOutputProduct>(&compilation), 1);
    assert_eq!(executions::<SsrOutputProduct>(&compilation), 1);
    assert_eq!(
        executions::<SfcRenderModuleProduct>(&compilation),
        render_module + 1
    );
}

#[test]
fn source_vapor_attribute_change_replans_without_reinstalling_settings() {
    let mut compilation = Compilation::new();
    crate::register_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source(
            "SourceMode.vue",
            "<script setup>const ready = true</script><template>{{ ready }}</template>",
        )
        .unwrap();
    install_sfc_compile_request(
        &mut compilation,
        source,
        SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard),
    )
    .unwrap();
    let dom = compilation.query::<SfcCompileProduct>(source).unwrap();
    assert!(dom.plan().contains::<DomOutputProduct>());

    compilation
        .update_source(
            source,
            "<script setup vapor>const ready = true</script><template>{{ ready }}</template>",
        )
        .unwrap();
    let vapor = compilation.query::<SfcCompileProduct>(source).unwrap();

    assert!(vapor.plan().contains::<VaporOutputProduct>());
    assert!(!vapor.plan().contains::<DomOutputProduct>());
    assert!(vapor.value().code.contains("__vapor = true"));
}

fn executions<P: vize_atlas::Product>(compilation: &Compilation) -> u64 {
    compilation.counters().for_product::<P>().executions()
}

#[test]
fn one_snapshot_compiles_source_specific_requests_in_parallel_sessions() {
    let cases = cases();
    let mut compilation = Compilation::new();
    crate::register_atlas_providers(&mut compilation).unwrap();
    let (sources, settings) = add_cases(&mut compilation, &cases);
    settings.install(&mut compilation).unwrap();
    let snapshot = compilation.snapshot();

    std::thread::scope(|scope| {
        let handles: Vec<_> = sources
            .into_iter()
            .map(|source| {
                let mut session = snapshot.query_session();
                scope.spawn(move || {
                    let outcome = session.query::<SfcCompileProduct>(source).unwrap();
                    assert_eq!(outcome.status(), ProductStatus::Executed);
                    assert!(!outcome.value().code.is_empty());
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
    });

    let mut verifier = snapshot.query_session();
    for source in snapshot.sources().iter().map(|source| source.id()) {
        assert_eq!(
            verifier
                .query::<SfcCompileProduct>(source)
                .unwrap()
                .status(),
            ProductStatus::CacheHit
        );
    }
}

#[test]
fn shared_syntax_rejects_descriptor_artifact_presence_mismatches() {
    let descriptor = parse_sfc("<template><div /></template>", Default::default()).unwrap();
    let error = crate::compile_sfc_with_shared_syntax(
        &descriptor,
        SfcCompileOptions::default(),
        TemplateSyntaxMode::Standard,
        None,
    )
    .unwrap_err();
    assert_eq!(
        error.code.as_deref(),
        Some("INCONSISTENT_TEMPLATE_ARTIFACTS")
    );
}

#[test]
fn malformed_sfc_diagnostic_is_cached_while_neutral_dependencies_stay_queryable() {
    let source_text = "<template><div /></template><template><span /></template>";
    let mut compilation = Compilation::new();
    crate::register_atlas_providers(&mut compilation).unwrap();
    let source = compilation.add_source("Broken.vue", source_text).unwrap();

    let parsed = compilation.query::<SfcDescriptorProduct>(source).unwrap();
    let diagnostic = parsed.value().diagnostic().expect("cached SFC diagnostic");
    assert_eq!(parsed.status(), ProductStatus::Executed);
    assert_eq!(diagnostic.code.as_deref(), Some("DUPLICATE_TEMPLATE"));
    assert!(diagnostic.loc.is_some());
    assert!(parsed.value().descriptor().is_none());
    assert!(parsed.value().as_result().is_err());
    assert_eq!(
        compilation
            .query::<SfcDescriptorProduct>(source)
            .unwrap()
            .status(),
        ProductStatus::CacheHit
    );
    assert!(
        compilation
            .query::<SfcTemplateProduct>(source)
            .unwrap()
            .value()
            .is_none()
    );
    assert!(
        compilation
            .query::<ReliefProduct>(source)
            .unwrap()
            .value()
            .is_none()
    );
    compilation
        .query::<CroquisDocumentProduct>(source)
        .expect("malformed SFC has a neutral Croquis product");
    assert!(compilation.query::<SfcCompileProduct>(source).is_err());
    assert_eq!(
        compilation
            .counters()
            .for_product::<SfcDescriptorProduct>()
            .executions(),
        1
    );
}

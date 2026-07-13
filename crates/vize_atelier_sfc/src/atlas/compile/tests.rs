//! Production graph tests for the Atlas SFC compile product.

use vize_atelier_dom::{DomCompilerOptions, DomOutputProduct};
use vize_atelier_ssr::SsrOutputProduct;
use vize_atelier_vapor::{VaporOutputProduct, VaporPlanProduct};
use vize_atlas::{Compilation, ProductId, ProductStatus, SourceId};
use vize_croquis::CroquisDocumentProduct;
use vize_relief::TemplateSyntaxMode;
use vize_relief::{ReliefProduct, TransformedReliefProduct};
use vize_rendu::RenduProduct;

use super::*;
use crate::{SfcCompileOptions, SfcTemplateProduct, parse_sfc};

#[path = "tests/scoped_inference.rs"]
mod scoped_inference;

struct Case {
    name: &'static str,
    source: &'static str,
    request: SfcCompileRequest,
}

fn dom_options() -> DomCompilerOptions {
    DomCompilerOptions {
        comments: true,
        source_map: true,
        experimental_in_tag_comments: true,
        ..Default::default()
    }
}

fn cases() -> Vec<Case> {
    let mut template_only = SfcCompileOptions::default();
    template_only.parse.source_map = true;
    template_only.template.custom_renderer = true;
    template_only.template.compiler_options = Some(dom_options());

    let mut options_api = SfcCompileOptions::default();
    options_api.scope_id = Some("a1b2c3d4".into());
    options_api.template.compiler_options = Some(dom_options());

    let mut script_setup = SfcCompileOptions::default();
    script_setup.script.is_ts = true;
    script_setup.template.is_ts = true;
    script_setup.style.source_map = true;
    script_setup.template.compiler_options = Some(dom_options());

    let mut ssr = SfcCompileOptions::default();
    ssr.template.ssr = true;
    ssr.template.ssr_css_vars = Some("{ color: theme }".into());
    ssr.template.compiler_options = Some(dom_options());

    let mut vapor = SfcCompileOptions::default();
    vapor.vapor = true;
    vapor.template.custom_renderer = true;
    vapor.template.compiler_options = Some(dom_options());

    vec![
        Case {
            name: "TemplateOnly.vue",
            source: "<template><!--keep--><widget :value=\"count\">{{ count }}</widget></template>",
            request: SfcCompileRequest::new(template_only, TemplateSyntaxMode::Standard),
        },
        Case {
            name: "Recoverable.vue",
            source: "<template><div id=\"first\" id=\"second\" /></template>",
            request: SfcCompileRequest::new(
                SfcCompileOptions::default(),
                TemplateSyntaxMode::Standard,
            ),
        },
        Case {
            name: "OptionsApi.vue",
            source: r#"<script>export default { data: () => ({ count: 1 }) }</script>
<template><button @click="count++">{{ count }}</button></template>
<style scoped>.button { color: red }</style>"#,
            request: SfcCompileRequest::new(options_api, TemplateSyntaxMode::Quirks),
        },
        Case {
            name: "ScriptSetup.vue",
            source: r#"<script setup lang="ts">const msg: string = 'hello'</script>
<template><main class="app">{{ msg }}</main></template>
<style scoped>.app { display: grid }</style>"#,
            request: SfcCompileRequest::new(script_setup, TemplateSyntaxMode::Standard)
                .with_inferred_scoped_from_descriptor(),
        },
        Case {
            name: "Server.vue",
            source: r#"<script setup>const title = 'server'</script>
<template><!--ssr--><article>{{ title }}</article></template>"#,
            request: SfcCompileRequest::new(ssr, TemplateSyntaxMode::Strict),
        },
        Case {
            name: "Vapor.vue",
            source: r#"<script setup>let count = 0</script>
<template><surface @click="count++">{{ count }}</surface></template>"#,
            request: SfcCompileRequest::new(vapor, TemplateSyntaxMode::Standard),
        },
        Case {
            name: "ScriptOnly.vue",
            source: "<script setup lang=\"ts\">export const answer: number = 42</script>",
            request: SfcCompileRequest::new(
                SfcCompileOptions::default(),
                TemplateSyntaxMode::Strict,
            ),
        },
    ]
}

fn add_cases(compilation: &mut Compilation, cases: &[Case]) -> (Vec<SourceId>, SfcCompileSettings) {
    let mut sources = Vec::with_capacity(cases.len());
    let mut settings = SfcCompileSettings::default();
    for case in cases {
        let source = compilation.add_source(case.name, case.source).unwrap();
        settings.insert(source, case.request.clone());
        sources.push(source);
    }
    (sources, settings)
}

#[test]
fn multi_source_product_assembles_every_target_from_rendu() {
    let cases = cases();
    let mut compilation = Compilation::new();
    crate::register_atlas_providers(&mut compilation).unwrap();
    let (sources, settings) = add_cases(&mut compilation, &cases);
    settings.install(&mut compilation).unwrap();

    for (case, source) in cases.iter().zip(sources) {
        let outcome = compilation.query::<SfcCompileProduct>(source).unwrap();
        let actual = outcome.value();

        assert_eq!(outcome.status(), ProductStatus::Executed, "{}", case.name);
        assert!(
            actual.code.contains("export default _sfc_main"),
            "{}",
            case.name
        );
        let has_template = !matches!(case.name, "ScriptOnly.vue");
        assert_eq!(outcome.plan().contains::<ReliefProduct>(), has_template);
        assert_eq!(
            outcome.plan().contains::<TransformedReliefProduct>(),
            has_template
        );
        assert_eq!(outcome.plan().contains::<RenduProduct>(), has_template);
        let expected_target = match case.name {
            "Server.vue" => Some(SfcRenderTarget::Ssr),
            "Vapor.vue" => Some(SfcRenderTarget::Vapor),
            "ScriptOnly.vue" => None,
            _ => Some(SfcRenderTarget::Dom),
        };
        assert_eq!(
            outcome.plan().contains::<DomOutputProduct>(),
            expected_target == Some(SfcRenderTarget::Dom)
        );
        assert_eq!(
            outcome.plan().contains::<SsrOutputProduct>(),
            expected_target == Some(SfcRenderTarget::Ssr)
        );
        assert_eq!(
            outcome.plan().contains::<VaporOutputProduct>(),
            expected_target == Some(SfcRenderTarget::Vapor)
        );
        assert_eq!(
            outcome.plan().contains::<VaporPlanProduct>(),
            expected_target == Some(SfcRenderTarget::Vapor)
        );
        let needs_semantics = case.source.contains("<script") && has_template;
        assert_eq!(
            outcome.plan().contains::<CroquisDocumentProduct>(),
            needs_semantics
        );
        match case.name {
            "TemplateOnly.vue" => {
                assert!(actual.code.contains("value: _ctx.count"), "{}", actual.code);
                assert!(actual.code.contains("_toDisplayString(_ctx.count)"));
                assert!(actual.map.is_some());
            }
            "Recoverable.vue" => {
                assert!(
                    actual
                        .warnings
                        .iter()
                        .any(|warning| { warning.code.as_deref() == Some("DuplicateAttribute") })
                );
            }
            "OptionsApi.vue" => {
                assert!(actual.code.contains("\"onClick\""));
                assert!(
                    actual
                        .css
                        .as_deref()
                        .is_some_and(|css| css.contains("data-v-a1b2c3d4"))
                );
            }
            "ScriptSetup.vue" => {
                assert!(actual.code.contains("$setup.msg"));
                assert!(actual.code.contains("$props, $setup, $data, $options"));
                assert!(actual.bindings.is_some());
            }
            "Server.vue" => {
                assert!(actual.code.contains("export function ssrRender"));
                assert!(actual.code.contains("$setup.title"));
            }
            "Vapor.vue" => {
                assert!(actual.code.contains("_template("));
                assert!(actual.code.contains("$setup.count"));
                assert!(actual.code.contains("__vapor = true"));
            }
            "ScriptOnly.vue" => assert!(!actual.code.contains("function render")),
            _ => unreachable!(),
        }
        let source_map = compilation.query::<SfcSourceMapProduct>(source).unwrap();
        assert_eq!(source_map.value(), &actual.map, "{}", case.name);
        assert!(source_map.plan().contains::<SfcCompileProduct>());
        assert_eq!(source_map.plan().contains::<RenduProduct>(), has_template);
    }
}

#[test]
fn one_rendu_module_can_feed_all_registered_backend_products() {
    let mut compilation = Compilation::new();
    crate::register_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source(
            "Shared.vue",
            "<template><section>{{ message }}</section></template>",
        )
        .unwrap();
    let plan = compilation
        .plan(
            source,
            [
                ProductId::of::<DomOutputProduct>(),
                ProductId::of::<SsrOutputProduct>(),
                ProductId::of::<VaporOutputProduct>(),
            ],
        )
        .unwrap();
    assert!(plan.contains::<RenduProduct>());
    let output = compilation.execute(plan).unwrap();
    assert!(output.get::<DomOutputProduct>().unwrap().is_some());
    assert!(output.get::<SsrOutputProduct>().unwrap().is_some());
    assert!(output.get::<VaporOutputProduct>().unwrap().is_some());
    assert_eq!(
        compilation
            .counters()
            .for_product::<RenduProduct>()
            .executions(),
        1
    );
}

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
    compilation
        .set_source_input::<SfcCompileSettingsInput>(
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

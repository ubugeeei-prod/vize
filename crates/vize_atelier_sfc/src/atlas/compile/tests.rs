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
use crate::SfcTemplateBindingsProduct;

pub(super) fn register_compile_test_providers(compilation: &mut vize_atlas::Compilation) {
    crate::register_atlas_providers(compilation).unwrap();
    vize_atelier_dom::register_atlas_provider(compilation).unwrap();
    vize_atelier_ssr::register_atlas_provider(compilation).unwrap();
    vize_atelier_vapor::register_atlas_provider(compilation).unwrap();
}
use crate::{SfcCompileOptions, SfcTemplateProduct, parse_sfc};

#[path = "tests/bindings.rs"]
mod bindings;
#[path = "tests/builtins.rs"]
mod builtins;
#[path = "tests/incremental.rs"]
mod incremental;
#[path = "tests/runtime_names.rs"]
mod runtime_names;
#[path = "tests/scoped_inference.rs"]
mod scoped_inference;
#[path = "tests/scoped_ssr.rs"]
mod scoped_ssr;
#[path = "tests/slots.rs"]
mod slots;
#[path = "tests/ssr_semantics.rs"]
mod ssr_semantics;

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
    register_compile_test_providers(&mut compilation);
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
        let needs_semantics = matches!(
            case.name,
            "OptionsApi.vue" | "ScriptSetup.vue" | "Server.vue" | "Vapor.vue"
        );
        assert_eq!(
            outcome.plan().contains::<SfcTemplateBindingsProduct>(),
            needs_semantics
        );
        assert!(
            !outcome.plan().contains::<CroquisDocumentProduct>(),
            "production compile must not require the full semantic document"
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
    register_compile_test_providers(&mut compilation);
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
fn art_source_preserves_runtime_macro_calls_in_compiled_script() {
    let mut compilation = Compilation::new();
    register_compile_test_providers(&mut compilation);
    let source = compilation
        .add_source(
            "Button.art.vue#inspector",
            r#"<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });
</script>"#,
        )
        .unwrap();
    let mut options = SfcCompileOptions::default();
    options.parse.filename = "Button.art.vue#inspector".into();
    options.script.id = Some("Button.art.vue#inspector".into());
    options.script.is_ts = true;
    let mut settings = SfcCompileSettings::default();
    settings.insert(
        source,
        SfcCompileRequest::new(options, TemplateSyntaxMode::Standard),
    );
    settings.install(&mut compilation).unwrap();

    let output = compilation.query::<SfcCompileProduct>(source).unwrap();

    assert!(
        output.value().code.contains("defineArt(\"./Button.vue\""),
        "{}",
        output.value().code
    );
}

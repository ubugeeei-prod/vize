use vize_atelier_sfc::{
    SfcCompileOptions, SfcCompileRequest, SfcCompileSettings, SfcDescriptorProduct,
    TemplateCompileOptions,
};
use vize_atlas::{ObservationKind, ProductStatus};
use vize_carton::config::VueVersion;
use vize_croquis::CroquisDocumentProduct;
use vize_relief::TemplateSyntaxMode;
use vize_relief::{ReliefProduct, VueDialectInput};

use super::*;

#[test]
fn descriptor_executes_once_and_document_reuses_it_without_fallback() {
    let graph = SfcAnalysisGraph::new(
        [(
            "App.vue",
            "<script setup>const x=1</script><template>{{x}}</template>",
        )],
        SfcCroquisMode::Full,
    )
    .unwrap();
    let source = graph.source("App.vue");
    let mut session = graph.snapshot.query_session();
    let descriptor = session.query::<SfcDescriptorProduct>(source).unwrap();
    let document = session.query::<CroquisDocumentProduct>(source).unwrap();

    assert_eq!(descriptor.status(), ProductStatus::Executed);
    assert!(document.trace().cache_hit::<SfcDescriptorProduct>());
    assert!(document.plan().contains::<ReliefProduct>());
    assert_eq!(
        document
            .execution()
            .observations()
            .iter()
            .filter(|observation| observation.kind() == ObservationKind::Fallback)
            .count(),
        0
    );
}

#[test]
fn declaration_mode_skips_relief_and_caches_malformed_diagnostic() {
    let declaration = SfcAnalysisGraph::new(
        [("Types.vue", "<script setup lang=\"ts\">defineProps<{x:string}>()</script><template>{{x}}</template>")],
        SfcCroquisMode::Declaration,
    )
    .unwrap();
    let source = declaration.source("Types.vue");
    let document = declaration
        .snapshot
        .query_session()
        .query::<CroquisDocumentProduct>(source)
        .unwrap();
    assert!(!document.plan().contains::<ReliefProduct>());

    let malformed = SfcAnalysisGraph::new(
        [("Broken.vue", "<template /><template />")],
        SfcCroquisMode::Full,
    )
    .unwrap();
    assert!(malformed.query("Broken.vue").is_err());
    let mut session = malformed.snapshot.query_session();
    assert_eq!(
        session
            .query::<SfcDescriptorProduct>(malformed.source("Broken.vue"))
            .unwrap()
            .status(),
        ProductStatus::CacheHit
    );
}

#[test]
fn ffi_vue_versions_configure_the_compile_plan_before_any_query() {
    for (raw, expected) in [
        ("1", VueVersion::V1),
        ("2", VueVersion::V2),
        ("2.7", VueVersion::V2_7),
        ("3", VueVersion::V3),
    ] {
        let dialect = resolve_vue_version(Some(raw)).unwrap();
        assert_eq!(dialect, expected);
        let mut compilation = Compilation::new();
        vize_atelier_sfc::register_atlas_providers(&mut compilation).unwrap();
        let source = compilation
            .add_source(
                "Dialect.vue",
                "<template><p>{{ msg | capitalize }}</p></template>",
            )
            .unwrap();
        let request = SfcCompileRequest::new(
            SfcCompileOptions {
                template: TemplateCompileOptions {
                    dialect,
                    ..Default::default()
                },
                ..Default::default()
            },
            TemplateSyntaxMode::Standard,
        );
        let mut settings = SfcCompileSettings::default();
        settings.insert(source, request);
        settings.install(&mut compilation).unwrap();
        compilation.set_input::<VueDialectInput>(dialect).unwrap();
        let snapshot = compilation.snapshot();
        assert_eq!(snapshot.inputs().get::<VueDialectInput>(), Some(&expected));

        let artifacts = query_sfc_compile(&snapshot, source).unwrap();
        assert!(artifacts.compiled().is_ok(), "vueVersion={raw}");
        assert_eq!(artifacts.descriptor_executions, 1, "vueVersion={raw}");
        assert!(artifacts.descriptor_cache_hits >= 1, "vueVersion={raw}");
        assert_eq!(artifacts.fallback_observations, 0, "vueVersion={raw}");
        assert!(artifacts.compile_depends_on_dialect, "vueVersion={raw}");
        assert!(artifacts.render_cache_hit, "vueVersion={raw}");
        #[cfg(feature = "legacy")]
        if expected.is_legacy() {
            assert!(
                artifacts
                    .compiled()
                    .unwrap()
                    .code
                    .contains("_filter_capitalize"),
                "legacy filter rewrite for vueVersion={raw}"
            );
        } else {
            assert!(
                !artifacts
                    .compiled()
                    .unwrap()
                    .code
                    .contains("_filter_capitalize"),
                "Vue 3 keeps the JavaScript pipe expression"
            );
        }
    }
}

#[test]
fn ffi_vue_version_rejects_ambiguous_and_unknown_values() {
    assert!(resolve_vue_version(Some("0")).is_err());
    assert!(resolve_vue_version(Some("latest")).is_err());
    assert_eq!(resolve_vue_version(None).unwrap(), VueVersion::V3);
}

#[cfg(feature = "wasm")]
#[test]
fn vapor_render_product_preserves_static_templates_for_wasm() {
    let mut compilation = Compilation::new();
    vize_atelier_sfc::register_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source("Vapor.vue", "<template><main>Hello</main></template>")
        .unwrap();
    let request = SfcCompileRequest::new(
        SfcCompileOptions {
            vapor: true,
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
    );
    let mut settings = SfcCompileSettings::default();
    settings.insert(source, request);
    settings.install(&mut compilation).unwrap();

    let artifacts = query_sfc_compile(&compilation.snapshot(), source).unwrap();
    let render = artifacts.render().unwrap().unwrap();
    assert_eq!(render.target(), vize_atelier_sfc::SfcRenderTarget::Vapor);
    assert_eq!(render.templates().unwrap(), ["<main>Hello</main>"]);
}

#[test]
fn ffi_sfc_hosts_do_not_call_shadow_template_compilers() {
    let wasm = include_str!("../wasm/sfc_compile.rs");
    let napi = include_str!("../napi/sfc/compile.rs");
    assert!(!wasm.contains("compile_internal"));
    assert!(!wasm.contains("compile_sfc_with_"));
    assert!(!napi.contains("compile_sfc_with_"));
    assert!(!napi.contains("parse_sfc("));
}

#[test]
fn ffi_typecheck_is_parity_preserving_and_plans_full_shared_artifacts() {
    let source = r#"<script setup lang="ts">
const count = 1
</script><template>{{ count }} {{ missing }}</template>"#;
    let options = vize_canon::SfcTypeCheckOptions::new("Typed.vue").with_virtual_ts();
    let expected = vize_canon::type_check_sfc(source, &options);
    let graph = SfcTypeCheckGraph::new(
        vec![("Typed.vue".into(), source.into(), options)],
        SfcCroquisMode::Full,
    )
    .unwrap();
    let id = graph.sources["Typed.vue"];
    let mut session = graph.snapshot.query_session();
    let outcome = session
        .query::<vize_canon::SfcTypeCheckProduct>(id)
        .unwrap();

    assert!(outcome.plan().contains::<SfcDescriptorProduct>());
    assert!(outcome.plan().contains::<ReliefProduct>());
    assert!(outcome.plan().contains::<CroquisDocumentProduct>());
    assert_eq!(outcome.value().error_count, expected.error_count);
    assert_eq!(outcome.value().warning_count, expected.warning_count);
    assert_eq!(outcome.value().virtual_ts, expected.virtual_ts);
    assert_eq!(
        outcome
            .value()
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.code.as_deref(), diagnostic.start, diagnostic.end))
            .collect::<Vec<_>>(),
        expected
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.code.as_deref(), diagnostic.start, diagnostic.end))
            .collect::<Vec<_>>()
    );
}

#[test]
fn ffi_typecheck_batch_shares_one_compilation_without_shadow_parsers() {
    let sources = vec![
        (
            "One.vue".into(),
            "<script setup>const one = 1</script><template>{{ one }}</template>".into(),
            vize_canon::SfcTypeCheckOptions::new("One.vue"),
        ),
        (
            "Two.vue".into(),
            "<script setup>const two = 2</script><template>{{ two }}</template>".into(),
            vize_canon::SfcTypeCheckOptions::new("Two.vue"),
        ),
    ];
    let graph = SfcTypeCheckGraph::new(sources, SfcCroquisMode::Full).unwrap();

    assert_eq!(graph.sources.len(), 2);
    assert_eq!(graph.query("One.vue").unwrap().error_count, 0);
    assert_eq!(graph.query("Two.vue").unwrap().error_count, 0);
    let napi = include_str!("../napi_typecheck.rs");
    let wasm = include_str!("../wasm_typecheck.rs");
    assert!(!napi.contains("type_check_sfc(&"));
    assert!(!napi.contains("parse_sfc("));
    assert!(!wasm.contains("type_check_sfc("));
    assert!(!wasm.contains("parse_sfc("));
}

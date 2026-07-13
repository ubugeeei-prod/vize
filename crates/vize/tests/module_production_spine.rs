use vize::artifact_graph::{VizeGraphConfig, create_compilation};
use vize::atelier_sfc::compile_script::{
    legacy_from_source_compile_invocations, reset_legacy_from_source_compile_invocations,
};
use vize::atelier_sfc::{
    SfcCompileProduct, SfcCompileRequest, SfcCroquisMode, SfcScriptSyntaxProduct,
    authored_script_parse_invocations, install_sfc_compile_request,
    reset_authored_script_parse_invocations,
};
use vize::canon::virtual_ts::{
    authored_script_fallback_parse_invocations, reset_authored_script_fallback_parse_invocations,
};
use vize::canon::{
    SfcTypeCheckOptions, SfcTypeCheckProduct, SfcTypeCheckRequest, install_sfc_typecheck_request,
};
use vize::croquis_cf::{
    CrossFileAnalysisInput, CrossFileAnalysisProduct, CrossFileAnalysisRequest, CrossFileOptions,
};
use vize::flow::FlowProduct;
use vize::module::ModuleSyntaxProduct;
use vize::patina::PatinaDocumentReportProduct;
use vize::relief::ReliefProduct;
use vize_atlas::{ProductId, ProductRequest, ProductStatus};

const SCRIPTED: &str = r#"<script setup lang="ts">
import { ref } from 'vue'
const count = ref(0)
</script>
<template><button @click="count++">{{ count }}</button></template>"#;

const TEMPLATE_ONLY: &str = "<template><p>{{ message }}</p></template>";
const SCRIPT_ONLY: &str = "<script lang=\"ts\">export { computed } from 'vue'</script>";

const NORMAL_ONLY: &str = r#"<script lang="ts">
interface State { count: number }
export default { data(): State { return { count: 1 } } }
</script><template>{{ count }}</template>"#;

const DUAL_SCRIPT: &str = r#"<script lang="ts">
export interface Props { title: string }
export default { inheritAttrs: false }
</script><script setup lang="ts">
const props = defineProps<Props>()
</script><template>{{ props.title }}</template>"#;

#[test]
fn compiler_lint_and_typecheck_share_one_script_and_module_frontend() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source = compilation.add_source("Shared.vue", SCRIPTED).unwrap();
    install_requests(&mut compilation, source);
    let plan = compilation
        .plan(
            source,
            [
                ProductId::of::<SfcCompileProduct>(),
                ProductId::of::<PatinaDocumentReportProduct>(),
                ProductId::of::<SfcTypeCheckProduct>(),
            ],
        )
        .unwrap();

    assert!(plan.contains::<SfcScriptSyntaxProduct>());
    assert!(plan.contains::<ModuleSyntaxProduct>());
    assert!(!plan.contains::<FlowProduct>());
    let output = compilation.execute(plan).unwrap();
    assert!(output.get::<SfcCompileProduct>().unwrap().is_some());
    assert!(
        output
            .get::<PatinaDocumentReportProduct>()
            .unwrap()
            .is_some()
    );
    assert!(output.get::<SfcTypeCheckProduct>().unwrap().is_some());
    assert_eq!(
        compilation
            .counters()
            .for_product::<SfcScriptSyntaxProduct>()
            .executions(),
        1
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<ModuleSyntaxProduct>()
            .executions(),
        1
    );
}

#[test]
fn cross_file_compiler_lint_and_typecheck_share_modules_across_sources() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source = compilation.add_source("Shared.vue", SCRIPTED).unwrap();
    let raw = compilation
        .add_source("state.ts", "export const ready = true")
        .unwrap();
    install_requests(&mut compilation, source);
    compilation
        .set_input::<CrossFileAnalysisInput>(CrossFileAnalysisRequest::new(
            CrossFileOptions::minimal(),
        ))
        .unwrap();

    let plan = compilation
        .plan(
            source,
            [
                ProductId::of::<SfcCompileProduct>(),
                ProductId::of::<PatinaDocumentReportProduct>(),
                ProductId::of::<SfcTypeCheckProduct>(),
                ProductId::of::<CrossFileAnalysisProduct>(),
            ],
        )
        .unwrap();
    let first = compilation.execute(plan).unwrap();

    assert!(first.get::<SfcCompileProduct>().unwrap().is_some());
    assert!(
        first
            .get::<PatinaDocumentReportProduct>()
            .unwrap()
            .is_some()
    );
    assert!(first.get::<SfcTypeCheckProduct>().unwrap().is_some());
    assert!(first.get::<CrossFileAnalysisProduct>().unwrap().is_some());
    assert_eq!(
        compilation
            .counters()
            .for_product::<ModuleSyntaxProduct>()
            .executions(),
        2,
        "the SFC script and raw module must each execute exactly once"
    );
    assert!(
        first
            .trace()
            .executed_for_source::<ModuleSyntaxProduct>(source)
    );
    assert!(
        first
            .trace()
            .executed_for_source::<ModuleSyntaxProduct>(raw)
    );

    compilation
        .update_source(raw, "export const ready = false")
        .unwrap();
    let plan = compilation
        .plan(
            source,
            [
                ProductId::of::<SfcCompileProduct>(),
                ProductId::of::<PatinaDocumentReportProduct>(),
                ProductId::of::<SfcTypeCheckProduct>(),
                ProductId::of::<CrossFileAnalysisProduct>(),
            ],
        )
        .unwrap();
    let revised = compilation.execute(plan).unwrap();

    assert_eq!(
        revised.status(ProductId::of::<SfcCompileProduct>()),
        Some(ProductStatus::CacheHit)
    );
    assert_eq!(
        revised.status(ProductId::of::<PatinaDocumentReportProduct>()),
        Some(ProductStatus::CacheHit)
    );
    assert_eq!(
        revised.status(ProductId::of::<SfcTypeCheckProduct>()),
        Some(ProductStatus::CacheHit)
    );
    assert_eq!(
        revised.status(ProductId::of::<CrossFileAnalysisProduct>()),
        Some(ProductStatus::Executed)
    );
    assert_eq!(
        revised.status_for_request(ProductRequest::for_product::<ModuleSyntaxProduct>(source)),
        Some(ProductStatus::CacheHit)
    );
    assert_eq!(
        revised.status_for_request(ProductRequest::for_product::<ModuleSyntaxProduct>(raw)),
        Some(ProductStatus::Executed)
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<ModuleSyntaxProduct>()
            .executions(),
        3,
        "only the changed raw source may re-execute Module"
    );
}

#[test]
fn template_only_combined_roots_never_plan_module_syntax() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source = compilation
        .add_source("TemplateOnly.vue", TEMPLATE_ONLY)
        .unwrap();
    install_requests(&mut compilation, source);
    let plan = compilation
        .plan(
            source,
            [
                ProductId::of::<SfcCompileProduct>(),
                ProductId::of::<PatinaDocumentReportProduct>(),
                ProductId::of::<SfcTypeCheckProduct>(),
            ],
        )
        .unwrap();

    assert!(!plan.contains::<SfcScriptSyntaxProduct>());
    assert!(!plan.contains::<ModuleSyntaxProduct>());
    assert!(!plan.contains::<FlowProduct>());
    compilation.execute(plan).unwrap();
    assert_eq!(
        compilation
            .counters()
            .for_product::<ModuleSyntaxProduct>()
            .executions(),
        0
    );
}

#[test]
fn script_only_compiler_lint_and_typecheck_roots_never_plan_relief() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source = compilation
        .add_source("ScriptOnly.vue", SCRIPT_ONLY)
        .unwrap();
    install_requests(&mut compilation, source);
    let plan = compilation
        .plan(
            source,
            [
                ProductId::of::<SfcCompileProduct>(),
                ProductId::of::<PatinaDocumentReportProduct>(),
                ProductId::of::<SfcTypeCheckProduct>(),
            ],
        )
        .unwrap();

    assert!(!plan.contains::<ReliefProduct>());
    assert!(plan.contains::<SfcScriptSyntaxProduct>());
    assert!(plan.contains::<ModuleSyntaxProduct>());
    compilation.execute(plan).unwrap();
    assert_eq!(
        compilation
            .counters()
            .for_product::<ModuleSyntaxProduct>()
            .executions(),
        1
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<ReliefProduct>()
            .executions(),
        0
    );
}

#[test]
fn combined_roots_parse_each_authored_script_block_once() {
    for (name, source_text, mode, expected_parses) in [
        ("NormalOnly.vue", NORMAL_ONLY, SfcCroquisMode::Full, 1),
        (
            "NormalOptions.vue",
            NORMAL_ONLY,
            SfcCroquisMode::OptionsApi,
            1,
        ),
        (
            "NormalLegacy.vue",
            NORMAL_ONLY,
            SfcCroquisMode::LegacyVue2,
            1,
        ),
        ("SetupOnly.vue", SCRIPTED, SfcCroquisMode::Full, 1),
        ("DualScript.vue", DUAL_SCRIPT, SfcCroquisMode::Full, 2),
    ] {
        reset_authored_script_parse_invocations();
        reset_legacy_from_source_compile_invocations();
        reset_authored_script_fallback_parse_invocations();
        let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
        let source = compilation.add_source(name, source_text).unwrap();
        install_sfc_compile_request(&mut compilation, source, SfcCompileRequest::default())
            .unwrap();
        install_sfc_typecheck_request(
            &mut compilation,
            source,
            SfcTypeCheckRequest::new(SfcTypeCheckOptions::new(name).with_virtual_ts(), mode),
        )
        .unwrap();
        let plan = compilation
            .plan(
                source,
                [
                    ProductId::of::<SfcCompileProduct>(),
                    ProductId::of::<PatinaDocumentReportProduct>(),
                    ProductId::of::<SfcTypeCheckProduct>(),
                ],
            )
            .unwrap();

        compilation.execute(plan).unwrap();
        assert_eq!(
            authored_script_parse_invocations(),
            expected_parses,
            "{name} must parse each authored script block exactly once"
        );
        assert_eq!(
            legacy_from_source_compile_invocations(),
            0,
            "{name} must compile from its cached ScriptCompileContext"
        );
        assert_eq!(
            authored_script_fallback_parse_invocations(),
            0,
            "{name} must generate Canon virtual TS from cached script facts"
        );
    }
}

fn install_requests(compilation: &mut vize_atlas::Compilation, source: vize_atlas::SourceId) {
    install_sfc_compile_request(compilation, source, SfcCompileRequest::default()).unwrap();
    install_sfc_typecheck_request(
        compilation,
        source,
        SfcTypeCheckRequest::new(
            SfcTypeCheckOptions::new("Shared.vue").with_virtual_ts(),
            SfcCroquisMode::Full,
        ),
    )
    .unwrap();
}

use vize::artifact_graph::{VizeGraphConfig, create_compilation};
use vize::croquis_cf::{
    CroquisProjectProduct, CrossFileAnalysisInput, CrossFileAnalysisProduct,
    CrossFileAnalysisRequest, CrossFileAnalyzer, CrossFileOffsetRegion, CrossFileOptions,
};
use vize_atlas::{ProductRequest, ProductStatus};
use vize_croquis::{
    CroquisDocumentProduct, EffectGraphScript, build_effect_graph_from_sfc_scripts,
};

const APP: &str = r#"<script setup lang="ts">
import Child from './Child.vue'
import { provide } from 'vue'
provide('theme', 'dark')
</script>
<template><Child /></template>"#;

const CHILD: &str = r#"<script setup lang="ts">
import { inject } from 'vue'
const theme = inject('theme')
</script>
<template><div>{{ theme }}</div></template>"#;

#[test]
fn full_cross_file_analysis_is_a_real_opt_in_product() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let app = compilation.add_source("src/App.vue", APP).unwrap();
    let child = compilation.add_source("src/Child.vue", CHILD).unwrap();
    compilation
        .add_source("src/store.ts", "export const ready = true")
        .unwrap();
    compilation
        .set_input::<CrossFileAnalysisInput>(CrossFileAnalysisRequest::new(
            CrossFileOptions::minimal().with_provide_inject(true),
        ))
        .unwrap();

    let plan = compilation
        .plan_for::<CrossFileAnalysisProduct>(app)
        .unwrap();
    assert!(plan.contains::<CrossFileAnalysisProduct>());
    assert!(plan.contains::<CroquisDocumentProduct>());
    assert!(!plan.contains::<CroquisProjectProduct>());

    let outcome = compilation.query::<CrossFileAnalysisProduct>(app).unwrap();
    let artifact = outcome.value();
    assert_eq!(artifact.result().stats.files_analyzed, 3);
    assert_eq!(artifact.layouts().len(), 3);
    assert!(artifact.provide_inject_tree().is_some());

    let child_layout = artifact.layout_for_source(child).unwrap();
    let script_offset = CHILD.find("import").unwrap() as u32;
    let template_offset = CHILD.find("<div>").unwrap() as u32;
    assert_eq!(
        child_layout.map_offset(CrossFileOffsetRegion::Script, 1),
        script_offset
    );
    assert_eq!(
        child_layout.map_offset(CrossFileOffsetRegion::Template, 0),
        template_offset
    );
}

#[test]
fn one_source_change_reuses_unrelated_croquis_documents() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let app = compilation.add_source("src/App.vue", APP).unwrap();
    let child = compilation.add_source("src/Child.vue", CHILD).unwrap();
    compilation
        .set_input::<CrossFileAnalysisInput>(CrossFileAnalysisRequest::new(
            CrossFileOptions::minimal().with_provide_inject(true),
        ))
        .unwrap();
    compilation.query::<CrossFileAnalysisProduct>(app).unwrap();

    compilation
        .update_source(child, CHILD.replace("theme", "palette"))
        .unwrap();
    let second = compilation.query::<CrossFileAnalysisProduct>(app).unwrap();
    assert_eq!(second.status(), ProductStatus::Executed);
    assert_eq!(
        second
            .execution()
            .status_for_request(ProductRequest::for_product::<CroquisDocumentProduct>(app)),
        Some(ProductStatus::CacheHit)
    );
    assert_eq!(
        second
            .execution()
            .status_for_request(ProductRequest::for_product::<CroquisDocumentProduct>(child)),
        Some(ProductStatus::Executed)
    );
}

#[test]
fn split_script_layout_projects_each_synthetic_segment_to_its_block() {
    const SPLIT: &str = r#"<script lang="ts">
export const plain: number = 1
</script>
<script setup lang="tsx">
const setup = <span>{plain}</span>
</script>
<template><div>{{ plain }}</div></template>"#;
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source = compilation.add_source("src/Split.vue", SPLIT).unwrap();
    compilation
        .set_input::<CrossFileAnalysisInput>(CrossFileAnalysisRequest::new(
            CrossFileOptions::minimal(),
        ))
        .unwrap();

    let artifact = compilation
        .query::<CrossFileAnalysisProduct>(source)
        .unwrap();
    let layout = artifact.value().layout_for_source(source).unwrap();
    let plain = SPLIT.find("export const plain").unwrap() as u32;
    let setup = SPLIT.find("const setup").unwrap() as u32;
    let plain_len = "\nexport const plain: number = 1\n".len() as u32;

    assert_eq!(layout.map_offset(CrossFileOffsetRegion::Script, 1), plain);
    assert_eq!(
        layout.map_offset(CrossFileOffsetRegion::Script, plain_len + 2),
        setup
    );
    assert_eq!(
        layout.map_offset(CrossFileOffsetRegion::Template, 0),
        SPLIT.find("<div>").unwrap() as u32
    );
}

#[test]
fn split_script_effect_summary_preserves_each_declared_language() {
    const SCRIPT: &str = r#"
import { ref, type Ref } from 'vue'
const count: Ref<number> = ref(0)
"#;
    const SETUP: &str = r#"
import { computed } from 'vue'
const doubled = computed(() => count.value * 2)
const node = <span>{doubled.value}</span>
"#;
    let source =
        format!("<script lang=\"ts\">{SCRIPT}</script><script setup lang=\"tsx\">{SETUP}</script>");
    let expected = build_effect_graph_from_sfc_scripts(
        Some(EffectGraphScript::new(SCRIPT, Some("ts"))),
        Some(EffectGraphScript::new(SETUP, Some("tsx"))),
    )
    .summary();
    assert!(expected.edge_count > 0);

    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source_id = compilation.add_source("src/Reactive.vue", source).unwrap();
    compilation
        .set_input::<CrossFileAnalysisInput>(CrossFileAnalysisRequest::new(
            CrossFileOptions::minimal(),
        ))
        .unwrap();
    let artifact = compilation
        .query::<CrossFileAnalysisProduct>(source_id)
        .unwrap();
    let input = &artifact.value().result().complexity_report.input;

    assert_eq!(input.reactive_node_count, expected.node_count);
    assert_eq!(input.reactive_edge_count, expected.edge_count);
    assert_eq!(input.reactive_cycle_count, expected.cycle_count);
}

#[test]
fn raw_module_changes_invalidate_only_the_project_product() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let app = compilation.add_source("src/App.vue", APP).unwrap();
    let raw = compilation
        .add_source("src/store.ts", "export const ready = true")
        .unwrap();
    let ignored = compilation.add_source("README.md", "one").unwrap();
    compilation
        .set_input::<CrossFileAnalysisInput>(CrossFileAnalysisRequest::new(
            CrossFileOptions::minimal(),
        ))
        .unwrap();
    compilation.query::<CrossFileAnalysisProduct>(app).unwrap();

    compilation.update_source(ignored, "two").unwrap();
    assert_eq!(
        compilation
            .query::<CrossFileAnalysisProduct>(app)
            .unwrap()
            .status(),
        ProductStatus::CacheHit
    );

    compilation
        .update_source(raw, "export const ready = false")
        .unwrap();
    let revised = compilation.query::<CrossFileAnalysisProduct>(app).unwrap();
    assert_eq!(revised.status(), ProductStatus::Executed);
    assert_eq!(
        revised
            .execution()
            .status_for_request(ProductRequest::for_product::<CroquisDocumentProduct>(app)),
        Some(ProductStatus::CacheHit)
    );
}

#[test]
fn full_analysis_remains_unexecuted_until_requested() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let app = compilation.add_source("src/App.vue", APP).unwrap();

    compilation.query::<CroquisDocumentProduct>(app).unwrap();

    assert_eq!(
        compilation
            .counters()
            .for_product::<CrossFileAnalysisProduct>()
            .executions(),
        0
    );
    assert!(
        !compilation
            .cache()
            .contains::<CrossFileAnalysisProduct>(app)
    );
}

#[test]
fn empty_malformed_and_frontend_neutral_inputs_are_total() {
    let mut empty = create_compilation(VizeGraphConfig::default()).unwrap();
    let anchor = empty.add_source("<cross-file-anchor>", "").unwrap();
    empty
        .set_input::<CrossFileAnalysisInput>(CrossFileAnalysisRequest::new(
            CrossFileOptions::minimal(),
        ))
        .unwrap();
    let result = empty.query::<CrossFileAnalysisProduct>(anchor).unwrap();
    assert_eq!(result.value().result().stats.files_analyzed, 0);
    assert!(result.value().layouts().is_empty());

    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let malformed = compilation
        .add_source(
            "Malformed.vue",
            "<template><div /></template><template><span /></template>",
        )
        .unwrap();
    let jsx = compilation
        .add_source("View.jsx", "export const View = () => <main />")
        .unwrap();
    let tsx = compilation
        .add_source(
            "Typed.tsx",
            "export const Typed = (p: { n: number }) => <b>{p.n}</b>",
        )
        .unwrap();
    compilation
        .set_input::<CrossFileAnalysisInput>(CrossFileAnalysisRequest::new(
            CrossFileOptions::minimal(),
        ))
        .unwrap();
    let artifact = compilation
        .query::<CrossFileAnalysisProduct>(malformed)
        .unwrap();
    assert_eq!(artifact.value().result().stats.files_analyzed, 3);
    for source in [malformed, jsx, tsx] {
        let layout = artifact.value().layout_for_source(source).unwrap();
        assert_eq!(layout.source(), source);
    }
    assert_eq!(
        artifact
            .value()
            .layout_for_source(jsx)
            .unwrap()
            .map_offset(CrossFileOffsetRegion::Script, 12),
        12
    );
}

#[test]
fn atlas_raw_module_result_matches_the_direct_analyzer() {
    let options = CrossFileOptions::minimal()
        .with_provide_inject(true)
        .with_circular_dependencies(true);
    let files = [
        (
            "src/parent.ts",
            "import { provide } from 'vue'; provide('theme', 'dark')",
        ),
        (
            "src/child.ts",
            "import { inject } from 'vue'; inject('missing')",
        ),
    ];
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let mut anchor = None;
    for (path, source) in files {
        anchor.get_or_insert(compilation.add_source(path, source).unwrap());
    }
    compilation
        .set_input::<CrossFileAnalysisInput>(CrossFileAnalysisRequest::new(options.clone()))
        .unwrap();
    let atlas = compilation
        .query::<CrossFileAnalysisProduct>(anchor.unwrap())
        .unwrap();

    let mut direct = CrossFileAnalyzer::new(options);
    for (path, source) in files {
        direct.add_file(path, source);
    }
    direct.rebuild_import_edges();
    direct.rebuild_component_edges();
    let direct = direct.analyze();
    let atlas_codes: Vec<_> = atlas
        .value()
        .result()
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code(), diagnostic.message.as_str()))
        .collect();
    let direct_codes: Vec<_> = direct
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code(), diagnostic.message.as_str()))
        .collect();

    assert_eq!(atlas_codes, direct_codes);
    assert_eq!(
        atlas.value().result().stats.files_analyzed,
        direct.stats.files_analyzed
    );
    assert_eq!(
        atlas.value().result().stats.dependency_edges,
        direct.stats.dependency_edges
    );
}

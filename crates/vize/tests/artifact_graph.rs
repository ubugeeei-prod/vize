use vize::artifact_graph::{VizeGraphConfig, analysis_roots, compiler_roots, create_compilation};
use vize::atelier_dom::DomOutputProduct;
use vize::atelier_jsx::JsxSyntaxProduct;
use vize::atelier_sfc::{SfcDescriptorProduct, SfcTemplateProduct};
use vize::atelier_ssr::SsrOutputProduct;
use vize::atelier_vapor::VaporPlanProduct;
use vize::canon::CanonSemanticVirtualTsProduct;
use vize::croquis_cf::CroquisProjectProduct;
use vize::flow::FlowProduct;
use vize::patina::PatinaSemanticReportProduct;
use vize::relief::{ReliefProduct, VueDialectInput};
use vize::rendu::{RenderCapabilities, RenderCapabilitiesInput, RenduProduct};
use vize_atlas::{ObservationKind, PlanError, ProductId, SourceProvenance};
use vize_carton::config::VueVersion;
use vize_croquis::CroquisSemanticProduct;

#[path = "artifact_graph/project.rs"]
mod project;

const SFC: &str = r#"<script setup lang="ts">
const ready = true
const items = [{ id: 1, label: 'one' }]
</script>
<template>
  <section v-if="ready">
    <p v-for="(item, index) in items" :key="item.id">{{ item.label }}</p>
  </section>
</template>"#;

const TSX: &str = r#"const ready = true;
const items = [{ id: 1, label: 'one' }];
export const App = () => ready
  ? <section>{items.map((item, index) => <p key={item.id}>{item.label}</p>)}</section>
  : <span>empty</span>;"#;

#[test]
fn registration_and_source_insertion_create_no_artifacts() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    compilation.add_source("App.vue", SFC).unwrap();
    compilation.add_source("App.tsx", TSX).unwrap();

    assert!(compilation.cache().is_empty());
    for product in [
        ProductId::of::<ReliefProduct>(),
        ProductId::of::<CroquisSemanticProduct>(),
        ProductId::of::<FlowProduct>(),
        ProductId::of::<RenduProduct>(),
        ProductId::of::<CroquisProjectProduct>(),
    ] {
        assert_eq!(compilation.counters().for_id(product).executions(), 0);
    }
}

#[test]
fn sfc_multi_backend_request_shares_frontend_and_rendu_once() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source = compilation.add_source("App.vue", SFC).unwrap();
    let plan = compilation
        .plan(source, compiler_roots(true, true, true))
        .unwrap();

    assert!(plan.contains::<SfcDescriptorProduct>());
    assert!(plan.contains::<ReliefProduct>());
    assert!(plan.contains::<RenduProduct>());
    assert!(!plan.contains::<JsxSyntaxProduct>());
    assert!(!plan.contains::<CroquisProjectProduct>());

    let output = compilation.execute(plan).unwrap();
    assert!(
        output
            .get::<DomOutputProduct>()
            .unwrap()
            .unwrap()
            .code
            .contains("export function render")
    );
    assert!(
        output
            .get::<SsrOutputProduct>()
            .unwrap()
            .unwrap()
            .code
            .contains("export function ssrRender")
    );
    assert!(
        !output
            .get::<VaporPlanProduct>()
            .unwrap()
            .unwrap()
            .blocks()
            .is_empty()
    );
    assert_eq!(executions::<ReliefProduct>(&compilation), 1);
    assert_eq!(executions::<RenduProduct>(&compilation), 1);
    assert_eq!(executions::<JsxSyntaxProduct>(&compilation), 0);
    assert_eq!(executions::<CroquisProjectProduct>(&compilation), 0);
    assert!(
        !compilation
            .cache()
            .contains::<CroquisProjectProduct>(source)
    );
}

#[test]
fn tsx_multi_backend_request_never_constructs_relief() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source = compilation.add_source("App.tsx", TSX).unwrap();
    let plan = compilation
        .plan(source, compiler_roots(true, true, true))
        .unwrap();

    assert!(plan.contains::<JsxSyntaxProduct>());
    assert!(plan.contains::<RenduProduct>());
    assert!(!plan.contains::<SfcDescriptorProduct>());
    assert!(!plan.contains::<ReliefProduct>());

    let output = compilation.execute(plan).unwrap();
    assert!(output.get::<DomOutputProduct>().unwrap().is_some());
    assert!(output.get::<SsrOutputProduct>().unwrap().is_some());
    assert!(output.get::<VaporPlanProduct>().unwrap().is_some());
    assert_eq!(executions::<JsxSyntaxProduct>(&compilation), 1);
    assert_eq!(executions::<RenduProduct>(&compilation), 1);
    assert_eq!(executions::<ReliefProduct>(&compilation), 0);
    assert!(!compilation.cache().contains::<ReliefProduct>(source));
}

#[test]
fn combined_lint_and_typecheck_share_semantics_without_render_work() {
    for (name, source_text, expects_relief) in [("App.vue", SFC, true), ("App.tsx", TSX, false)] {
        let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
        let source = compilation.add_source(name, source_text).unwrap();
        let plan = compilation
            .plan(source, analysis_roots(true, true))
            .unwrap();

        assert!(plan.contains::<CroquisSemanticProduct>());
        assert!(plan.contains::<FlowProduct>());
        assert_eq!(plan.contains::<ReliefProduct>(), expects_relief);
        assert!(!plan.contains::<RenduProduct>());
        assert!(!plan.contains::<DomOutputProduct>());

        let output = compilation.execute(plan).unwrap();
        assert!(
            output
                .get::<PatinaSemanticReportProduct>()
                .unwrap()
                .is_some()
        );
        let virtual_ts = output
            .get::<CanonSemanticVirtualTsProduct>()
            .unwrap()
            .unwrap();
        assert!(virtual_ts.expression_guard_count > 0);
        assert!(virtual_ts.reachable_block_count > 0);
        assert!(virtual_ts.flow_mapped_expression_count > 0);
        assert_eq!(executions::<CroquisSemanticProduct>(&compilation), 1);
        assert_eq!(executions::<FlowProduct>(&compilation), 1);
        assert_eq!(queries::<CroquisSemanticProduct>(&compilation), 2);
        assert_eq!(executions::<RenduProduct>(&compilation), 0);
    }
}

#[test]
fn sfc_and_tsx_produce_the_same_peer_flow_product() {
    for (name, source_text, is_jsx) in [("App.vue", SFC, false), ("App.tsx", TSX, true)] {
        let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
        let source = compilation.add_source(name, source_text).unwrap();
        let plan = compilation.plan_for::<FlowProduct>(source).unwrap();

        assert_eq!(plan.contains::<JsxSyntaxProduct>(), is_jsx);
        assert_eq!(plan.contains::<ReliefProduct>(), !is_jsx);
        assert!(!plan.contains::<RenduProduct>());
        let output = compilation.execute(plan).unwrap();
        let flow = output.get::<FlowProduct>().unwrap().unwrap();
        assert!(flow.blocks().len() > 1);
        assert!(flow.control_edges().len() > 0);
        flow.validate().unwrap();
        assert_eq!(executions::<FlowProduct>(&compilation), 1);
        assert_eq!(executions::<RenduProduct>(&compilation), 0);
    }
}

#[test]
fn frontend_diagnostics_are_attributed_to_the_executing_provider() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source = compilation
        .add_source("Broken.tsx", "export const Broken = () => <div>{")
        .unwrap();
    let output = compilation.query::<JsxSyntaxProduct>(source).unwrap();
    let observations = output.execution().observations();

    assert!(!observations.is_empty());
    assert!(
        observations
            .iter()
            .any(|observation| observation.kind() == ObservationKind::Diagnostic)
    );
    assert!(observations.iter().all(|observation| {
        observation.request().source() == source
            && observation.request().product() == ProductId::of::<JsxSyntaxProduct>()
            && observation.source() == source
            && observation.provider().name().contains("JsxSyntaxProvider")
    }));
}

#[test]
fn sfc_template_product_preserves_embedded_source_identity() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let parent = compilation.add_source("App.vue", SFC).unwrap();
    let template = compilation
        .query::<SfcTemplateProduct>(parent)
        .unwrap()
        .value()
        .clone();
    let embedded = compilation
        .add_embedded_source(
            template.parent,
            template.range,
            template.name,
            template.text,
        )
        .unwrap();

    assert_eq!(template.parent, parent);
    assert_eq!(
        compilation.source(embedded).unwrap().provenance(),
        &SourceProvenance::Embedded {
            parent,
            parent_revision: template.parent_revision,
            range: template.range,
        }
    );
}

#[test]
fn typed_dimensions_invalidate_only_relevant_cached_products() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source = compilation.add_source("App.vue", SFC).unwrap();
    compilation.query::<DomOutputProduct>(source).unwrap();

    let render_change = compilation
        .set_input::<RenderCapabilitiesInput>(RenderCapabilities {
            custom_renderer: true,
            ..VizeGraphConfig::default().render
        })
        .unwrap();
    assert!(evicted::<DomOutputProduct>(&render_change));
    assert!(!evicted::<RenduProduct>(&render_change));
    assert!(!evicted::<ReliefProduct>(&render_change));
    assert!(compilation.cache().contains::<RenduProduct>(source));
    assert!(compilation.cache().contains::<ReliefProduct>(source));

    compilation.query::<DomOutputProduct>(source).unwrap();
    let dialect_change = compilation
        .set_input::<VueDialectInput>(VueVersion::V2_7)
        .unwrap();
    assert!(evicted::<ReliefProduct>(&dialect_change));
    assert!(evicted::<RenduProduct>(&dialect_change));
    assert!(evicted::<DomOutputProduct>(&dialect_change));
    assert!(!evicted::<SfcDescriptorProduct>(&dialect_change));
    assert!(!evicted::<SfcTemplateProduct>(&dialect_change));
}

#[test]
fn vue_lines_and_vapor_mode_are_explicit_query_dimensions() {
    let mut compilation = create_compilation(VizeGraphConfig {
        render: RenderCapabilities::default(),
        ..Default::default()
    })
    .unwrap();
    let source = compilation.add_source("App.vue", SFC).unwrap();

    assert!(matches!(
        compilation.plan_for::<VaporPlanProduct>(source),
        Err(PlanError::NoApplicableProvider { .. })
    ));
    for vue in [
        VueVersion::V1,
        VueVersion::V2,
        VueVersion::V2_7,
        VueVersion::V3,
    ] {
        compilation.set_input::<VueDialectInput>(vue).unwrap();
        let plan = compilation.plan_for::<ReliefProduct>(source).unwrap();
        assert!(
            plan.input_revisions()
                .iter()
                .any(|(input, _)| *input == vize_atlas::InputId::of::<VueDialectInput>())
        );
    }
    compilation
        .set_input::<RenderCapabilitiesInput>(RenderCapabilities {
            vapor: true,
            ..Default::default()
        })
        .unwrap();
    assert!(compilation.plan_for::<VaporPlanProduct>(source).is_ok());
}

fn executions<P: vize_atlas::Product>(compilation: &vize_atlas::Compilation) -> u64 {
    compilation.counters().for_product::<P>().executions()
}

fn queries<P: vize_atlas::Product>(compilation: &vize_atlas::Compilation) -> u64 {
    compilation.counters().for_product::<P>().queries()
}

fn evicted<P: vize_atlas::Product>(report: &vize_atlas::InputInvalidationReport) -> bool {
    report
        .evicted()
        .iter()
        .any(|entry| entry.product == ProductId::of::<P>())
}

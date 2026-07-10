use vize::artifact_graph::{VizeGraphConfig, create_compilation};
use vize::atelier_dom::DomOutputProduct;
use vize::atelier_sfc::SfcTemplateProduct;
use vize::atelier_ssr::SsrOutputProduct;
use vize::canon::{CanonSemanticVirtualTsProduct, SemanticVirtualTsMappingKind};
use vize::flow::FlowProduct;
use vize::rendu::RenduProduct;
use vize_atlas::ProductId;
use vize_carton::{source_anchor::SourceAnchor, source_range::SourceRange};

const SFC: &str = r#"<script setup lang="ts">
const ready = true
const message = 'anchored'
</script>
<template>
  <section v-if="ready">{{ message }}</section>
</template>"#;

const TSX: &str = r#"const ready = true;
export const App = () => ready ? <section>yes</section> : <span>no</span>;"#;

#[test]
fn sfc_parent_anchor_survives_render_flow_and_virtual_ts_products() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source = compilation.add_source("src/App.vue", SFC).unwrap();
    let revision = compilation.source(source).unwrap().revision();
    let template = compilation
        .query::<SfcTemplateProduct>(source)
        .unwrap()
        .value()
        .clone();
    let template_range = SourceRange::new(
        u32::try_from(template.range.start).unwrap(),
        u32::try_from(template.range.end).unwrap(),
    );
    let template_anchor =
        SourceAnchor::new(source.get(), revision.get()).with_parent_range(template_range);
    let root_anchor = SourceAnchor::new(source.get(), revision.get());

    let plan = compilation
        .plan(
            source,
            [
                ProductId::of::<DomOutputProduct>(),
                ProductId::of::<SsrOutputProduct>(),
                ProductId::of::<FlowProduct>(),
                ProductId::of::<CanonSemanticVirtualTsProduct>(),
            ],
        )
        .unwrap();
    let output = compilation.execute(plan).unwrap();

    let rendu = output.get::<RenduProduct>().unwrap().unwrap();
    assert_eq!(rendu.sources()[0].anchor(), Some(template_anchor));

    let flow = output.get::<FlowProduct>().unwrap().unwrap();
    assert_eq!(
        flow.sources().next().and_then(|source| source.anchor()),
        Some(template_anchor)
    );

    let dom = output.get::<DomOutputProduct>().unwrap().unwrap();
    assert!(!dom.mappings.is_empty());
    assert!(
        dom.mappings
            .iter()
            .all(|mapping| mapping.anchor == Some(template_anchor))
    );
    let dom_mapping = &dom.mappings[0];
    let absolute = template_anchor.resolve_range(SourceRange::new(
        dom_mapping.source.start.offset,
        dom_mapping.source.end.offset,
    ));
    assert!(absolute.start >= template_range.start);
    assert!(absolute.end <= template_range.end);

    let ssr = output.get::<SsrOutputProduct>().unwrap().unwrap();
    assert!(!ssr.mappings.is_empty());
    assert!(
        ssr.mappings
            .iter()
            .all(|mapping| mapping.anchor == Some(template_anchor))
    );

    let virtual_ts = output
        .get::<CanonSemanticVirtualTsProduct>()
        .unwrap()
        .unwrap();
    let template_mapping = virtual_ts
        .mappings
        .iter()
        .find(|mapping| mapping.kind == SemanticVirtualTsMappingKind::TemplateExpression)
        .expect("template expression mapping");
    assert_eq!(template_mapping.source_anchor, Some(root_anchor));
    assert!(template_mapping.source.start >= template_range.start);
    assert!(template_mapping.source.end <= template_range.end);
}

#[test]
fn tsx_anchor_is_shared_by_rendu_and_flow_without_frontend_local_identity_loss() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source = compilation.add_source("src/App.tsx", TSX).unwrap();
    let revision = compilation.source(source).unwrap().revision();
    let expected = SourceAnchor::new(source.get(), revision.get());
    let plan = compilation
        .plan(
            source,
            [
                ProductId::of::<RenduProduct>(),
                ProductId::of::<FlowProduct>(),
            ],
        )
        .unwrap();
    let output = compilation.execute(plan).unwrap();

    let rendu = output.get::<RenduProduct>().unwrap().unwrap();
    assert_eq!(rendu.sources()[0].anchor(), Some(expected));
    assert_eq!(rendu.sources()[0].name.as_deref(), Some("src/App.tsx"));

    let flow = output.get::<FlowProduct>().unwrap().unwrap();
    let flow_source = flow.sources().next().expect("TSX Flow source");
    assert_eq!(flow_source.anchor(), Some(expected));
    assert_eq!(flow_source.name(), "src/App.tsx");
}

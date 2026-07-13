use super::executions;
use vize::artifact_graph::{VizeGraphConfig, create_compilation, project_roots};
use vize::croquis_cf::{CroquisProjectProduct, CrossFileAnalysisProduct};
use vize::rendu::RenduProduct;
use vize_atlas::ProductStatus;
use vize_croquis::CroquisSemanticProduct;

const PROJECT_SFC: &str = r#"<script setup lang="ts">
import Card from './Card.tsx'
const title = 'Atlas'
</script>
<template><Card :title="title" /></template>"#;

const PROJECT_TSX: &str = r#"export const Card = (props: { title: string }) =>
  <article>{props.title}</article>;"#;

#[test]
fn project_analysis_is_opt_in_and_reuses_cross_source_semantics() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let app = compilation.add_source("src/App.vue", PROJECT_SFC).unwrap();
    let card = compilation.add_source("src/Card.tsx", PROJECT_TSX).unwrap();
    compilation
        .add_source("README.md", "not a semantic source")
        .unwrap();

    let plan = compilation.plan(app, project_roots(true)).unwrap();
    assert!(plan.contains::<CroquisProjectProduct>());
    assert!(!plan.contains::<CrossFileAnalysisProduct>());
    assert!(plan.contains::<CroquisSemanticProduct>());
    assert!(!plan.contains::<RenduProduct>());
    let output = compilation.execute(plan).unwrap();
    let project = output.get::<CroquisProjectProduct>().unwrap().unwrap();

    assert_eq!(project.sources.len(), 2);
    let card_usage = project
        .component_usages
        .iter()
        .find(|usage| usage.name == "Card")
        .expect("SFC component usage");
    assert_eq!(card_usage.source, app);
    assert_eq!(card_usage.candidates, [card]);
    assert_eq!(executions::<CroquisSemanticProduct>(&compilation), 2);
    assert_eq!(executions::<CroquisProjectProduct>(&compilation), 1);
    assert_eq!(executions::<RenduProduct>(&compilation), 0);

    compilation
        .update_source(card, PROJECT_TSX.replace("article", "section"))
        .unwrap();
    assert!(compilation.cache().contains::<CroquisSemanticProduct>(app));
    assert!(!compilation.cache().contains::<CroquisSemanticProduct>(card));
    assert!(!compilation.cache().contains::<CroquisProjectProduct>(app));

    compilation.query::<CroquisProjectProduct>(app).unwrap();
    assert_eq!(executions::<CroquisSemanticProduct>(&compilation), 3);
    assert_eq!(executions::<CroquisProjectProduct>(&compilation), 2);
}

#[test]
fn immutable_snapshot_forks_a_reusable_project_view() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let app = compilation.add_source("src/App.vue", PROJECT_SFC).unwrap();
    let card = compilation.add_source("src/Card.tsx", PROJECT_TSX).unwrap();
    compilation.query::<CroquisProjectProduct>(app).unwrap();

    let captured_card_revision = compilation.source(card).unwrap().revision();
    let snapshot = compilation.snapshot();
    assert_eq!(
        snapshot
            .plan_for::<CroquisProjectProduct>(app)
            .unwrap()
            .source_revisions()
            .len(),
        2
    );

    compilation
        .update_source(card, PROJECT_TSX.replace("article", "aside"))
        .unwrap();
    assert_eq!(
        snapshot.source(card).unwrap().revision(),
        captured_card_revision
    );
    assert_ne!(
        compilation.source(card).unwrap().revision(),
        captured_card_revision
    );

    let mut fork = snapshot.fork();
    assert_eq!(
        fork.query::<CroquisProjectProduct>(app).unwrap().status(),
        ProductStatus::CacheHit
    );
    fork.update_source(app, PROJECT_SFC.replace("Atlas", "Fork"))
        .unwrap();
    assert_ne!(
        fork.source(app).unwrap().revision(),
        snapshot.source(app).unwrap().revision()
    );
    assert_eq!(
        compilation.source(app).unwrap().revision(),
        snapshot.source(app).unwrap().revision()
    );
}

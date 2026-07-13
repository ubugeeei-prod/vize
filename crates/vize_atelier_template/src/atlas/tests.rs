use vize_atlas::{Compilation, ProductStatus};
use vize_croquis::{CroquisDocumentProduct, CroquisSemanticProduct};
use vize_flow::FlowProduct;
use vize_relief::{ReliefProduct, TransformedReliefProduct};
use vize_rendu::RenduProduct;

use super::*;

fn compilation(target: TemplateRenderTarget) -> (Compilation, vize_atlas::SourceId) {
    let mut compilation = Compilation::new();
    register_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source("fixture.vue-template", "<main>{{ message }}</main>")
        .unwrap();
    let mut request = TemplateCompileRequest::default().for_target(target);
    request.source_map = true;
    install_template_compile_request(&mut compilation, source, request).unwrap();
    (compilation, source)
}

#[test]
fn compile_root_uses_relief_rendu_and_only_the_requested_backend() {
    for target in [
        TemplateRenderTarget::Dom,
        TemplateRenderTarget::Ssr,
        TemplateRenderTarget::Vapor,
    ] {
        let (compilation, source) = compilation(target);
        let outcome = compilation
            .snapshot()
            .query_session()
            .query::<TemplateCompileProduct>(source)
            .unwrap();
        assert!(outcome.plan().contains::<ReliefProduct>());
        assert!(outcome.plan().contains::<TransformedReliefProduct>());
        assert!(outcome.plan().contains::<RenduProduct>());
        assert!(!outcome.plan().contains::<FlowProduct>());
        assert_eq!(
            outcome
                .plan()
                .contains::<vize_atelier_dom::DomOutputProduct>(),
            target == TemplateRenderTarget::Dom
        );
        assert_eq!(
            outcome
                .plan()
                .contains::<vize_atelier_ssr::SsrOutputProduct>(),
            target == TemplateRenderTarget::Ssr
        );
        assert_eq!(
            outcome
                .plan()
                .contains::<vize_atelier_vapor::VaporOutputProduct>(),
            target == TemplateRenderTarget::Vapor
        );
        assert!(!outcome.value().mappings.is_empty());
        assert!(
            outcome
                .value()
                .mappings
                .iter()
                .all(|mapping| { mapping.generated_end >= mapping.generated_start })
        );
        assert!(
            !outcome.value().map.as_ref().unwrap()["mappings"]
                .as_str()
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn raw_template_source_is_not_applicable_without_typed_settings() {
    let mut compilation = Compilation::new();
    register_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source("fixture.vue-template", "<div />")
        .unwrap();
    assert!(
        compilation
            .snapshot()
            .query_session()
            .query::<ReliefProduct>(source)
            .is_err()
    );
}

#[test]
fn repeat_query_reuses_the_complete_target_closure() {
    let (compilation, source) = compilation(TemplateRenderTarget::Dom);
    let mut session = compilation.snapshot().query_session();
    session.query::<TemplateCompileProduct>(source).unwrap();
    let second = session.query::<TemplateCompileProduct>(source).unwrap();
    assert_eq!(second.status(), ProductStatus::CacheHit);
}

#[test]
fn one_compile_query_executes_shared_and_target_products_once() {
    let (compilation, source) = compilation(TemplateRenderTarget::Dom);
    let mut session = compilation.snapshot().query_session();
    session.query::<TemplateCompileProduct>(source).unwrap();
    assert_eq!(
        session
            .counters()
            .for_product::<ReliefProduct>()
            .executions(),
        1
    );
    assert_eq!(
        session
            .counters()
            .for_product::<TransformedReliefProduct>()
            .executions(),
        1
    );
    assert_eq!(
        session
            .counters()
            .for_product::<RenduProduct>()
            .executions(),
        1
    );
    assert_eq!(
        session
            .counters()
            .for_product::<vize_atelier_dom::DomOutputProduct>()
            .executions(),
        1
    );
}

#[test]
fn standalone_document_relief_and_croquis_share_one_frontend_revision() {
    let text = r#"<!doctype html>
<html><body>
  <div v-scope="{ count: 0 }">{{ count }}</div>
</body></html>"#;
    let mut compilation = Compilation::new();
    register_atlas_providers(&mut compilation).unwrap();
    let source = compilation.add_source("index.html", text).unwrap();
    install_template_compile_request(&mut compilation, source, TemplateCompileRequest::default())
        .unwrap();
    install_template_parse_mode(&mut compilation, source, TemplateParseMode::Document).unwrap();

    let mut session = compilation.snapshot().query_session();
    let syntax = session.query::<ReliefProduct>(source).unwrap();
    assert!(
        syntax
            .value()
            .as_ref()
            .unwrap()
            .parse_diagnostics()
            .is_empty()
    );

    let croquis = session.query::<CroquisDocumentProduct>(source).unwrap();
    let interpolation = text.find("{{ count }}").unwrap() as u32 + 3;
    assert!(
        croquis
            .value()
            .analysis()
            .scopes
            .bindings_visible_at(interpolation)
            .iter()
            .any(|(name, _, _)| *name == "count")
    );
    assert!(
        session
            .query::<CroquisSemanticProduct>(source)
            .unwrap()
            .plan()
            .contains::<CroquisDocumentProduct>()
    );
    assert_eq!(
        session
            .query::<CroquisDocumentProduct>(source)
            .unwrap()
            .status(),
        ProductStatus::CacheHit
    );
    assert_eq!(
        session
            .counters()
            .for_product::<ReliefProduct>()
            .executions(),
        1
    );
    assert_eq!(
        session
            .counters()
            .for_product::<CroquisDocumentProduct>()
            .executions(),
        1
    );
}

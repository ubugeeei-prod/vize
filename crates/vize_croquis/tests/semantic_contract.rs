use vize_atlas::ProductId;
use vize_croquis::{CroquisSemanticProduct, CroquisSemanticSnapshotBuilder, SemanticSourceRange};

#[test]
fn graph_product_and_owned_model_need_no_syntax_adapter() {
    let mut builder = CroquisSemanticSnapshotBuilder::new();
    builder.add_binding(
        "count",
        "setupRef",
        "setup",
        Some(SemanticSourceRange::new(6, 11)),
    );
    builder.add_template_expression(
        "count",
        "interpolation",
        SemanticSourceRange::new(20, 25),
        0,
    );
    let snapshot = builder.finish();

    assert_eq!(
        ProductId::of::<CroquisSemanticProduct>().name(),
        "croquis.semantics"
    );
    assert_eq!(snapshot.binding_by_name("count").unwrap().kind, "setupRef");
    assert_eq!(snapshot.template_expressions_in_scope(0).count(), 1);
    assert_eq!(snapshot.summary.script_binding_count, 1);
}

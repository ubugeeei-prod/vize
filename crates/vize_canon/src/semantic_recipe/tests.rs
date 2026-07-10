use super::{
    CanonSemanticVirtualTsProduct, SemanticVirtualTsMappingKind, generate_semantic_virtual_ts,
    generate_semantic_virtual_ts_with_flow, register_semantic_virtual_ts_recipe,
};
use oxc_allocator::Allocator as OxcAllocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_armature::parse;
use vize_atlas::{Compilation, ProductStatus, Provider, ProviderContext, ProviderError};
use vize_carton::{Bump, source_anchor::SourceAnchor, source_range::SourceRange};
use vize_croquis::{
    CroquisSemanticProduct, CroquisSemanticSnapshot, CroquisSemanticSnapshotBuilder, Drawer,
    DrawerOptions, SemanticSourceRange,
};
use vize_flow::{ControlEdgeKind, FlowGraph, FlowProduct, NodeKind, Provenance};

fn semantic_snapshot() -> CroquisSemanticSnapshot {
    let allocator = Bump::new();
    let (root, errors) = parse(
        &allocator,
        r#"<button @click="count++" v-if="ready">{{ count }}</button>"#,
    );
    assert!(errors.is_empty());
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup("const count = ref(0); const ready = true;");
    drawer.draw_template(&root);
    drawer.finish().semantic_snapshot()
}

#[test]
fn output_is_deterministic_and_maps_bindings_and_expressions() {
    let snapshot = semantic_snapshot();
    let first = generate_semantic_virtual_ts(&snapshot);
    let second = generate_semantic_virtual_ts(&snapshot);

    assert_eq!(first, second);
    assert!(first.code.contains("declare const count: any;"));
    assert!(first.code.contains("void (count)"));
    assert!(first.code.contains("count++;"));
    assert!(first.binding_declaration_count >= 2);
    assert_eq!(first.expression_guard_count, 3);
    let allocator = OxcAllocator::default();
    let parsed = Parser::new(&allocator, &first.code, SourceType::ts()).parse();
    assert!(parsed.errors.is_empty(), "{:#?}", parsed.errors);
    assert!(
        first
            .mappings
            .iter()
            .any(|mapping| { mapping.kind == SemanticVirtualTsMappingKind::BindingDeclaration })
    );
    assert!(
        first
            .mappings
            .iter()
            .any(|mapping| { mapping.kind == SemanticVirtualTsMappingKind::TemplateExpression })
    );
}

#[test]
fn flow_reorders_but_retains_unreachable_expressions_and_maps_dominance() {
    let anchor = SourceAnchor::new(42, 7);
    let mut builder = CroquisSemanticSnapshotBuilder::new();
    builder.add_template_expression(
        "unreachableName",
        "interpolation",
        SemanticSourceRange::new(20, 35),
        0,
    );
    builder.add_template_expression(
        "reachableName",
        "interpolation",
        SemanticSourceRange::new(0, 13),
        0,
    );
    let mut semantics = builder.finish();
    semantics.source_anchor = Some(anchor);

    let mut flow = FlowGraph::new();
    let source = flow.add_source_with_anchor("fixture.vue", anchor).unwrap();
    let reachable = flow.add_block(Provenance::Synthetic).unwrap();
    let unreachable = flow.add_block(Provenance::Synthetic).unwrap();
    flow.add_node(
        flow.entry_block(),
        NodeKind::Operation,
        Provenance::source(source, SourceRange::new(0, 100)),
    )
    .unwrap();
    flow.add_node(
        unreachable,
        NodeKind::Operation,
        Provenance::source(source, SourceRange::new(20, 35)),
    )
    .unwrap();
    flow.add_node(
        reachable,
        NodeKind::Operation,
        Provenance::source(source, SourceRange::new(0, 13)),
    )
    .unwrap();
    let disconnected = flow.clone();
    flow.add_control_edge(
        flow.entry_block(),
        reachable,
        ControlEdgeKind::Normal,
        Provenance::Synthetic,
    )
    .unwrap();

    let output = generate_semantic_virtual_ts_with_flow(&semantics, &flow);
    let disconnected_output = generate_semantic_virtual_ts_with_flow(&semantics, &disconnected);
    assert_ne!(output, disconnected_output);
    assert_eq!(output.flow_mapped_expression_count, 2);
    assert_eq!(output.unreachable_expression_count, 1);
    assert!(output.code.find("reachableName") < output.code.find("unreachableName"));
    assert!(output.code.contains("retained for diagnostics"));
    let reachable_mapping = output
        .mappings
        .iter()
        .find(|mapping| mapping.source.start == 0)
        .unwrap();
    assert_eq!(reachable_mapping.flow_block, Some(reachable));
    assert_eq!(
        reachable_mapping.immediate_dominator,
        Some(flow.entry_block())
    );
    let unreachable_mapping = output
        .mappings
        .iter()
        .find(|mapping| mapping.source.start == 20)
        .unwrap();
    assert_eq!(unreachable_mapping.flow_block, Some(unreachable));
    assert_eq!(unreachable_mapping.immediate_dominator, None);

    let mut mismatched = FlowGraph::new();
    let mismatch_source = mismatched
        .add_source_with_anchor("other.vue", SourceAnchor::new(99, 1))
        .unwrap();
    mismatched
        .add_node(
            mismatched.entry_block(),
            NodeKind::Operation,
            Provenance::source(mismatch_source, SourceRange::new(20, 35)),
        )
        .unwrap();
    let mismatched_output = generate_semantic_virtual_ts_with_flow(&semantics, &mismatched);
    assert_eq!(mismatched_output.flow_mapped_expression_count, 0);
    assert!(
        mismatched_output.code.find("unreachableName")
            < mismatched_output.code.find("reachableName")
    );
}

struct SnapshotProvider(CroquisSemanticSnapshot);

impl Provider for SnapshotProvider {
    type Product = CroquisSemanticProduct;

    fn provide(
        &self,
        _context: &mut ProviderContext<'_>,
    ) -> Result<CroquisSemanticSnapshot, ProviderError> {
        Ok(self.0.clone())
    }
}

struct FlowSnapshotProvider;

impl Provider for FlowSnapshotProvider {
    type Product = FlowProduct;

    fn provide(&self, _context: &mut ProviderContext<'_>) -> Result<FlowGraph, ProviderError> {
        Ok(FlowGraph::new())
    }
}

#[test]
fn atlas_recipe_generates_virtual_ts_without_reading_source_text() {
    let mut compilation = Compilation::new();
    compilation
        .register_provider(SnapshotProvider(semantic_snapshot()))
        .unwrap();
    compilation.register_provider(FlowSnapshotProvider).unwrap();
    register_semantic_virtual_ts_recipe(&mut compilation).unwrap();
    let source = compilation
        .add_source("broken.vue", "<<< not valid Vue or TypeScript >>>")
        .unwrap();

    let outcome = compilation
        .query::<CanonSemanticVirtualTsProduct>(source)
        .unwrap();

    assert_eq!(outcome.status(), ProductStatus::Executed);
    assert_eq!(outcome.value().expression_guard_count, 3);
    assert_eq!(outcome.value().reachable_block_count, 1);
    let products: Vec<_> = outcome
        .plan()
        .products()
        .iter()
        .map(|product| product.name())
        .collect();
    assert_eq!(
        products,
        [
            "croquis.semantics",
            "flow.graph",
            "canon.semantic-virtual-ts"
        ]
    );
}

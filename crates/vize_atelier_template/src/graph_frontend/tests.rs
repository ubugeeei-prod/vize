use std::mem::{needs_drop, size_of, size_of_val};

use vize_armature::parse;
use vize_atelier_core::transform;
use vize_carton::Bump;
use vize_flow::{ControlEdgeKind, DataEdgeKind, EffectKind};
use vize_relief::TransformOptions;
use vize_relief::{ReliefSnapshot, RootNode, SnapshotProp, TemplateChildNode};
use vize_rendu::{RenduCapability, RenduNode, RenduProperty};

use super::TemplateGraphAdapter;

const TEMPLATE: &str = r#"<div id="app" :class="theme">
  <UserCard v-if="ready" @select.stop="onSelect">
    <template #default="{ item }">{{ item.name }}</template>
  </UserCard>
  <slot name="footer">fallback</slot>
  <p v-for="(row, index) in rows" :key="row.id">{{ row.label }}</p>
</div>"#;

fn transformed_snapshot() -> ReliefSnapshot {
    let allocator = Bump::new();
    let (mut root, errors) = parse(&allocator, TEMPLATE);
    assert!(errors.is_empty(), "template parse errors: {errors:?}");
    let result = transform(&allocator, &mut root, TransformOptions::default(), None);
    assert!(
        result.errors.is_empty(),
        "template transform errors: {:?}",
        result.errors
    );
    ReliefSnapshot::from_root(&root)
}

#[test]
fn adapter_is_a_borrowed_view_until_a_product_is_requested() {
    let snapshot = transformed_snapshot();
    let adapter = TemplateGraphAdapter::new(&snapshot);

    assert_eq!(size_of_val(&adapter), size_of::<&ReliefSnapshot>());
    assert!(!needs_drop::<TemplateGraphAdapter<'_>>());
    assert!(std::ptr::eq(adapter.snapshot(), &snapshot));
    assert!(!adapter.snapshot().nodes().is_empty());
}

#[test]
fn snapshot_lowers_directly_to_valid_rendu_with_all_template_shapes() {
    let snapshot = transformed_snapshot();
    let rendu = TemplateGraphAdapter::new(&snapshot)
        .lower_rendu()
        .expect("valid Rendu product");
    rendu.validate().expect("Rendu remains valid");

    assert_eq!(rendu.sources()[0].contents.as_ref(), TEMPLATE);
    for capability in [
        RenduCapability::Elements,
        RenduCapability::Components,
        RenduCapability::Slots,
        RenduCapability::Text,
        RenduCapability::Expressions,
        RenduCapability::Properties,
        RenduCapability::Directives,
        RenduCapability::Conditionals,
        RenduCapability::Iteration,
        RenduCapability::SourceProvenance,
    ] {
        assert!(
            rendu.capabilities().contains(capability),
            "missing capability {capability:?}"
        );
    }

    assert!(
        rendu
            .nodes()
            .iter()
            .any(|node| matches!(node, RenduNode::Component { .. }))
    );
    assert!(
        rendu
            .nodes()
            .iter()
            .any(|node| matches!(node, RenduNode::SlotOutlet { .. }))
    );
    assert!(
        rendu
            .nodes()
            .iter()
            .any(|node| matches!(node, RenduNode::SlotContent { .. }))
    );
    assert!(
        rendu
            .nodes()
            .iter()
            .any(|node| matches!(node, RenduNode::If { .. }))
    );
    assert!(
        rendu
            .nodes()
            .iter()
            .any(|node| matches!(node, RenduNode::For { .. }))
    );
    assert!(rendu.nodes().iter().flat_map(node_properties).any(|property| {
        matches!(
            property,
            RenduProperty::Directive(directive)
                if directive.name.as_ref() == "on"
                    && directive.modifiers.iter().any(|modifier| modifier.as_ref() == "stop")
        )
    }));
    assert!(rendu.nodes().iter().all(|node| {
        node.provenance().primary.is_none()
            || node
                .provenance()
                .primary
                .is_some_and(|span| span.end.offset as usize <= TEMPLATE.len())
    }));
}

#[test]
fn snapshot_projects_branches_loops_expression_uses_and_effects_to_flow() {
    let snapshot = transformed_snapshot();
    let flow = TemplateGraphAdapter::new(&snapshot)
        .project_flow()
        .expect("valid Flow product");
    flow.validate().expect("Flow remains valid");

    let edge_kinds: Vec<_> = flow.control_edges().map(|edge| edge.kind()).collect();
    for kind in [
        ControlEdgeKind::TrueBranch,
        ControlEdgeKind::FalseBranch,
        ControlEdgeKind::LoopBack,
        ControlEdgeKind::Return,
    ] {
        assert!(edge_kinds.contains(&kind), "missing control edge {kind:?}");
    }
    assert!(
        flow.data_edges()
            .any(|edge| edge.kind() == DataEdgeKind::Use)
    );
    assert!(
        flow.data_edges()
            .any(|edge| edge.kind() == DataEdgeKind::Definition)
    );
    assert!(
        flow.effects()
            .any(|effect| effect.kind() == EffectKind::Read)
    );
    assert!(
        flow.effects()
            .any(|effect| effect.kind() == EffectKind::Write)
    );
    assert!(
        flow.effects()
            .any(|effect| effect.kind() == EffectKind::Call)
    );

    let reachability = flow.reachability();
    assert_eq!(reachability.len(), flow.blocks().len());
    assert!(
        flow.data_edges()
            .all(|edge| flow.value(edge.value()).is_some() && flow.node(edge.node()).is_some())
    );
}

fn node_properties(node: &RenduNode) -> &[RenduProperty] {
    match node {
        RenduNode::Element { properties, .. }
        | RenduNode::Component { properties, .. }
        | RenduNode::SlotOutlet { properties, .. } => properties,
        _ => &[],
    }
}

#[test]
fn relief_snapshot_itself_still_contains_source_directives() {
    let snapshot = transformed_snapshot();
    assert!(snapshot.nodes().iter().any(|node| {
        let vize_relief::ReliefSnapshotNode::Element(element) = node else {
            return false;
        };
        element
            .props
            .iter()
            .any(|prop| matches!(prop, SnapshotProp::Directive(_)))
    }));
}

#[test]
fn relief_hoist_references_remain_typed_rendu_hoists() {
    let allocator = Bump::new();
    let mut root = RootNode::new(&allocator, "hoist");
    root.children.push(TemplateChildNode::Hoisted(9));
    let snapshot = ReliefSnapshot::from_root(&root);

    let rendu = TemplateGraphAdapter::new(&snapshot)
        .lower_rendu()
        .expect("hoist product");
    assert!(matches!(
        rendu.node(rendu.entry()[0]),
        Some(RenduNode::HoistRef { index: 9, .. })
    ));
}

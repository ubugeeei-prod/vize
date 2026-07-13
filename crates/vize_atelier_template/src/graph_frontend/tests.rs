use std::mem::{needs_drop, size_of, size_of_val};

use vize_armature::parse;
use vize_atelier_core::transform;
use vize_carton::Bump;
use vize_flow::{ControlEdgeKind, DataEdgeKind, EffectKind};
use vize_relief::TransformOptions;
use vize_relief::{ReliefSnapshot, RootNode, SnapshotProp, TemplateChildNode};
use vize_rendu::{RenduCapability, RenduName, RenduNode, RenduProperty};

use super::TemplateGraphAdapter;

const TEMPLATE: &str = r#"<div id="app" :class="theme">
  <UserCard v-if="ready" @select.stop="onSelect">
    <template #default="{ item }">{{ item.name }}</template>
  </UserCard>
  <slot name="footer">fallback</slot>
  <p v-for="(row, index) in rows" :key="row.id">{{ row.label }}</p>
</div>"#;

fn transformed_snapshot() -> ReliefSnapshot {
    transformed_snapshot_of(TEMPLATE)
}

pub(super) fn transformed_snapshot_of(template: &str) -> ReliefSnapshot {
    let allocator = Bump::new();
    let (mut root, errors) = parse(&allocator, template);
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

    assert!(rendu.nodes().iter().any(|node| matches!(
        node,
        RenduNode::Component { name: RenduName::Static(name), .. }
            if name.as_ref() == "UserCard"
    )));
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

#[test]
fn component_v_slot_becomes_first_class_slot_with_lexical_bindings() {
    let template = r#"<Popover v-slot="{ item }">{{ item.label }} {{ outside }}</Popover>"#;
    let snapshot = transformed_snapshot_of(template);
    let rendu = TemplateGraphAdapter::new(&snapshot)
        .lower_rendu()
        .expect("component v-slot Rendu product");

    let component = rendu
        .nodes()
        .iter()
        .find_map(|node| match node {
            RenduNode::Component {
                properties,
                children,
                ..
            } => Some((properties, children)),
            _ => None,
        })
        .expect("component node");
    assert!(
        component.0.iter().all(|property| !matches!(
            property,
            RenduProperty::Directive(directive) if directive.name.as_ref() == "slot"
        )),
        "component v-slot must not survive as a runtime directive"
    );
    assert!(matches!(
        component.1.as_slice(),
        [slot] if matches!(
            rendu.node(*slot),
            Some(RenduNode::SlotContent {
                name: RenduName::Static(name),
                bindings,
                ..
            }) if name.as_ref() == "default"
                && bindings.iter().any(|binding| binding.pattern.as_ref() == "{ item }")
        )
    ));

    let dom = vize_atelier_dom::compile_rendu(&rendu);
    assert!(
        dom.code.contains("default: _withCtx(({ item }) => ["),
        "{}",
        dom.code
    );
    assert!(
        dom.code.contains("_toDisplayString(item.label)"),
        "{}",
        dom.code
    );
    assert!(!dom.code.contains("_ctx.item"), "{}", dom.code);
    assert!(
        !dom.code.contains("_resolveDirective(\"slot\")"),
        "{}",
        dom.code
    );

    let ssr = vize_atelier_ssr::compile_rendu(&rendu);
    assert!(
        ssr.code
            .contains("\"default\": _withCtx(({ item }, _push, _parent, _scopeId)"),
        "{}",
        ssr.code
    );
    assert!(
        ssr.code.contains("_ssrInterpolate(item.label)"),
        "{}",
        ssr.code
    );
    assert!(!ssr.code.contains("_ctx.item"), "{}", ssr.code);
    assert!(
        !ssr.code.contains("_resolveDirective(\"slot\")"),
        "{}",
        ssr.code
    );
}

#[test]
fn suspense_fallback_survives_dom_ssr_and_ssr_vnode_slot_paths() {
    let template = r#"<Outer><Suspense><AsyncView /><template #fallback>loading</template></Suspense></Outer>"#;
    let snapshot = transformed_snapshot_of(template);
    let rendu = TemplateGraphAdapter::new(&snapshot)
        .lower_rendu()
        .expect("Suspense Rendu product");

    let dom = vize_atelier_dom::compile_rendu(&rendu);
    assert!(
        dom.code.contains("fallback: _withCtx(() => [\"loading\"])")
            || dom
                .code
                .contains("\"fallback\": _withCtx(() => [\"loading\"])"),
        "DOM fallback slot was dropped:\n{}",
        dom.code
    );

    let ssr = vize_atelier_ssr::compile_rendu(&rendu);
    assert!(
        ssr.code.contains("_ssrRenderSuspense(_push, {")
            && ssr.code.contains("\"fallback\": () => {")
            && ssr.code.contains("_push(\"loading\")"),
        "SSR push fallback slot was dropped:\n{}",
        ssr.code
    );
    assert!(
        ssr.code
            .contains("\"fallback\": _withCtx(() => [_createTextVNode(\"loading\")])"),
        "SSR VNode fallback path must retain the Suspense fallback slot too:\n{}",
        ssr.code
    );
}

use vize_rendu::{
    RenduAttribute, RenduBinding, RenduBuilder, RenduCapability, RenduComponentKind,
    RenduDirective, RenduEscapeMode, RenduExpression, RenduExpressionKind, RenduIfBranch,
    RenduName, RenduNode, RenduProperty, RenduProvenance, RenduSource, RenduSpan,
};

use super::compile_rendu;

#[test]
fn graph_ssr_covers_dynamic_names_directives_hoists_and_keys() {
    let mut builder = RenduBuilder::new();
    let source = builder.add_source(
        RenduSource::named("Parity.vue", " ".repeat(512))
            .with_anchor(vize_carton::source_anchor::SourceAnchor::new(13, 2)),
    );
    let provenance = RenduProvenance::from_span(RenduSpan::offsets(source, 5, 18));
    let mut expression = |code| {
        builder.add_expression(
            RenduExpression::new(code, RenduExpressionKind::Reference)
                .with_provenance(provenance.clone()),
        )
    };
    let component_name = expression("$setup.component");
    let property_name = expression("$data.property");
    let event_name = expression("$options.event");
    let slot_name = expression("$props.slot");
    let value = expression("$data.value");
    let handler = expression("$setup.handler");
    let attrs = expression("$props.attrs");
    let condition = expression("$data.ok");
    let items = expression("$props.items");
    let key = expression("item.id");
    let text = builder.add_node(RenduNode::Text {
        value: "<static>".into(),
        provenance: provenance.clone(),
    });
    let raw = builder.add_node(RenduNode::Expression {
        expression: value,
        escape: RenduEscapeMode::Raw,
        provenance: provenance.clone(),
    });
    let comment = builder.add_node(RenduNode::Comment {
        value: "comment".into(),
        provenance: provenance.clone(),
    });
    let element = builder.add_node(RenduNode::Element {
        tag: "input".into(),
        namespace: vize_rendu::RenduNamespace::Html,
        properties: vec![
            RenduProperty::Attribute(RenduAttribute {
                name: RenduName::Dynamic(property_name),
                value: Some(vize_rendu::RenduAttributeValue::Expression(value)),
                provenance: provenance.clone(),
            }),
            RenduProperty::spread(attrs),
            RenduProperty::Directive(
                RenduDirective::new("bind")
                    .with_argument(RenduName::Dynamic(property_name))
                    .with_expression(value),
            ),
            RenduProperty::Directive(
                RenduDirective::new("on")
                    .with_argument(RenduName::Dynamic(event_name))
                    .with_expression(handler)
                    .with_modifier("stop"),
            ),
            RenduProperty::Directive(RenduDirective::new("model").with_expression(value)),
            RenduProperty::Directive(RenduDirective::new("show").with_expression(condition)),
            RenduProperty::Directive(
                RenduDirective::new("focus")
                    .with_argument(RenduName::Dynamic(property_name))
                    .with_expression(value)
                    .with_modifier("lazy"),
            ),
        ],
        children: vec![],
        provenance: provenance.clone(),
    });
    let named_slot = builder.add_node(RenduNode::SlotContent {
        name: RenduName::Dynamic(slot_name),
        bindings: vec![RenduBinding::new("slotProps")],
        children: vec![text, raw],
        provenance: provenance.clone(),
    });
    let component = builder.add_node(RenduNode::Component {
        kind: RenduComponentKind::Ordinary,
        name: RenduName::Dynamic(component_name),
        properties: vec![
            RenduProperty::Directive(RenduDirective::new("model").with_expression(value)),
            RenduProperty::Directive(
                RenduDirective::new("on")
                    .with_argument(RenduName::Dynamic(event_name))
                    .with_expression(handler)
                    .with_modifier("stop"),
            ),
        ],
        children: vec![named_slot, comment],
        provenance: provenance.clone(),
    });
    let outlet = builder.add_node(RenduNode::SlotOutlet {
        name: RenduName::Dynamic(slot_name),
        properties: vec![],
        fallback: vec![comment],
        provenance: provenance.clone(),
    });
    let conditional = builder.add_node(RenduNode::If {
        branches: vec![
            RenduIfBranch::new(Some(condition), vec![component]),
            RenduIfBranch::new(None, vec![outlet]),
        ],
        provenance: provenance.clone(),
    });
    let iteration = builder.add_node(RenduNode::For {
        source: items,
        value: RenduBinding::new("item"),
        key: None,
        index: Some(RenduBinding::new("index")),
        key_expression: Some(key),
        body: vec![element],
        provenance: provenance.clone(),
    });
    let hoist = builder.add_node(RenduNode::HoistRef {
        index: 3,
        provenance: provenance.clone(),
    });
    let fragment = builder.add_node(RenduNode::Fragment {
        children: vec![conditional, iteration, hoist],
        provenance,
    });
    builder.push_entry(fragment);
    let root = builder.finish().unwrap();

    assert!(
        RenduCapability::ALL
            .into_iter()
            .all(|capability| root.capabilities().contains(capability))
    );
    let output = compile_rendu(&root);
    for expected in [
        "ssrRender(_ctx, _push, _parent, _attrs, $props, $setup, $data, $options)",
        "_ssrRenderDynamicAttr($data.property, $data.value, \"input\")",
        "_ssrRenderAttrs($props.attrs)",
        "_ssrRenderAttr(\"value\", $data.value)",
        "display: none",
        "_resolveDirective(\"focus\")",
        "_ssrRenderComponent($setup.component",
        "_createSlots(",
        "{ name: $props.slot, fn: _withCtx((slotProps, _push, _parent, _scopeId)",
        "[\"on\" + ($options.event)]: _withModifiers($setup.handler, [\"stop\"])",
        "\"modelValue\": $data.value",
        "_ssrRenderSlot(_ctx.$slots, $props.slot",
        "_ssrRenderList($props.items, (item, index) =>",
        "void (item.id)",
        "_ctx._ssrHoisted?.[3] ?? \"\"",
    ] {
        assert!(
            output.code.contains(expected),
            "missing {expected}:\n{}",
            output.code
        );
    }
    assert!(!output.code.contains("unsupported Rendu"));
    assert!(output.mappings.iter().any(|mapping| {
        mapping.source == RenduSpan::offsets(source, 5, 18)
            && mapping.anchor == Some(vize_carton::source_anchor::SourceAnchor::new(13, 2))
    }));
}

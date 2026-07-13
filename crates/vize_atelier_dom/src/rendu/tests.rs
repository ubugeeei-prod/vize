use super::compile_rendu;
use vize_rendu::{
    RenduAttribute, RenduBinding, RenduBuilder, RenduCapability, RenduComponentKind,
    RenduDirective, RenduEscapeMode, RenduExpression, RenduExpressionKind, RenduIfBranch,
    RenduName, RenduNode, RenduProperty, RenduProvenance, RenduSource, RenduSpan,
};

#[test]
fn emits_owned_rendu_without_a_frontend_ast() {
    let mut builder = RenduBuilder::new();
    builder.add_source(RenduSource::named("App.vue", "<div>{{ msg }}</div>"));
    let expression = builder.add_expression(RenduExpression::new(
        "_ctx.msg",
        RenduExpressionKind::Reference,
    ));
    let child = builder.add_node(RenduNode::Expression {
        expression,
        escape: RenduEscapeMode::Escaped,
        provenance: RenduProvenance::generated(),
    });
    let root = builder.add_node(RenduNode::Element {
        tag: "div".into(),
        namespace: vize_rendu::RenduNamespace::Html,
        properties: Vec::new(),
        children: vec![child],
        provenance: RenduProvenance::generated(),
    });
    builder.push_entry(root);
    let output = compile_rendu(&builder.finish().unwrap());
    assert!(output.code.contains("_h(\"div\""), "{}", output.code);
    assert!(output.code.contains("_toDisplayString(_ctx.msg)"));
}

#[test]
fn emits_every_rendu_capability_and_preserves_final_provenance() {
    let mut builder = RenduBuilder::new();
    let source = builder.add_source(
        RenduSource::named("Adversarial.vue", " ".repeat(512))
            .with_anchor(vize_carton::source_anchor::SourceAnchor::new(9, 4)),
    );
    let provenance = RenduProvenance::from_span(RenduSpan::offsets(source, 4, 12));
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
    let dynamic_text = builder.add_node(RenduNode::Expression {
        expression: value,
        escape: RenduEscapeMode::Raw,
        provenance: provenance.clone(),
    });
    let comment = builder.add_node(RenduNode::Comment {
        value: "safe--comment".into(),
        provenance: provenance.clone(),
    });
    let text = builder.add_node(RenduNode::Text {
        value: "text".into(),
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
                    .with_argument(RenduName::static_name("title"))
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
    let slot_content = builder.add_node(RenduNode::SlotContent {
        name: RenduName::Dynamic(slot_name),
        bindings: vec![RenduBinding::new("slotProps")],
        children: vec![dynamic_text, text],
        provenance: provenance.clone(),
    });
    let component = builder.add_node(RenduNode::Component {
        kind: RenduComponentKind::Ordinary,
        name: RenduName::Dynamic(component_name),
        properties: vec![RenduProperty::Directive(
            RenduDirective::new("model").with_expression(value),
        )],
        children: vec![slot_content, comment],
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
        index: 2,
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
        "export function render(_ctx, _cache, $props, $setup, $data, $options)",
        "_h($setup.component",
        "_createSlots(",
        "{ name: $props.slot, fn: _withCtx((slotProps) =>",
        "_renderSlot(_ctx.$slots, $props.slot",
        "[$data.property]: $data.value",
        "...$props.attrs",
        "title: $data.value",
        "[\"on\" + ($options.event)]: _withModifiers($setup.handler, [\"stop\"])",
        "\"modelValue\": $data.value",
        "\"onUpdate:modelValue\"",
        "_vModelText",
        "_vShow",
        "_resolveDirective(\"focus\")",
        "_renderList($props.items, (item, index) => _h(_Fragment, { key: item.id }",
        "_ctx._hoisted?.[2] ?? null",
    ] {
        assert!(
            output.code.contains(expected),
            "missing {expected}:\n{}",
            output.code
        );
    }
    assert!(!output.code.contains("unsupported Rendu"));
    assert!(output.mappings.iter().any(|mapping| {
        mapping.source == RenduSpan::offsets(source, 4, 12)
            && mapping.anchor == Some(vize_carton::source_anchor::SourceAnchor::new(9, 4))
    }));
}

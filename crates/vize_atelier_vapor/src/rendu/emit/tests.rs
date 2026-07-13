use vize_rendu::{
    RenduAttribute, RenduBinding, RenduBuilder, RenduCapability, RenduDirective, RenduExpression,
    RenduExpressionKind, RenduIfBranch, RenduName, RenduNamespace, RenduNode, RenduProperty,
    RenduProvenance, RenduSource, RenduSpan,
};

use super::emit_rendu;

fn generated() -> RenduProvenance {
    RenduProvenance::generated()
}

#[test]
fn emits_directives_dynamic_names_bindings_and_full_render_scope() {
    let mut builder = RenduBuilder::new();
    let source = builder.add_source(RenduSource::named("Parity.vue", " ".repeat(256)));
    let provenance = RenduProvenance::from_span(RenduSpan::offsets(source, 2, 14));
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
    let raw = builder.add_node(RenduNode::Expression {
        expression: value,
        escape: vize_rendu::RenduEscapeMode::Raw,
        provenance: provenance.clone(),
    });
    let text = builder.add_node(RenduNode::Text {
        value: "text".into(),
        provenance: provenance.clone(),
    });
    let comment = builder.add_node(RenduNode::Comment {
        value: "comment".into(),
        provenance: provenance.clone(),
    });
    let input = builder.add_node(RenduNode::Element {
        tag: "input".into(),
        namespace: RenduNamespace::Html,
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
            RenduProperty::Directive(
                RenduDirective::new("model")
                    .with_expression(value)
                    .with_modifier("trim"),
            ),
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
    let slot = builder.add_node(RenduNode::SlotContent {
        name: RenduName::Dynamic(slot_name),
        bindings: vec![RenduBinding::new("slotProps")],
        children: vec![raw, text],
        provenance: provenance.clone(),
    });
    let component = builder.add_node(RenduNode::Component {
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
        children: vec![slot, comment],
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
        body: vec![input],
        provenance: provenance.clone(),
    });
    let hoist = builder.add_node(RenduNode::HoistRef {
        index: 4,
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
    let output = emit_rendu(&root);
    for expected in [
        "render(_ctx = {}, _cache, $props, $setup, $data, $options)",
        "_createComponentWithFallback($setup.component",
        "[$props.slot]: (slotProps) =>",
        "_renderSlot($slots, $props.slot",
        "_setProp(n",
        "$data.property, $data.value",
        "_setDynamicProps",
        "\"on\" + ($options.event)",
        "_withModifiers($setup.handler, [\"stop\"])",
        "_applyTextModel",
        "_applyVShow",
        "_resolveDirective(\"focus\")",
        "\"modelValue\": () => ($data.value)",
        "\"innerHTML\", $data.value)",
        "(item, index) => (item.id)",
        "_hoisted[4]",
    ] {
        assert!(
            output.code.contains(expected),
            "missing {expected}:\n{}",
            output.code
        );
    }
    assert!(!output.code.contains("unsupported"));
}

#[test]
fn emits_static_rendu_as_a_real_vapor_template() {
    let mut builder = RenduBuilder::new();
    let text = builder.add_node(RenduNode::Text {
        value: "hello".into(),
        provenance: generated(),
    });
    let element = builder.add_node(RenduNode::Element {
        tag: "div".into(),
        namespace: RenduNamespace::Html,
        properties: vec![RenduProperty::Attribute(RenduAttribute::static_value(
            "class", "hero",
        ))],
        children: vec![text],
        provenance: generated(),
    });
    builder.push_entry(element);

    let output = emit_rendu(&builder.finish().unwrap());

    assert_eq!(output.templates, ["<div class=\"hero\">hello</div>"]);
    assert!(output.code.contains("const t0 = _template("));
    assert!(output.code.contains("const n0 = t0()"));
    assert!(output.code.contains("export function render"));
}

#[test]
fn emits_graph_operations_without_legacy_materialization() {
    let mut builder = RenduBuilder::new();
    let show = builder.add_expression(RenduExpression::new(
        "state.show",
        RenduExpressionKind::Reference,
    ));
    let items = builder.add_expression(RenduExpression::new(
        "state.items",
        RenduExpressionKind::Reference,
    ));
    let label = builder.add_expression(RenduExpression::new(
        "item.label",
        RenduExpressionKind::Reference,
    ));
    let spread = builder.add_expression(RenduExpression::new(
        "attrs",
        RenduExpressionKind::Reference,
    ));
    let text = builder.add_node(RenduNode::Expression {
        expression: label,
        escape: vize_rendu::RenduEscapeMode::Escaped,
        provenance: generated(),
    });
    let button = builder.add_node(RenduNode::Element {
        tag: "button".into(),
        namespace: RenduNamespace::Html,
        properties: vec![
            RenduProperty::Spread {
                expression: spread,
                provenance: generated(),
            },
            RenduProperty::Directive(RenduDirective::new("focus").with_expression(show)),
        ],
        children: vec![text],
        provenance: generated(),
    });
    let loop_node = builder.add_node(RenduNode::For {
        source: items,
        value: RenduBinding::new("item"),
        key: None,
        index: Some(RenduBinding::new("index")),
        key_expression: None,
        body: vec![button],
        provenance: generated(),
    });
    let fallback = builder.add_node(RenduNode::Text {
        value: "empty".into(),
        provenance: generated(),
    });
    let conditional = builder.add_node(RenduNode::If {
        branches: vec![
            RenduIfBranch::new(Some(show), vec![loop_node]),
            RenduIfBranch::new(None, vec![fallback]),
        ],
        provenance: generated(),
    });
    let slot = builder.add_node(RenduNode::SlotContent {
        name: RenduName::static_name("default"),
        bindings: vec![RenduBinding::new("slotProps")],
        children: vec![conditional],
        provenance: generated(),
    });
    let component = builder.add_node(RenduNode::Component {
        name: RenduName::static_name("Panel"),
        properties: vec![RenduProperty::Attribute(RenduAttribute::expression(
            "open", show,
        ))],
        children: vec![slot],
        provenance: generated(),
    });
    builder.push_entry(component);

    let output = emit_rendu(&builder.finish().unwrap());

    for expected in [
        "_resolveComponent(\"Panel\")",
        "\"default\": (slotProps) =>",
        "_createIf(() => (state.show)",
        "_createFor(() => (state.items), (item, index) =>",
        "_setDynamicProps",
        "_withDirectives",
        "_renderEffect",
    ] {
        assert!(
            output.code.contains(expected),
            "missing {expected}:\n{}",
            output.code
        );
    }
    assert!(
        output
            .templates
            .iter()
            .any(|template| template == "<button></button>")
    );
}

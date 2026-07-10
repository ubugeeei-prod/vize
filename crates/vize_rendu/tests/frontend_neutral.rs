use vize_rendu::{
    RenduAttribute, RenduBinding, RenduBuilder, RenduCapabilities, RenduCapability, RenduDirective,
    RenduEscapeMode, RenduExpression, RenduExpressionKind, RenduIfBranch, RenduName,
    RenduNamespace, RenduNode, RenduNodeId, RenduPosition, RenduProperty, RenduProvenance,
    RenduSource, RenduSpan, RenduValidationError, RenduWalkEvent, walk_rendu,
};

#[derive(Debug)]
struct SyntheticTemplate<'a> {
    source: &'a str,
    visible: &'a str,
    collection: &'a str,
    body_expression: &'a str,
}

fn lower_template(shape: &SyntheticTemplate<'_>) -> vize_rendu::RenduRoot {
    let mut builder = RenduBuilder::new();
    let source = builder
        .add_source(RenduSource::named("Panel.vue", shape.source).with_language("vue-template"));
    let provenance = |start, end| {
        RenduProvenance::from_span(RenduSpan::new(
            source,
            RenduPosition::offset(start),
            RenduPosition::offset(end),
        ))
    };
    let visible = builder.add_expression(
        RenduExpression::new(shape.visible, RenduExpressionKind::Reference)
            .with_provenance(provenance(11, 18)),
    );
    let collection = builder.add_expression(
        RenduExpression::new(shape.collection, RenduExpressionKind::Reference)
            .with_provenance(provenance(35, 40)),
    );
    let label = builder.add_expression(
        RenduExpression::new(shape.body_expression, RenduExpressionKind::Compound)
            .with_provenance(provenance(44, 54)),
    );
    let label_node = builder.add_node(RenduNode::Expression {
        expression: label,
        escape: RenduEscapeMode::Escaped,
        provenance: provenance(41, 57),
    });
    let loop_node = builder.add_node(RenduNode::For {
        source: collection,
        value: RenduBinding::new("item").with_provenance(provenance(27, 31)),
        key: None,
        index: None,
        key_expression: None,
        body: vec![label_node],
        provenance: provenance(20, 65),
    });
    let element = builder.add_node(RenduNode::Element {
        tag: "section".into(),
        namespace: RenduNamespace::Html,
        properties: vec![
            RenduProperty::Attribute(RenduAttribute::static_value("class", "panel")),
            RenduProperty::Directive(
                RenduDirective::new("show")
                    .with_expression(visible)
                    .with_provenance(provenance(8, 18)),
            ),
        ],
        children: vec![loop_node],
        provenance: provenance(0, shape.source.len() as u32),
    });
    let conditional = builder.add_node(RenduNode::If {
        branches: vec![RenduIfBranch::new(Some(visible), vec![element])],
        provenance: provenance(0, shape.source.len() as u32),
    });
    builder.push_entry(conditional);
    builder.finish().expect("template lowers to valid Rendu")
}

#[derive(Debug)]
struct SyntheticJsx<'a> {
    source: &'a str,
    component: &'a str,
    prop_expression: &'a str,
    slot_text: &'a str,
}

fn lower_jsx(shape: &SyntheticJsx<'_>) -> vize_rendu::RenduRoot {
    let mut builder = RenduBuilder::new();
    let source =
        builder.add_source(RenduSource::named("Card.tsx", shape.source).with_language("tsx"));
    let full = RenduProvenance::from_span(RenduSpan::offsets(source, 0, shape.source.len() as u32));
    let active = builder.add_expression(
        RenduExpression::new(shape.prop_expression, RenduExpressionKind::Compound)
            .with_provenance(full.clone()),
    );
    let text = builder.add_node(RenduNode::Text {
        value: shape.slot_text.into(),
        provenance: full.clone(),
    });
    let slot = builder.add_node(RenduNode::SlotContent {
        name: RenduName::static_name("default"),
        bindings: vec![RenduBinding::new("{ close }")],
        children: vec![text],
        provenance: full.clone(),
    });
    let component = builder.add_node(RenduNode::Component {
        name: RenduName::static_name(shape.component),
        properties: vec![RenduProperty::Attribute(RenduAttribute::expression(
            "active", active,
        ))],
        children: vec![slot],
        provenance: full,
    });
    builder.push_entry(component);
    builder.finish().expect("JSX lowers to valid Rendu")
}

#[test]
fn template_shape_lowers_without_a_relief_model() {
    let root = lower_template(&SyntheticTemplate {
        source: "<section v-show=\"visible\"><i v-for=\"item in items\">{{item.label}}</i></section>",
        visible: "visible",
        collection: "items",
        body_expression: "item.label",
    });

    let required = root.capabilities();
    for capability in [
        RenduCapability::Elements,
        RenduCapability::Expressions,
        RenduCapability::Properties,
        RenduCapability::Directives,
        RenduCapability::Conditionals,
        RenduCapability::Iteration,
        RenduCapability::SourceProvenance,
    ] {
        assert!(required.contains(capability), "missing {capability:?}");
    }
    assert!(!required.contains(RenduCapability::Components));

    let mut entered = Vec::new();
    walk_rendu(&root, |event| {
        if let RenduWalkEvent::Enter { node, .. } = event {
            entered.push(std::mem::discriminant(node));
        }
    });
    assert_eq!(entered.len(), 4);
}

#[test]
fn jsx_shape_uses_the_same_owned_hir_without_template_nodes() {
    let root = lower_jsx(&SyntheticJsx {
        source: "<Card active={state.ok}>{({ close }) => <>Save</>}</Card>",
        component: "Card",
        prop_expression: "state.ok",
        slot_text: "Save",
    });

    assert_eq!(root.sources()[0].language.as_deref(), Some("tsx"));
    assert!(root.capabilities().contains(RenduCapability::Components));
    assert!(root.capabilities().contains(RenduCapability::Slots));
    assert!(root.capabilities().contains(RenduCapability::Text));
    assert!(root.capabilities().contains(RenduCapability::Expressions));
    assert!(matches!(
        root.node(root.entry()[0]),
        Some(RenduNode::Component { .. })
    ));

    let text = root.nodes().iter().find_map(|node| match node {
        RenduNode::Text { value, .. } => Some(value.as_ref()),
        _ => None,
    });
    assert_eq!(text, Some("Save"));
}

#[test]
fn validation_rejects_cycles_dangling_expressions_and_bad_spans() {
    let mut cyclic = RenduBuilder::new();
    cyclic.add_node(RenduNode::Fragment {
        children: vec![RenduNodeId::new(0)],
        provenance: RenduProvenance::generated(),
    });
    cyclic.push_entry(RenduNodeId::new(0));
    let errors = cyclic.finish().expect_err("self-cycle must fail");
    assert!(errors.iter().any(
        |error| matches!(error, RenduValidationError::CyclicNode(id) if *id == RenduNodeId::new(0))
    ));

    let mut invalid = RenduBuilder::new();
    let source = invalid.add_source(RenduSource::anonymous("x"));
    let node = invalid.add_node(RenduNode::Expression {
        expression: vize_rendu::RenduExpressionId::new(7),
        escape: RenduEscapeMode::Escaped,
        provenance: RenduProvenance::from_span(RenduSpan::offsets(source, 0, 9)),
    });
    invalid.push_entry(node);
    let errors = invalid.finish().expect_err("invalid references must fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        RenduValidationError::MissingExpression { expression, .. }
            if *expression == vize_rendu::RenduExpressionId::new(7)
    )));
    assert!(
        errors
            .iter()
            .any(|error| matches!(error, RenduValidationError::InvalidSpan(_)))
    );
}

#[test]
fn backend_capability_negotiation_is_frontend_agnostic() {
    let root = lower_jsx(&SyntheticJsx {
        source: "<Card>{'Save'}</Card>",
        component: "Card",
        prop_expression: "true",
        slot_text: "Save",
    });
    let backend = RenduCapabilities::empty()
        .with(RenduCapability::Components)
        .with(RenduCapability::Text)
        .with(RenduCapability::SourceProvenance);
    let unsupported = root.capabilities().unsupported_by(backend);
    assert!(unsupported.contains(RenduCapability::Slots));
    assert!(unsupported.contains(RenduCapability::Properties));
    assert!(unsupported.contains(RenduCapability::Expressions));
}

use vize_rendu::{
    RenduAttribute, RenduBinding, RenduBuilder, RenduComponentKind, RenduExpression,
    RenduExpressionKind, RenduIfBranch, RenduName, RenduNode, RenduProperty, RenduProvenance,
    RenduSource, RenduSpan,
};

use super::{VaporName, VaporOperation, VaporProperty, plan_rendu};

#[test]
fn collapses_static_subtrees_without_losing_provenance() {
    let plan = {
        let mut builder = RenduBuilder::new();
        let source = builder.add_source(RenduSource::named(
            "App.vue",
            "<div class=\"hero\">Hello & welcome</div>",
        ));
        let child = builder.add_node(RenduNode::Text {
            value: "Hello & welcome".into(),
            provenance: RenduProvenance::from_span(RenduSpan::offsets(source, 18, 33)),
        });
        let element = builder.add_node(RenduNode::Element {
            tag: "div".into(),
            namespace: vize_rendu::RenduNamespace::Html,
            properties: vec![RenduProperty::Attribute(
                RenduAttribute::static_value("class", "hero").with_provenance(
                    RenduProvenance::from_span(RenduSpan::offsets(source, 5, 17)),
                ),
            )],
            children: vec![child],
            provenance: RenduProvenance::from_span(RenduSpan::offsets(source, 0, 39)),
        });
        builder.push_entry(element);
        plan_rendu(&builder.finish().unwrap())
    };

    let entry = plan.block(plan.entry()).unwrap();
    let [VaporOperation::StaticHtml { html, provenance }] = entry.operations.as_slice() else {
        panic!("expected one static template operation");
    };
    assert_eq!(
        html.as_ref(),
        "<div class=\"hero\">Hello &amp; welcome</div>"
    );
    assert_eq!(provenance.primary.unwrap().start.offset, 0);
    assert_eq!(provenance.related.len(), 2);
    assert_eq!(plan.sources()[0].name.as_deref(), Some("App.vue"));
}

#[test]
fn plans_components_conditionals_and_lists_as_owned_blocks() {
    let plan = {
        let mut builder = RenduBuilder::new();
        builder.add_source(RenduSource::named("View.tsx", " ".repeat(256)));
        let title = builder.add_expression(RenduExpression::new(
            "props.title",
            RenduExpressionKind::Reference,
        ));
        let condition = builder.add_expression(RenduExpression::new(
            "state.ready",
            RenduExpressionKind::Reference,
        ));
        let items = builder.add_expression(RenduExpression::new(
            "state.items",
            RenduExpressionKind::Reference,
        ));
        let item_key = builder.add_expression(RenduExpression::new(
            "item.id",
            RenduExpressionKind::Reference,
        ));
        let dynamic_name = builder.add_expression(RenduExpression::new(
            "state.component",
            RenduExpressionKind::Reference,
        ));
        let label = builder.add_node(RenduNode::Text {
            value: "label".into(),
            provenance: RenduProvenance::generated(),
        });
        let component = builder.add_node(RenduNode::Component {
            kind: RenduComponentKind::Ordinary,
            name: RenduName::Dynamic(dynamic_name),
            properties: vec![RenduProperty::Attribute(RenduAttribute::expression(
                "title", title,
            ))],
            children: vec![label],
            provenance: RenduProvenance::generated(),
        });
        let yes = builder.add_node(RenduNode::Text {
            value: "yes".into(),
            provenance: RenduProvenance::generated(),
        });
        let no = builder.add_node(RenduNode::Text {
            value: "no".into(),
            provenance: RenduProvenance::generated(),
        });
        let conditional = builder.add_node(RenduNode::If {
            branches: vec![
                RenduIfBranch::new(Some(condition), vec![yes]),
                RenduIfBranch::new(None, vec![no]),
            ],
            provenance: RenduProvenance::generated(),
        });
        let iteration = builder.add_node(RenduNode::For {
            source: items,
            value: RenduBinding::new("item"),
            key: None,
            index: Some(RenduBinding::new("index")),
            key_expression: Some(item_key),
            body: vec![component],
            provenance: RenduProvenance::generated(),
        });
        builder.set_entry([component, conditional, iteration]);
        plan_rendu(&builder.finish().unwrap())
    };

    let entry = plan.block(plan.entry()).unwrap();
    let [component, conditional, iteration] = entry.operations.as_slice() else {
        panic!("expected three root operations");
    };
    let VaporOperation::Component {
        kind,
        name,
        properties,
        slots,
        ..
    } = component
    else {
        panic!("expected component operation");
    };
    assert_eq!(*kind, RenduComponentKind::Ordinary);
    assert!(matches!(name, VaporName::Dynamic(_)));
    assert!(matches!(properties[0], VaporProperty::Attribute { .. }));
    let body = slots
        .default
        .expect("ordinary children form a default slot");
    assert!(matches!(
        plan.block(body).unwrap().operations[0],
        VaporOperation::StaticHtml { .. }
    ));

    let VaporOperation::Conditional { branches, .. } = conditional else {
        panic!("expected conditional operation");
    };
    assert_eq!(branches.len(), 2);
    assert!(branches[0].condition.is_some());
    assert!(branches[1].condition.is_none());

    let VaporOperation::Iterate {
        source,
        value,
        index,
        key_expression,
        body,
        ..
    } = iteration
    else {
        panic!("expected iteration operation");
    };
    assert_eq!(
        plan.expression(*source).unwrap().code.as_ref(),
        "state.items"
    );
    assert_eq!(value.pattern.as_ref(), "item");
    assert_eq!(index.as_ref().unwrap().pattern.as_ref(), "index");
    assert_eq!(
        plan.expression(key_expression.unwrap())
            .unwrap()
            .code
            .as_ref(),
        "item.id"
    );
    assert!(matches!(
        plan.block(*body).unwrap().operations[0],
        VaporOperation::Component { .. }
    ));
}

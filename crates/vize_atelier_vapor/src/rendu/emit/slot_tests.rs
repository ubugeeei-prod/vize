use vize_rendu::{
    RenduBinding, RenduBuilder, RenduComponentKind, RenduExpression, RenduExpressionKind,
    RenduIfBranch, RenduName, RenduNode, RenduProvenance,
};

use super::emit_rendu;

#[test]
fn emits_mixed_conditional_and_iterated_slots_from_the_rendu_plan() {
    let mut builder = RenduBuilder::new();
    let condition = builder.add_expression(RenduExpression::new(
        "ready",
        RenduExpressionKind::Reference,
    ));
    let slots = builder.add_expression(RenduExpression::new(
        "availableSlots",
        RenduExpressionKind::Reference,
    ));
    let name = builder.add_expression(RenduExpression::new("name", RenduExpressionKind::Reference));
    let ordinary = builder.add_node(RenduNode::Text {
        value: "ordinary".into(),
        provenance: RenduProvenance::generated(),
    });
    let positive = builder.add_node(RenduNode::SlotContent {
        name: RenduName::static_name("header"),
        bindings: Vec::new(),
        children: Vec::new(),
        provenance: RenduProvenance::generated(),
    });
    let negative = builder.add_node(RenduNode::SlotContent {
        name: RenduName::static_name("header"),
        bindings: Vec::new(),
        children: Vec::new(),
        provenance: RenduProvenance::generated(),
    });
    let conditional = builder.add_node(RenduNode::If {
        branches: vec![
            RenduIfBranch::new(Some(condition), vec![positive]),
            RenduIfBranch::new(None, vec![negative]),
        ],
        provenance: RenduProvenance::generated(),
    });
    let iterated_slot = builder.add_node(RenduNode::SlotContent {
        name: RenduName::Dynamic(name),
        bindings: vec![RenduBinding::new("slotData")],
        children: Vec::new(),
        provenance: RenduProvenance::generated(),
    });
    let iterated = builder.add_node(RenduNode::For {
        source: slots,
        value: RenduBinding::new("_"),
        key: Some(RenduBinding::new("name")),
        index: None,
        key_expression: None,
        body: vec![iterated_slot],
        provenance: RenduProvenance::generated(),
    });
    let component = builder.add_node(RenduNode::Component {
        kind: RenduComponentKind::Ordinary,
        name: RenduName::static_name("Panel"),
        properties: Vec::new(),
        children: vec![ordinary, conditional, iterated],
        provenance: RenduProvenance::generated(),
    });
    builder.push_entry(component);

    let output = emit_rendu(&builder.finish().unwrap());

    for expected in [
        "\"default\": () =>",
        "$: [() => ((ready) ? { name: \"header\"",
        ": { name: \"header\"",
        "() => (_createForSlots(availableSlots, (_, name) => ({ name: name",
        "fn: (slotData) =>",
    ] {
        assert!(
            output.code.contains(expected),
            "missing {expected}:\n{}",
            output.code
        );
    }
}

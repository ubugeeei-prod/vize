use super::compile_rendu;
use vize_rendu::{
    RenduBuilder, RenduEscapeMode, RenduExpression, RenduExpressionKind, RenduNode,
    RenduProvenance, RenduSource,
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

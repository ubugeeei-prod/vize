use vize_rendu::{
    RenduAttribute, RenduBinding, RenduBuilder, RenduDirective, RenduEscapeMode, RenduExpression,
    RenduExpressionKind, RenduIfBranch, RenduName, RenduNamespace, RenduNode, RenduProperty,
    RenduProvenance, RenduSource, RenduSpan,
};

use super::{RenduSsrMappingKind, compile_rendu};

#[test]
fn emits_the_complete_render_vocabulary_deterministically() {
    let mut builder = RenduBuilder::new();
    let source = builder.add_source(RenduSource::named("App.vue", " ".repeat(512)));
    let provenance =
        |start, end| RenduProvenance::from_span(RenduSpan::offsets(source, start, end));
    let message = builder.add_expression(
        RenduExpression::new("_ctx.msg", RenduExpressionKind::Reference)
            .with_provenance(provenance(10, 18)),
    );
    let title = builder.add_expression(RenduExpression::new(
        "_ctx.title",
        RenduExpressionKind::Reference,
    ));
    let attrs = builder.add_expression(RenduExpression::new(
        "_ctx.attrs",
        RenduExpressionKind::Reference,
    ));
    let visible = builder.add_expression(RenduExpression::new(
        "_ctx.visible",
        RenduExpressionKind::Reference,
    ));
    let condition = builder.add_expression(RenduExpression::new(
        "_ctx.ok",
        RenduExpressionKind::Reference,
    ));
    let items = builder.add_expression(RenduExpression::new(
        "_ctx.items",
        RenduExpressionKind::Reference,
    ));

    let text = builder.add_node(RenduNode::Text {
        value: "<hello>".into(),
        provenance: provenance(20, 27),
    });
    let interpolation = builder.add_node(RenduNode::Expression {
        expression: message,
        escape: RenduEscapeMode::Escaped,
        provenance: provenance(28, 37),
    });
    let section = builder.add_node(RenduNode::Element {
        tag: "section".into(),
        namespace: RenduNamespace::Html,
        properties: vec![
            RenduProperty::Attribute(
                RenduAttribute::static_value("id", "main").with_provenance(provenance(38, 47)),
            ),
            RenduProperty::Attribute(RenduAttribute::expression("title", title)),
            RenduProperty::spread(attrs),
            RenduProperty::Directive(RenduDirective::new("show").with_expression(visible)),
            RenduProperty::Directive(
                RenduDirective::new("tooltip")
                    .with_expression(message)
                    .with_argument(RenduName::static_name("top"))
                    .with_modifier("eager")
                    .with_provenance(provenance(48, 60)),
            ),
        ],
        children: vec![text, interpolation],
        provenance: provenance(19, 90),
    });

    let slot_text = builder.add_node(RenduNode::Text {
        value: "heading".into(),
        provenance: provenance(91, 98),
    });
    let named_slot = builder.add_node(RenduNode::SlotContent {
        name: RenduName::static_name("header"),
        bindings: vec![RenduBinding::new("slotProps").with_provenance(provenance(99, 108))],
        children: vec![slot_text],
        provenance: provenance(90, 105),
    });
    let component = builder.add_node(RenduNode::Component {
        name: RenduName::static_name("Card"),
        properties: vec![RenduProperty::Attribute(RenduAttribute::expression(
            "title", title,
        ))],
        children: vec![named_slot, section],
        provenance: provenance(89, 150),
    });
    let fallback = builder.add_node(RenduNode::Text {
        value: "fallback".into(),
        provenance: provenance(151, 159),
    });
    let outlet = builder.add_node(RenduNode::SlotOutlet {
        name: RenduName::static_name("footer"),
        properties: vec![RenduProperty::Attribute(RenduAttribute::static_value(
            "tone", "quiet",
        ))],
        fallback: vec![fallback],
        provenance: provenance(150, 180),
    });
    let conditional = builder.add_node(RenduNode::If {
        branches: vec![
            RenduIfBranch::new(Some(condition), vec![component])
                .with_provenance(provenance(181, 200)),
            RenduIfBranch::new(None, vec![outlet]),
        ],
        provenance: provenance(181, 230),
    });
    let iteration = builder.add_node(RenduNode::For {
        source: items,
        value: RenduBinding::new("item").with_provenance(provenance(235, 239)),
        key: Some(RenduBinding::new("key")),
        index: Some(RenduBinding::new("index")),
        key_expression: None,
        body: vec![interpolation],
        provenance: provenance(231, 270),
    });
    builder.set_entry([conditional, iteration]);
    let root = builder.finish().expect("valid Rendu graph");

    let first = compile_rendu(&root);
    let second = compile_rendu(&root);
    assert_eq!(first, second);
    assert!(first.code.contains("_push(\"&lt;hello&gt;\")"));
    assert!(first.code.contains("_ssrInterpolate(_ctx.msg)"));
    assert!(first.code.contains("_push(\"<section\")"));
    assert!(first.code.contains("_ssrRenderAttr(\"title\", _ctx.title)"));
    assert!(first.code.contains("_ssrRenderAttrs(_ctx.attrs)"));
    assert!(first.code.contains("display: none"));
    assert!(first.code.contains("_ssrGetDirectiveProps"));
    assert!(first.code.contains("_resolveDirective(\"tooltip\")"));
    assert!(
        first
            .code
            .contains("_ssrRenderComponent(_resolveComponent(\"Card\")")
    );
    assert!(
        first
            .code
            .contains("\"header\": (slotProps, _push, _parent)")
    );
    assert!(
        first
            .code
            .contains("_ssrRenderSlot(_ctx.$slots, \"footer\"")
    );
    assert!(first.code.contains("if (_ctx.ok)"));
    assert!(
        first
            .code
            .contains("_ssrRenderList(_ctx.items, (item, key, index) =>")
    );
    assert!(
        first
            .mappings
            .iter()
            .any(|mapping| mapping.kind == RenduSsrMappingKind::Node)
    );
    assert!(
        first
            .mappings
            .iter()
            .any(|mapping| mapping.kind == RenduSsrMappingKind::Property)
    );
    assert!(first.mappings.iter().any(|mapping| {
        mapping.kind == RenduSsrMappingKind::Expression
            && mapping.source == RenduSpan::offsets(source, 10, 18)
    }));
    assert!(
        first
            .mappings
            .iter()
            .any(|mapping| mapping.kind == RenduSsrMappingKind::Binding)
    );
    assert!(
        first
            .mappings
            .iter()
            .any(|mapping| mapping.kind == RenduSsrMappingKind::Branch)
    );
}

#[test]
fn raw_and_text_directives_replace_element_children() {
    let mut builder = RenduBuilder::new();
    let raw = builder.add_expression(RenduExpression::new(
        "_ctx.raw",
        RenduExpressionKind::Reference,
    ));
    let ignored = builder.add_node(RenduNode::Text {
        value: "ignored".into(),
        provenance: RenduProvenance::generated(),
    });
    let element = builder.add_node(RenduNode::Element {
        tag: "div".into(),
        namespace: RenduNamespace::Html,
        properties: vec![RenduProperty::Directive(
            RenduDirective::new("html").with_expression(raw),
        )],
        children: vec![ignored],
        provenance: RenduProvenance::generated(),
    });
    builder.push_entry(element);

    let output = compile_rendu(&builder.finish().unwrap());
    assert!(output.code.contains("_push((_ctx.raw) ?? \"\")"));
    assert!(!output.code.contains("ignored"));
}

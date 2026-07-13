use std::collections::BTreeSet;

use vize_atelier_jsx::{
    JsxLang, JsxSyntaxNode, JsxSyntaxSnapshot, lower_source_to_rendu, snapshot_jsx,
    snapshot_jsx_named,
};
use vize_rendu::{
    RenduAttributeValue, RenduCapability, RenduComponentKind, RenduNamespace, RenduNode,
    RenduProperty, RenduWalkEvent, walk_rendu,
};

fn assert_owned_snapshot<T: Send + Sync + 'static>() {}

#[test]
fn snapshot_is_owned_after_oxc_is_dropped_and_uses_no_relief_api() {
    let direct_path_source = concat!(
        include_str!("../src/syntax.rs"),
        include_str!("../src/syntax/build.rs"),
        include_str!("../src/syntax/control.rs"),
        include_str!("../src/syntax/text.rs"),
        include_str!("../src/rendu.rs"),
        include_str!("../src/rendu/property.rs"),
    );
    assert!(!direct_path_source.contains("vize_relief"));
    assert!(!direct_path_source.contains("RootNode"));

    assert_owned_snapshot::<JsxSyntaxSnapshot>();
    let snapshot = snapshot_jsx_named(
        "Card.tsx",
        "const Card = () => <article>{title}</article>;",
        JsxLang::Tsx,
    );
    assert!(!snapshot.has_errors(), "{:?}", snapshot.diagnostics);
    assert_eq!(snapshot.filename.as_deref(), Some("Card.tsx"));
    assert_eq!(snapshot.roots.len(), 1);

    let root = std::thread::spawn(move || snapshot.lower_rendu())
        .join()
        .expect("owned snapshot crosses a thread")
        .expect("snapshot lowers to valid Rendu");
    assert_eq!(root.entry().len(), 1);
    assert_eq!(root.sources()[0].language.as_deref(), Some("tsx"));
}

#[test]
fn direct_graph_path_covers_render_structure_props_and_provenance() {
    let source = r#"
const App = (): JSX.Element => (
  <>
    <section id="main" hidden data-count={count} {...rest} v-show={visible}>
      {/* keep me */}
      {ready && <Widget active={state.ok} />}
      {flag ? <span>A</span> : <em>{fallback}</em>}
      {items.map((item, index) => <Row key={item.id}>{item.name}</Row>)}
    </section>
  </>
);
"#;
    let output = lower_source_to_rendu(source, JsxLang::Tsx).expect("valid Rendu graph");
    assert!(
        !output.snapshot.has_errors(),
        "{:?}",
        output.snapshot.diagnostics
    );
    output
        .root
        .validate()
        .expect("builder returned valid graph");

    let capabilities = output.root.capabilities();
    for capability in [
        RenduCapability::Elements,
        RenduCapability::Components,
        RenduCapability::Text,
        RenduCapability::Expressions,
        RenduCapability::Properties,
        RenduCapability::Directives,
        RenduCapability::Conditionals,
        RenduCapability::Iteration,
        RenduCapability::SourceProvenance,
    ] {
        assert!(capabilities.contains(capability), "missing {capability:?}");
    }

    let section = output
        .root
        .nodes()
        .iter()
        .find_map(|node| match node {
            RenduNode::Element {
                tag, properties, ..
            } if tag.as_ref() == "section" => Some(properties),
            _ => None,
        })
        .expect("section element");
    assert!(section.iter().any(|property| matches!(
        property,
        RenduProperty::Attribute(attribute)
            if attribute.name == vize_rendu::RenduName::static_name("id")
                && matches!(&attribute.value, Some(RenduAttributeValue::Static(value)) if value.as_ref() == "main")
    )));
    assert!(section.iter().any(|property| matches!(
        property,
        RenduProperty::Attribute(attribute)
            if attribute.name == vize_rendu::RenduName::static_name("hidden")
                && attribute.value.is_none()
    )));
    assert!(section.iter().any(|property| matches!(
        property,
        RenduProperty::Attribute(attribute)
            if attribute.name == vize_rendu::RenduName::static_name("data-count")
                && matches!(&attribute.value, Some(RenduAttributeValue::Expression(_)))
    )));
    assert!(
        section
            .iter()
            .any(|property| matches!(property, RenduProperty::Spread { .. }))
    );
    assert!(section.iter().any(|property| matches!(
        property,
        RenduProperty::Directive(directive) if directive.name.as_ref() == "show"
    )));

    assert!(output.root.nodes().iter().any(
        |node| matches!(node, RenduNode::Comment { value, .. } if value.as_ref() == "keep me")
    ));
    assert!(
        output
            .root
            .nodes()
            .iter()
            .any(|node| matches!(node, RenduNode::If { .. }))
    );
    assert!(output.root.nodes().iter().any(
        |node| matches!(node, RenduNode::For { value, index: Some(index), .. }
            if value.pattern.as_ref() == "item" && index.pattern.as_ref() == "index")
    ));
    assert!(output.root.nodes().iter().all(|node| !matches!(
        node,
        RenduNode::Component { kind, .. } if *kind != RenduComponentKind::Ordinary
    )));

    let expression_codes = output
        .root
        .expressions()
        .iter()
        .map(|expression| expression.code.as_ref())
        .collect::<BTreeSet<_>>();
    for expected in [
        "count",
        "rest",
        "visible",
        "ready",
        "state.ok",
        "flag",
        "fallback",
        "items",
        "item.id",
        "item.name",
    ] {
        assert!(expression_codes.contains(expected), "missing {expected}");
    }
    assert!(output.root.nodes().iter().all(|node| {
        node.provenance()
            .primary
            .is_some_and(|span| span.source == vize_rendu::RenduSourceId::new(0))
    }));

    let mut entered = 0;
    walk_rendu(&output.root, |event| {
        if matches!(event, RenduWalkEvent::Enter { .. }) {
            entered += 1;
        }
    });
    assert!(entered >= 12);
}

#[test]
fn root_level_conditionals_and_logical_fallbacks_stay_graph_native() {
    let conditional = snapshot_jsx("const View = () => ok ? <A /> : <B />;", JsxLang::Jsx);
    assert_eq!(conditional.roots.len(), 1);
    assert!(matches!(conditional.roots[0], JsxSyntaxNode::If { .. }));

    let fallback = snapshot_jsx("const View = () => value || <Fallback />;", JsxLang::Jsx);
    let JsxSyntaxNode::If { branches, .. } = &fallback.roots[0] else {
        panic!("logical fallback must be a structural branch");
    };
    let condition = branches[0].condition.as_ref().expect("condition");
    assert!(condition.synthetic);
    assert_eq!(condition.code.as_ref(), "!(value)");
}

#[test]
fn map_callbacks_support_function_and_block_arrow_forms() {
    let output = lower_source_to_rendu(
        r#"
const Lists = () => <>
  {first.map(function (item, index) { return <A>{item}</A>; })}
  {second.map((entry) => { return <B>{entry}</B>; })}
</>;
"#,
        JsxLang::Tsx,
    )
    .expect("map callbacks lower");
    let loops = output
        .root
        .nodes()
        .iter()
        .filter(|node| matches!(node, RenduNode::For { .. }))
        .count();
    assert_eq!(loops, 2);
    let expressions = output
        .root
        .expressions()
        .iter()
        .map(|expression| expression.code.as_ref())
        .collect::<BTreeSet<_>>();
    assert!(expressions.contains("first"));
    assert!(expressions.contains("second"));
}

#[test]
fn svg_namespace_propagates_and_foreign_object_returns_to_html() {
    let output = lower_source_to_rendu(
        "const Icon = () => <svg><path /><foreignObject><div /></foreignObject></svg>;",
        JsxLang::Jsx,
    )
    .expect("valid graph");
    let namespace = |tag: &str| {
        output.root.nodes().iter().find_map(|node| match node {
            RenduNode::Element {
                tag: present,
                namespace,
                ..
            } if present.as_ref() == tag => Some(namespace),
            _ => None,
        })
    };
    assert_eq!(namespace("svg"), Some(&RenduNamespace::Svg));
    assert_eq!(namespace("path"), Some(&RenduNamespace::Svg));
    assert_eq!(namespace("foreignObject"), Some(&RenduNamespace::Svg));
    assert_eq!(namespace("div"), Some(&RenduNamespace::Html));
}

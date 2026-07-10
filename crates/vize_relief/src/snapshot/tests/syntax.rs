use vize_carton::{Box, Bump, directive::DirectiveKind};

use crate::{
    AttributeNode, CommentKind, CommentNode, DirectiveNode, ElementNode, ElementType, ImportItem,
    Namespace, PropNode, RootNode, RuntimeHelper, SimpleExpressionNode, TemplateChildNode,
};

use super::{assert_owned_product, expression, interpolation, location, text};
use crate::{
    ReliefSnapshot, ReliefSnapshotNode, ReliefSnapshotNodeKind, SnapshotProp,
    SnapshotSimpleExpression,
};

fn build_snapshot() -> ReliefSnapshot {
    let allocator = Bump::new();
    let mut root = RootNode::new(
        &allocator,
        "<section id=\"main\" @click.stop=\"run\">hello {{ name }}<!--note--></section>",
    );
    root.loc = location(0, 74, "section-root");
    root.helpers.push(RuntimeHelper::ToDisplayString);
    root.components.push("UserCard".into());
    root.directives.push("focus".into());
    root.temps = 2;
    root.transformed = true;
    root.imports.push(ImportItem {
        exp: Box::new_in(
            SimpleExpressionNode::new("asset", false, location(0, 5, "asset")),
            &allocator,
        ),
        path: "./asset".into(),
    });

    let mut element = ElementNode::new(&allocator, "section", location(0, 74, "section"));
    element.ns = Namespace::Html;
    element.tag_type = ElementType::Element;
    element.inner_loc = Some(location(42, 69, "inner"));

    let mut attribute = AttributeNode::new("id", location(9, 18, "id=\"main\""));
    attribute.name_loc = location(9, 11, "id");
    attribute.value = Some(text("main", 13, 17));
    element
        .props
        .push(PropNode::Attribute(Box::new_in(attribute, &allocator)));

    let mut directive =
        DirectiveNode::new(&allocator, "on", location(19, 41, "@click.stop=\"run\""));
    directive.raw_name = Some("@click.stop".into());
    directive.exp = Some(expression(&allocator, "run", 36, 39));
    directive.arg = Some(expression(&allocator, "click", 20, 25));
    directive.modifiers.push(SimpleExpressionNode::new(
        "stop",
        true,
        location(26, 30, "stop"),
    ));
    element
        .props
        .push(PropNode::Directive(Box::new_in(directive, &allocator)));

    element.children.push(TemplateChildNode::Text(Box::new_in(
        text("hello ", 42, 48),
        &allocator,
    )));
    element
        .children
        .push(TemplateChildNode::Interpolation(Box::new_in(
            interpolation(&allocator, "name", 48, 58),
            &allocator,
        )));
    let mut comment = CommentNode::new("note", location(58, 69, "<!--note-->"));
    comment.kind = CommentKind::InTag;
    comment.directive = Some(DirectiveKind::Docs);
    root.comments.push(CommentNode::new(
        "root-note",
        location(58, 69, "<!--note-->"),
    ));
    element
        .children
        .push(TemplateChildNode::Comment(Box::new_in(comment, &allocator)));
    root.children
        .push(TemplateChildNode::Element(Box::new_in(element, &allocator)));

    ReliefSnapshot::from_root(&root)
}

#[test]
fn snapshot_is_owned_and_preserves_element_syntax() {
    let snapshot = build_snapshot();
    assert_owned_product(&snapshot);
    assert_eq!(snapshot.source().len(), 74);
    assert_eq!(snapshot.location().start.offset, 0);
    assert_eq!(snapshot.helpers(), &[RuntimeHelper::ToDisplayString]);
    assert_eq!(snapshot.components()[0].as_str(), "UserCard");
    assert_eq!(snapshot.directives()[0].as_str(), "focus");
    assert_eq!(snapshot.temps(), 2);
    assert!(snapshot.transformed());
    assert_eq!(snapshot.imports()[0].path.as_str(), "./asset");
    assert_eq!(snapshot.comments()[0].content.as_str(), "root-note");

    let root_id = snapshot.children()[0];
    let ReliefSnapshotNode::Element(element) = snapshot.node(root_id).expect("element") else {
        panic!("expected element snapshot");
    };
    assert_eq!(element.tag.as_str(), "section");
    assert_eq!(element.namespace, Namespace::Html);
    assert_eq!(
        element.inner_location.as_ref().map(|loc| loc.start.offset),
        Some(42)
    );
    assert_eq!(element.props.len(), 2);
    let SnapshotProp::Attribute(attribute) = &element.props[0] else {
        panic!("expected attribute");
    };
    assert_eq!(attribute.name.as_str(), "id");
    assert_eq!(
        attribute.value.as_ref().map(|value| value.content.as_str()),
        Some("main")
    );
    let SnapshotProp::Directive(directive) = &element.props[1] else {
        panic!("expected directive");
    };
    assert_eq!(directive.name.as_str(), "on");
    assert_eq!(
        directive.raw_name.as_ref().map(|name| name.as_str()),
        Some("@click.stop")
    );
    assert_eq!(directive.modifiers[0].content.as_str(), "stop");
    assert_eq!(directive.modifiers[0].location.start.column, 27);
    assert!(matches!(
        directive.expression,
        Some(crate::SnapshotExpression::Simple(SnapshotSimpleExpression { ref content, .. }))
            if content.as_str() == "run"
    ));
    assert!(matches!(
        directive.argument,
        Some(crate::SnapshotExpression::Simple(SnapshotSimpleExpression { ref content, .. }))
            if content.as_str() == "click"
    ));

    let children = element.children();
    let ReliefSnapshotNode::Text(text) = snapshot.node(children[0]).expect("text") else {
        panic!("expected text");
    };
    assert_eq!(text.content.as_str(), "hello ");
    let ReliefSnapshotNode::Interpolation(interpolation) =
        snapshot.node(children[1]).expect("interpolation")
    else {
        panic!("expected interpolation");
    };
    assert_eq!(interpolation.location.start.offset, 48);
    let ReliefSnapshotNode::Comment(comment) = snapshot.node(children[2]).expect("comment") else {
        panic!("expected comment");
    };
    assert_eq!(comment.content.as_str(), "note");
    assert_eq!(comment.kind, CommentKind::InTag);
    assert_eq!(comment.directive, Some(DirectiveKind::Docs));

    let kinds: Vec<_> = snapshot.walk().map(|visit| visit.node.kind()).collect();
    assert_eq!(
        kinds,
        vec![
            ReliefSnapshotNodeKind::Element,
            ReliefSnapshotNodeKind::Text,
            ReliefSnapshotNodeKind::Interpolation,
            ReliefSnapshotNodeKind::Comment,
        ]
    );
    let (_, node) = snapshot
        .node_at_offset(50)
        .expect("interpolation at offset");
    assert_eq!(node.kind(), ReliefSnapshotNodeKind::Interpolation);
}

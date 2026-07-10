use vize_carton::{Box, Bump};

use crate::{
    CompoundExpressionChild, CompoundExpressionNode, ReliefSnapshot, ReliefSnapshotNode, RootNode,
    RuntimeHelper, SimpleExpressionNode, TemplateChildNode, TextCallContent, TextCallNode,
};

use super::{interpolation, location, text};
use crate::{SnapshotCompoundChild, SnapshotTextCallContent};

#[test]
fn compound_and_text_call_parts_remain_in_exact_order() {
    let allocator = Bump::new();
    let mut root = RootNode::new(&allocator, "compound-fixture");
    let mut compound = CompoundExpressionNode::new(&allocator, location(0, 20, "compound"));
    compound
        .children
        .push(CompoundExpressionChild::String("prefix:".into()));
    compound
        .children
        .push(CompoundExpressionChild::Simple(Box::new_in(
            SimpleExpressionNode::new("name", false, location(7, 11, "name")),
            &allocator,
        )));
    compound.children.push(CompoundExpressionChild::Symbol(
        RuntimeHelper::ToDisplayString,
    ));
    compound
        .children
        .push(CompoundExpressionChild::Interpolation(Box::new_in(
            interpolation(&allocator, "count", 11, 20),
            &allocator,
        )));
    compound
        .children
        .push(CompoundExpressionChild::Text(Box::new_in(
            text("!", 20, 21),
            &allocator,
        )));
    root.children
        .push(TemplateChildNode::CompoundExpression(Box::new_in(
            compound, &allocator,
        )));

    root.children.push(TemplateChildNode::TextCall(Box::new_in(
        TextCallNode {
            content: TextCallContent::Interpolation(Box::new_in(
                interpolation(&allocator, "total", 22, 33),
                &allocator,
            )),
            loc: location(22, 33, "text-call"),
        },
        &allocator,
    )));
    root.children.push(TemplateChildNode::Hoisted(7));

    let snapshot = ReliefSnapshot::from_root(&root);
    let ReliefSnapshotNode::CompoundExpression(compound) =
        snapshot.node(snapshot.children()[0]).expect("compound")
    else {
        panic!("expected compound expression");
    };
    assert!(matches!(
        compound.children.as_slice(),
        [
            SnapshotCompoundChild::String(prefix),
            SnapshotCompoundChild::Simple(_),
            SnapshotCompoundChild::Symbol(RuntimeHelper::ToDisplayString),
            SnapshotCompoundChild::Interpolation(_),
            SnapshotCompoundChild::Text(_),
        ] if prefix.as_str() == "prefix:"
    ));

    let ReliefSnapshotNode::TextCall(call) =
        snapshot.node(snapshot.children()[1]).expect("text call")
    else {
        panic!("expected text call");
    };
    assert!(matches!(
        call.content,
        SnapshotTextCallContent::Interpolation(ref value)
            if value.content.location().source.as_str() == "total"
    ));
    let ReliefSnapshotNode::Hoisted(hoisted) =
        snapshot.node(snapshot.children()[2]).expect("hoist")
    else {
        panic!("expected hoisted reference");
    };
    assert_eq!(hoisted.index, 7);
}

use vize_carton::{Box, Bump, Vec as ArenaVec};

use crate::{
    ForNode, ForParseResult, IfBranchNode, IfNode, ReliefSnapshot, ReliefSnapshotNode,
    ReliefSnapshotNodeId, ReliefSnapshotNodeKind, RootNode, SnapshotExpression, TemplateChildNode,
};

use super::{expression, location, text};

#[test]
fn control_flow_retains_branch_loop_nesting_and_order() {
    let allocator = Bump::new();
    let mut root = RootNode::new(&allocator, "if-for-fixture");
    let mut if_node = IfNode::new(&allocator, location(0, 60, "if"));

    let mut first = IfBranchNode::new(
        &allocator,
        Some(expression(&allocator, "ready", 1, 6)),
        location(0, 20, "if-ready"),
    );
    first.children.push(TemplateChildNode::Text(Box::new_in(
        text("ready", 10, 15),
        &allocator,
    )));
    if_node.branches.push(first);

    let mut second = IfBranchNode::new(&allocator, None, location(20, 60, "else"));
    let mut loop_children = ArenaVec::new_in(&allocator);
    loop_children.push(TemplateChildNode::Text(Box::new_in(
        text("row", 45, 48),
        &allocator,
    )));
    let for_node = ForNode {
        source: expression(&allocator, "items", 25, 30),
        value_alias: Some(expression(&allocator, "item", 31, 35)),
        key_alias: Some(expression(&allocator, "key", 36, 39)),
        object_index_alias: Some(expression(&allocator, "index", 40, 45)),
        parse_result: ForParseResult {
            source: expression(&allocator, "items", 25, 30),
            value: Some(expression(&allocator, "item", 31, 35)),
            key: Some(expression(&allocator, "key", 36, 39)),
            index: Some(expression(&allocator, "index", 40, 45)),
            finalized: true,
        },
        children: loop_children,
        loc: location(24, 55, "for"),
    };
    second
        .children
        .push(TemplateChildNode::For(Box::new_in(for_node, &allocator)));
    if_node.branches.push(second);
    root.children
        .push(TemplateChildNode::If(Box::new_in(if_node, &allocator)));

    let snapshot = ReliefSnapshot::from_root(&root);
    let visits: Vec<_> = snapshot
        .walk()
        .map(|visit| (visit.node.kind(), visit.depth))
        .collect();
    assert_eq!(
        visits,
        vec![
            (ReliefSnapshotNodeKind::If, 0),
            (ReliefSnapshotNodeKind::IfBranch, 1),
            (ReliefSnapshotNodeKind::Text, 2),
            (ReliefSnapshotNodeKind::IfBranch, 1),
            (ReliefSnapshotNodeKind::For, 2),
            (ReliefSnapshotNodeKind::Text, 3),
        ]
    );

    let if_id = snapshot.children()[0];
    let ReliefSnapshotNode::If(if_snapshot) = snapshot.node(if_id).expect("if node") else {
        panic!("expected if node");
    };
    assert_eq!(if_snapshot.branches().len(), 2);
    let ReliefSnapshotNode::IfBranch(first_branch) = snapshot
        .node(if_snapshot.branches()[0])
        .expect("first branch")
    else {
        panic!("expected first branch");
    };
    assert!(matches!(
        first_branch.condition,
        Some(SnapshotExpression::Simple(ref condition)) if condition.content.as_str() == "ready"
    ));
    let second_branch = if_snapshot.branches()[1];
    let ReliefSnapshotNode::IfBranch(branch) = snapshot.node(second_branch).expect("branch") else {
        panic!("expected branch");
    };
    let ReliefSnapshotNode::For(loop_node) = snapshot.node(branch.children()[0]).expect("loop")
    else {
        panic!("expected for node");
    };
    assert!(loop_node.parse_result.finalized);
    assert!(matches!(
        loop_node.value_alias,
        Some(SnapshotExpression::Simple(ref alias)) if alias.content.as_str() == "item"
    ));
    assert_eq!(loop_node.children().len(), 1);
    assert_eq!(snapshot.walk_from(second_branch).count(), 3);
    assert_eq!(
        snapshot
            .walk_from(ReliefSnapshotNodeId::from_raw(u32::MAX))
            .count(),
        0
    );

    let materialized_allocator = Bump::new();
    let materialized = snapshot.materialize(&materialized_allocator);
    assert_eq!(ReliefSnapshot::from_root(&materialized), snapshot);
}

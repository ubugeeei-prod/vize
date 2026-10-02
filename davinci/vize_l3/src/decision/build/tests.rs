use super::{
    Builder, ControlKind, ControlRegion, DecisionBuildError, NodeDecision, NodeId, Placement,
    SideTable, StaticLevel, TargetPolicy,
};

fn id(index: u32) -> NodeId {
    NodeId::from_index(index).unwrap()
}

fn builder(node_count: u32) -> Builder<'static, 'static, 'static, super::LiteralExpressions> {
    Builder {
        policy: TargetPolicy::Ssr,
        node_count,
        frames: Default::default(),
        nodes: SideTable::new(),
        controls: SideTable::new(),
        dom: None,
    }
}

fn row() -> NodeDecision {
    NodeDecision {
        static_level: StaticLevel::Static,
        output_level: StaticLevel::Static,
        dynamic_bindings: Default::default(),
        placement: Placement::Inline,
        control: None,
    }
}

#[test]
fn insertion_rejects_out_of_owner_keys_and_duplicate_nodes() {
    let mut builder = builder(2);
    assert_eq!(
        builder.insert_node(id(2), row()),
        Err(DecisionBuildError::InvalidNode { node: id(2) })
    );
    assert!(builder.nodes.is_empty());
    builder.insert_node(id(1), row()).unwrap();
    assert_eq!(
        builder.insert_node(id(1), row()),
        Err(DecisionBuildError::DuplicateNode { node: id(1) })
    );
    assert_eq!(builder.nodes.len(), 1);
}

#[test]
fn finish_requires_every_unique_key_in_the_owner_range() {
    let mut builder = builder(2);
    builder.insert_node(id(1), row()).unwrap();
    assert_eq!(
        builder.finish(),
        Err(DecisionBuildError::NodeCountMismatch {
            expected: 2,
            actual: 1
        })
    );
    let mut builder = self::builder(2);
    builder.insert_node(id(1), row()).unwrap();
    builder.insert_node(id(0), row()).unwrap();
    let complete = builder.finish().unwrap();
    assert_eq!(complete.nodes.len(), 2);
    assert_eq!(complete.policy, TargetPolicy::Ssr);
}

#[test]
fn control_insertion_rejects_out_of_owner_keys_and_repeated_owners() {
    let mut builder = builder(1);
    let control = ControlRegion {
        kind: ControlKind::Loop,
        parent: None,
    };
    assert_eq!(
        builder.insert_control(id(1), control),
        Err(DecisionBuildError::InvalidNode { node: id(1) })
    );
    assert!(builder.controls.is_empty());
    builder.insert_control(id(0), control).unwrap();
    assert_eq!(
        builder.insert_control(id(0), control),
        Err(DecisionBuildError::DuplicateControl { node: id(0) })
    );
}

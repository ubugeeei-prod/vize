use vize_l3::op::{EdgeKind, OpKind, Phase};

#[test]
fn op_kind_mnemonics_are_the_vapor_generalization_set() {
    let kinds = [
        (OpKind::SetProp, "l3.set-prop"),
        (OpKind::SetDynamicProps, "l3.set-dynamic-props"),
        (OpKind::SetText, "l3.set-text"),
        (OpKind::SetEvent, "l3.set-event"),
        (OpKind::SetHtml, "l3.set-html"),
        (OpKind::SetTemplateRef, "l3.set-template-ref"),
        (OpKind::InsertNode, "l3.insert-node"),
        (OpKind::PrependNode, "l3.prepend-node"),
        (OpKind::Directive, "l3.directive"),
        (OpKind::If, "l3.if"),
        (OpKind::For, "l3.for"),
        (OpKind::CreateComponent, "l3.create-component"),
        (OpKind::SlotOutlet, "l3.slot-outlet"),
        (OpKind::GetTextChild, "l3.get-text-child"),
        (OpKind::ChildRef, "l3.child-ref"),
        (OpKind::NextRef, "l3.next-ref"),
    ];
    assert_eq!(kinds.len(), 16);
    for (kind, mnemonic) in kinds {
        assert_eq!(kind.mnemonic(), mnemonic);
        assert_eq!(OpKind::from_mnemonic(mnemonic), Some(kind));
    }
}

#[test]
fn phase_and_edge_spellings_are_stable() {
    assert_eq!(Phase::Built.as_str(), "built");
    assert_eq!(Phase::Partitioned.as_str(), "partitioned");
    assert_eq!(Phase::Scheduled.as_str(), "scheduled");

    assert_eq!(EdgeKind::DomOrder.as_str(), "dom-order");
    assert_eq!(EdgeKind::EffectOrder.as_str(), "effect-order");
    assert_eq!(EdgeKind::DataDependency.as_str(), "data-dependency");
    assert_eq!(
        EdgeKind::from_str(EdgeKind::DataDependency.as_str()),
        Some(EdgeKind::DataDependency)
    );
}

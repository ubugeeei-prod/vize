use vize_impeto::op::{EdgeKind, OpKind, Phase};

#[test]
fn op_kind_mnemonics_are_the_vapor_generalization_set() {
    let kinds = [
        (OpKind::SetProp, "impeto.set-prop"),
        (OpKind::SetDynamicProps, "impeto.set-dynamic-props"),
        (OpKind::SetText, "impeto.set-text"),
        (OpKind::SetEvent, "impeto.set-event"),
        (OpKind::SetHtml, "impeto.set-html"),
        (OpKind::SetTemplateRef, "impeto.set-template-ref"),
        (OpKind::InsertNode, "impeto.insert-node"),
        (OpKind::PrependNode, "impeto.prepend-node"),
        (OpKind::Directive, "impeto.directive"),
        (OpKind::If, "impeto.if"),
        (OpKind::For, "impeto.for"),
        (OpKind::CreateComponent, "impeto.create-component"),
        (OpKind::SlotOutlet, "impeto.slot-outlet"),
        (OpKind::GetTextChild, "impeto.get-text-child"),
        (OpKind::ChildRef, "impeto.child-ref"),
        (OpKind::NextRef, "impeto.next-ref"),
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

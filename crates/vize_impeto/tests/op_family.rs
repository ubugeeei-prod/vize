use vize_impeto::op::{EdgeKind, OpKind, Phase};

#[test]
fn op_kind_mnemonics_are_the_vapor_generalization_set() {
    let kinds = [
        OpKind::SetProp,
        OpKind::SetDynamicProps,
        OpKind::SetText,
        OpKind::SetEvent,
        OpKind::SetHtml,
        OpKind::SetTemplateRef,
        OpKind::InsertNode,
        OpKind::PrependNode,
        OpKind::Directive,
        OpKind::If,
        OpKind::For,
        OpKind::CreateComponent,
        OpKind::SlotOutlet,
        OpKind::GetTextChild,
        OpKind::ChildRef,
        OpKind::NextRef,
    ];
    assert_eq!(kinds.len(), 16);
    for kind in kinds {
        assert_eq!(OpKind::from_mnemonic(kind.mnemonic()), Some(kind));
        assert!(kind.mnemonic().starts_with("impeto."));
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

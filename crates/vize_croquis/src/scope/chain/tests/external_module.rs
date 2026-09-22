use super::{
    BindingType, CompactString, ExternalModuleScopeData, ScopeBinding, ScopeChain, ScopeKind,
};

#[test]
fn test_external_module_scope() {
    let mut chain = ScopeChain::new();

    chain.enter_external_module_scope(
        ExternalModuleScopeData {
            source: CompactString::new("vue"),
            is_type_only: false,
            exports: Vec::new(),
        },
        0,
        50,
    );

    assert_eq!(chain.current_scope().kind, ScopeKind::ExternalModule);

    // Add imports from external module
    chain.add_binding(
        CompactString::new("ref"),
        ScopeBinding::new(BindingType::ExternalModule, 10),
    );
    chain.add_binding(
        CompactString::new("computed"),
        ScopeBinding::new(BindingType::ExternalModule, 15),
    );

    assert!(chain.is_defined("ref"));
    assert!(chain.is_defined("computed"));

    let (_, binding) = chain.lookup("ref").unwrap();
    assert_eq!(binding.binding_type, BindingType::ExternalModule);
}

use super::*;

#[test]
fn binding_metadata_projects_into_lattice_classes() {
    assert_eq!(
        binding_kind_lattice_fact(BindingKind::SetupConst).proven_class(),
        Some(ReactivityClass::Static)
    );
    assert_eq!(
        binding_kind_lattice_fact(BindingKind::LiteralConst).proven_class(),
        Some(ReactivityClass::Static)
    );
    assert_eq!(
        binding_kind_lattice_fact(BindingKind::ExternalModule).proven_class(),
        Some(ReactivityClass::Static)
    );
    assert_eq!(
        binding_kind_lattice_fact(BindingKind::Props).proven_class(),
        Some(ReactivityClass::PropsStable)
    );
    assert_eq!(
        binding_kind_lattice_fact(BindingKind::PropsAliased).proven_class(),
        Some(ReactivityClass::PropsStable)
    );
    assert_eq!(
        binding_kind_lattice_fact(BindingKind::SetupRef).proven_class(),
        Some(ReactivityClass::Reactive)
    );
    assert_eq!(
        binding_kind_lattice_fact(BindingKind::Options).proven_class(),
        Some(ReactivityClass::Reactive)
    );
}

#[test]
fn handler_patch_gate_keeps_the_shipped_constant_surface() {
    assert!(handler_static_patch_binding(BindingKind::SetupConst));
    assert!(handler_static_patch_binding(BindingKind::LiteralConst));

    assert!(!handler_static_patch_binding(BindingKind::ExternalModule));
    assert!(!handler_static_patch_binding(
        BindingKind::JsGlobalUniversal
    ));
    assert!(!handler_static_patch_binding(BindingKind::Props));
    assert!(!handler_static_patch_binding(BindingKind::SetupRef));
    assert!(!handler_static_patch_binding(BindingKind::SetupLet));
}

#[test]
fn patch_facts_materialize_lattice_handler_gate() {
    let setup_const = materialized_handler_facts(BindingKind::SetupConst);
    assert_eq!(setup_const.flag, 0);
    assert!(setup_const.dynamic_props.is_empty());

    let setup_ref = materialized_handler_facts(BindingKind::SetupRef);
    assert_eq!(setup_ref.flag, 8);
    assert_eq!(setup_ref.dynamic_props, [String::from("onClick")]);
}

#[test]
fn lattice_inputs_keep_dense_ids_stable() {
    for (kind, id) in [
        (BindingKind::SetupLet, 0),
        (BindingKind::SetupMaybeRef, 1),
        (BindingKind::SetupRef, 2),
        (BindingKind::SetupReactiveConst, 3),
        (BindingKind::SetupConst, 4),
        (BindingKind::Props, 5),
        (BindingKind::PropsAliased, 6),
        (BindingKind::Data, 7),
        (BindingKind::Options, 8),
        (BindingKind::LiteralConst, 9),
        (BindingKind::JsGlobalUniversal, 10),
        (BindingKind::JsGlobalBrowser, 11),
        (BindingKind::JsGlobalNode, 12),
        (BindingKind::JsGlobalDeno, 13),
        (BindingKind::JsGlobalBun, 14),
        (BindingKind::VueGlobal, 15),
        (BindingKind::ExternalModule, 16),
    ] {
        assert_eq!(binding_kind_lattice_input(kind).id.index(), id);
    }
}

fn materialized_handler_facts(kind: BindingKind) -> PatchFacts {
    let allocator = vize_s0::Allocator::new();
    let arena = &allocator;
    let handler = "handler";
    let on = BindingOp::On(vize_s0::Box::new_in(
        OnOp {
            name: Some(vize_s2::op::DynamicName::Static("click")),
            modifiers: vize_s0::Vec::new_in(&arena),
            handler: Some(vize_s2::expr::ExprRef::parse_js_in(
                &allocator,
                handler,
                Span::new(0, handler.len() as u32),
            )),
            span: Span::new(0, handler.len() as u32),
        },
        &arena,
    ));
    binding_patch_facts(
        &[on],
        false,
        None,
        false,
        false,
        &|name| name == handler && handler_static_patch_binding(kind),
        &|_| false,
        false,
    )
}

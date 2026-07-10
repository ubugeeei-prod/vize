use crate::{VaporCompilerOptions, compile_vapor};
use vize_atelier_core::options::{BindingMetadata, BindingType};
use vize_carton::{Bump, FxHashMap};

#[test]
fn experimental_self_reference_passes_maybe_self_reference() {
    let allocator = Bump::new();
    let result = compile_vapor(
        &allocator,
        "<Self />",
        VaporCompilerOptions {
            experimental_self_reference: true,
            ..Default::default()
        },
    );

    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    assert!(
        result.code.contains(r#"_resolveComponent("Self", true)"#),
        "{}",
        result.code
    );
}

#[test]
fn experimental_self_reference_ignores_setup_binding() {
    let allocator = Bump::new();
    let mut bindings = FxHashMap::default();
    bindings.insert("Self".into(), BindingType::SetupConst);
    let result = compile_vapor(
        &allocator,
        "<Self />",
        VaporCompilerOptions {
            binding_metadata: Some(BindingMetadata {
                bindings,
                is_script_setup: true,
                ..Default::default()
            }),
            experimental_self_reference: true,
            ..Default::default()
        },
    );

    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    assert!(
        result.code.contains(r#"_resolveComponent("Self", true)"#),
        "{}",
        result.code
    );
    assert!(!result.code.contains("_ctx.Self"), "{}", result.code);
}

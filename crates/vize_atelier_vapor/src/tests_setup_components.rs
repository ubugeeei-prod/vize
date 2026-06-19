use super::{VaporCompilerOptions, compile_vapor};
use vize_atelier_core::options::{BindingMetadata, BindingType};
use vize_carton::{Bump, FxHashMap};

fn component_resolution_lines(code: &str) -> Vec<String> {
    code.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("const _component_"))
        .map(ToOwned::to_owned)
        .collect()
}

#[test]
fn test_compile_custom_renderer_intrinsics_with_bound_lowercase_component() {
    let allocator = Bump::new();
    let mut bindings = FxHashMap::default();
    bindings.insert("Primitive".into(), BindingType::SetupConst);
    let result = compile_vapor(
        &allocator,
        r#"<mesh><group v-if="visible"><primitive></primitive></group></mesh>"#,
        VaporCompilerOptions {
            custom_renderer: true,
            binding_metadata: Some(BindingMetadata {
                bindings,
                props_aliases: FxHashMap::default(),
                is_script_setup: true,
            }),
            ..Default::default()
        },
    );

    assert_eq!(result.error_messages.len(), 0);
    assert_eq!(
        component_resolution_lines(&result.code),
        vec!["const _component_primitive = _ctx.Primitive"]
    );
}

#[test]
fn test_setup_component_tag_binding_matrix_matches_vapor_behavior() {
    let allocator = Bump::new();
    let mut bindings = FxHashMap::default();
    bindings.insert("RefMenu".into(), BindingType::SetupRef);
    bindings.insert("MaybeMenu".into(), BindingType::SetupMaybeRef);
    bindings.insert("LetMenu".into(), BindingType::SetupLet);
    bindings.insert("ImportedMenu".into(), BindingType::SetupConst);
    bindings.insert("ShallowMenu".into(), BindingType::SetupRef);
    bindings.insert("lowercaseWidget".into(), BindingType::SetupConst);

    let result = compile_vapor(
        &allocator,
        r#"<RefMenu /><MaybeMenu /><LetMenu /><ImportedMenu /><ShallowMenu /><lowercase-widget /><RefMenu.Item />"#,
        VaporCompilerOptions {
            binding_metadata: Some(BindingMetadata {
                bindings,
                props_aliases: FxHashMap::default(),
                is_script_setup: true,
            }),
            ..Default::default()
        },
    );

    assert_eq!(result.error_messages.len(), 0);
    assert_eq!(
        component_resolution_lines(&result.code),
        vec![
            "const _component_RefMenu = _ctx.RefMenu",
            "const _component_MaybeMenu = _ctx.MaybeMenu",
            "const _component_LetMenu = _ctx.LetMenu",
            "const _component_ImportedMenu = _ctx.ImportedMenu",
            "const _component_ShallowMenu = _ctx.ShallowMenu",
            "const _component_lowercase_widget = _ctx.lowercaseWidget",
            "const _component_RefMenu_Item = _ctx.RefMenu.Item",
        ]
    );
}

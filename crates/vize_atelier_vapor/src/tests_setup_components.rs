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

// Regression tests for #3072: template references to destructured props must
// read the render signature's `$props` (aliased destructures through the
// original prop key), while v-for aliases keep shadowing prop names.
#[test]
fn test_props_bindings_resolve_through_dollar_props() {
    let allocator = Bump::new();
    let mut bindings = FxHashMap::default();
    bindings.insert("tag".into(), BindingType::Props);
    bindings.insert("theTag".into(), BindingType::PropsAliased);
    let mut props_aliases = FxHashMap::default();
    props_aliases.insert("theTag".into(), "kind".into());

    let result = compile_vapor(
        &allocator,
        r#"<span :id="tag" :class="theTag">{{ tag }} {{ theTag.toUpperCase() }}</span>"#,
        VaporCompilerOptions {
            binding_metadata: Some(BindingMetadata {
                bindings,
                props_aliases,
                is_script_setup: true,
            }),
            ..Default::default()
        },
    );

    assert_eq!(result.error_messages.len(), 0);
    insta::assert_snapshot!(result.code.as_str());
}

#[test]
fn test_for_alias_shadows_props_binding() {
    let allocator = Bump::new();
    let mut bindings = FxHashMap::default();
    bindings.insert("tag".into(), BindingType::Props);
    bindings.insert("tags".into(), BindingType::Props);

    let result = compile_vapor(
        &allocator,
        r#"<div v-for="tag in tags">{{ tag }}</div>"#,
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
    insta::assert_snapshot!(result.code.as_str());
}

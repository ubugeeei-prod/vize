use vize_atelier_core::options::{BindingMetadata, BindingType};
use vize_atelier_ssr::{SsrCompilerOptions, compile_ssr_with_options};
use vize_carton::{Bump, FxHashMap};

fn binding_matrix() -> BindingMetadata {
    let mut bindings = FxHashMap::default();
    bindings.insert("RefMenu".into(), BindingType::SetupRef);
    bindings.insert("MaybeMenu".into(), BindingType::SetupMaybeRef);
    bindings.insert("LetMenu".into(), BindingType::SetupLet);
    bindings.insert("ImportedMenu".into(), BindingType::SetupConst);
    bindings.insert("ShallowMenu".into(), BindingType::SetupRef);
    bindings.insert("lowercaseWidget".into(), BindingType::SetupConst);
    BindingMetadata {
        bindings,
        props_aliases: FxHashMap::default(),
        is_script_setup: true,
    }
}

fn ssr_component_targets(source: &str) -> Vec<String> {
    let marker = "_ssrRenderComponent(";
    let mut targets = Vec::new();
    let mut offset = 0;
    while let Some(index) = source[offset..].find(marker) {
        let start = offset + index + marker.len();
        let Some(target) = first_call_arg(&source[start..]) else {
            break;
        };
        targets.push(target);
        offset = start;
    }
    targets
}

fn first_call_arg(source: &str) -> Option<String> {
    let mut depth = 0i32;
    for (index, ch) in source.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' if depth == 0 => return Some(source[..index].trim().to_owned()),
            ')' => depth -= 1,
            ',' if depth == 0 => return Some(source[..index].trim().to_owned()),
            _ => {}
        }
    }
    None
}

#[test]
fn setup_component_tag_binding_matrix_matches_ssr_modes() {
    let allocator = Bump::new();
    let source = r#"<RefMenu /><MaybeMenu /><LetMenu /><ImportedMenu /><ShallowMenu /><lowercase-widget /><RefMenu.Item />"#;

    let (_, inline_errors, inline_result) = compile_ssr_with_options(
        &allocator,
        source,
        SsrCompilerOptions {
            inline: true,
            binding_metadata: Some(binding_matrix()),
            ..Default::default()
        },
    );
    assert_eq!(inline_errors.len(), 0);
    assert_eq!(
        ssr_component_targets(&inline_result.code),
        vec![
            "_unref(RefMenu)",
            "_unref(MaybeMenu)",
            "_unref(LetMenu)",
            "ImportedMenu",
            "_unref(ShallowMenu)",
            "lowercaseWidget",
            "_unref(RefMenu).Item",
        ]
    );

    let (_, function_errors, function_result) = compile_ssr_with_options(
        &allocator,
        source,
        SsrCompilerOptions {
            inline: false,
            binding_metadata: Some(binding_matrix()),
            ..Default::default()
        },
    );
    assert_eq!(function_errors.len(), 0);
    assert_eq!(
        ssr_component_targets(&function_result.code),
        vec![
            "$setup.RefMenu",
            "$setup.MaybeMenu",
            "$setup.LetMenu",
            "$setup.ImportedMenu",
            "$setup.ShallowMenu",
            "$setup.lowercaseWidget",
            "$setup.RefMenu.Item",
        ]
    );
}

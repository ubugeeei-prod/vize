use vize_atelier_core::options::{BindingMetadata, BindingType, CodegenMode};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_carton::{Bump, FxHashMap, String};

fn full_output(preamble: &str, code: &str) -> String {
    let mut full = String::with_capacity(preamble.len() + code.len() + 1);
    full.push_str(preamble);
    full.push('\n');
    full.push_str(code);
    full
}

#[test]
fn inline_setup_ref_component_tag_uses_unref() {
    let allocator = Bump::new();
    let mut bindings = FxHashMap::default();
    bindings.insert("Menu".into(), BindingType::SetupRef);
    bindings.insert("show".into(), BindingType::SetupRef);
    bindings.insert("items".into(), BindingType::SetupRef);

    let options = DomCompilerOptions {
        mode: CodegenMode::Module,
        prefix_identifiers: true,
        inline: true,
        binding_metadata: Some(BindingMetadata {
            bindings,
            props_aliases: FxHashMap::default(),
            is_script_setup: true,
        }),
        ..Default::default()
    };

    let (_, errors, result) = compile_template_with_options(
        &allocator,
        r#"<div><Menu>hello</Menu><Menu v-if="show" /><Menu v-for="item in items" :key="item" /><Menu v-once /><Menu.Item /></div>"#,
        options,
    );

    assert!(errors.is_empty(), "Errors: {:?}", errors);
    let full = full_output(&result.preamble, &result.code);
    assert_eq!(
        component_call_targets(&full),
        vec![
            "_unref(Menu)",
            "_unref(Menu)",
            "_unref(Menu).Item",
            "_unref(Menu)",
            "_unref(Menu)",
        ]
    );
}

fn component_call_targets(source: &str) -> Vec<String> {
    let mut targets = Vec::new();
    for marker in ["_createVNode(", "_createBlock("] {
        let mut offset = 0;
        while let Some(index) = source[offset..].find(marker) {
            let start = offset + index + marker.len();
            let Some(target) = first_call_arg(&source[start..]) else {
                break;
            };
            if !target.starts_with('"') {
                targets.push(target);
            }
            offset = start;
        }
    }
    targets
}

fn first_call_arg(source: &str) -> Option<String> {
    let mut depth = 0i32;
    for (index, ch) in source.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' if depth == 0 => return Some(source[..index].trim().into()),
            ')' => depth -= 1,
            ',' if depth == 0 => return Some(source[..index].trim().into()),
            _ => {}
        }
    }
    None
}

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

#[test]
fn setup_component_tag_binding_matrix_matches_dom_modes() {
    let allocator = Bump::new();
    let source = r#"<RefMenu /><MaybeMenu /><LetMenu /><ImportedMenu /><ShallowMenu /><lowercase-widget /><RefMenu.Item />"#;

    let inline_options = DomCompilerOptions {
        mode: CodegenMode::Module,
        prefix_identifiers: true,
        inline: true,
        binding_metadata: Some(binding_matrix()),
        ..Default::default()
    };
    let (_, inline_errors, inline_result) =
        compile_template_with_options(&allocator, source, inline_options);
    assert_eq!(inline_errors.len(), 0);
    let inline = full_output(&inline_result.preamble, &inline_result.code);
    assert_eq!(
        component_call_targets(&inline),
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

    let function_options = DomCompilerOptions {
        mode: CodegenMode::Function,
        prefix_identifiers: true,
        inline: false,
        binding_metadata: Some(binding_matrix()),
        ..Default::default()
    };
    let (_, function_errors, function_result) =
        compile_template_with_options(&allocator, source, function_options);
    assert_eq!(function_errors.len(), 0);
    let function = full_output(&function_result.preamble, &function_result.code);
    assert_eq!(
        component_call_targets(&function),
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

//! Vue compiler for DOM platform.
//!
//! This module provides DOM-specific compilation including:
//! - DOM element and attribute validation
//! - v-model transforms for form elements
//! - v-on event modifiers
//! - v-show transform
//! - Style and class binding handling

#![allow(clippy::collapsible_match)]

pub mod options;
pub mod transforms;

pub use options::{element_checks, event_modifiers, DomCompilerOptions};
pub use transforms::{
    generate_html_prop, generate_html_warning, generate_key_guard, generate_model_props,
    generate_modifier_guard, generate_show_directive, generate_show_style, generate_text_children,
    generate_text_content, get_model_event, get_model_helper, get_model_prop, is_v_html, is_v_show,
    is_v_text, resolve_key_alias, EventModifiers, EventOptions, MouseModifiers,
    PropagationModifiers, SystemModifiers, VModelModifiers, V_SHOW, V_TEXT,
};

// Re-export core types
pub use vize_atelier_core::{
    ast, codegen, errors, parser, runtime_helpers, tokenizer, transform, Allocator, CompilerError,
    Namespace, RootNode, TemplateChildNode,
};

use vize_atelier_core::codegen::CodegenResult;
use vize_atelier_core::{
    codegen::generate,
    options::{CodegenOptions, ParserOptions, TransformOptions},
    parser::parse_with_options,
    transform::transform as do_transform,
};
use vize_carton::{profile, Bump, String};
use vize_croquis::Croquis;

/// Compile a Vue template for DOM with default options
pub fn compile_template<'a>(
    allocator: &'a Bump,
    source: &'a str,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_with_options(allocator, source, DomCompilerOptions::default())
}

/// Compile a Vue template for DOM with custom options
pub fn compile_template_with_options<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: DomCompilerOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    // Create parser options with DOM-specific settings
    let parser_opts = ParserOptions {
        is_void_tag: vize_carton::is_void_tag,
        is_native_tag: Some(vize_carton::is_native_tag),
        is_pre_tag: |tag| tag == "pre",
        get_namespace,
        comments: options.comments,
        ..ParserOptions::default()
    };

    // Parse
    let (mut root, errors) = profile!(
        "atelier.dom.template.parse",
        parse_with_options(allocator, source, parser_opts)
    );

    if !errors.is_empty() {
        let codegen_result = CodegenResult {
            code: String::default(),
            preamble: String::default(),
            map: None,
        };
        return (root, errors.to_vec(), codegen_result);
    }

    // Transform with DOM-specific transforms
    // BindingMetadata is passed directly (no string conversion needed)
    let transform_opts = TransformOptions {
        prefix_identifiers: options.prefix_identifiers,
        hoist_static: options.hoist_static,
        cache_handlers: options.cache_handlers,
        scope_id: options.scope_id.clone(),
        ssr: options.ssr,
        is_ts: options.is_ts,
        inline: options.inline,
        binding_metadata: options.binding_metadata.clone(),
        ..Default::default()
    };
    // Allocate Croquis in the arena so it shares the allocator lifetime
    let analysis: Option<&Croquis> = options.croquis.map(|c| &*allocator.alloc(*c));
    profile!(
        "atelier.dom.template.transform",
        do_transform(allocator, &mut root, transform_opts, analysis)
    );

    // Codegen
    let codegen_opts = CodegenOptions {
        mode: options.mode,
        source_map: options.source_map,
        scope_id: options.scope_id.clone(),
        ssr: options.ssr,
        is_ts: options.is_ts,
        inline: options.inline,
        cache_handlers: options.cache_handlers,
        binding_metadata: options.binding_metadata,
        ..Default::default()
    };
    let codegen_result = profile!(
        "atelier.dom.template.codegen",
        generate(&root, codegen_opts)
    );

    (root, errors.to_vec(), codegen_result)
}

/// Get the namespace for an element based on its parent
fn get_namespace(tag: &str, parent: Option<&str>) -> Namespace {
    if vize_carton::is_svg_tag(tag) {
        return Namespace::Svg;
    }
    if vize_carton::is_math_ml_tag(tag) {
        return Namespace::MathMl;
    }

    // Inherit namespace from parent
    if let Some(parent_tag) = parent {
        if vize_carton::is_svg_tag(parent_tag) && tag != "foreignObject" {
            return Namespace::Svg;
        }
        if vize_carton::is_math_ml_tag(parent_tag)
            && tag != "annotation-xml"
            && tag != "foreignObject"
        {
            return Namespace::MathMl;
        }
    }

    Namespace::Html
}

#[cfg(test)]
mod tests {
    use super::{
        compile_template, compile_template_with_options, DomCompilerOptions, Namespace,
        TemplateChildNode,
    };
    use vize_atelier_core::options::CodegenMode;
    use vize_carton::Bump;

    fn full_output(preamble: &str, code: &str) -> vize_carton::String {
        let mut full = vize_carton::String::with_capacity(preamble.len() + code.len() + 1);
        full.push_str(preamble);
        full.push('\n');
        full.push_str(code);
        full
    }

    #[test]
    fn test_compile_simple_element() {
        let allocator = Bump::new();
        let (root, errors, result) = compile_template(&allocator, "<div>hello</div>");

        assert!(errors.is_empty());
        assert_eq!(root.children.len(), 1);
        let full = full_output(&result.preamble, &result.code);
        insta::assert_snapshot!(full.as_str());
    }

    #[test]
    fn test_compile_svg() {
        let allocator = Bump::new();
        let (root, errors, _) = compile_template(&allocator, "<svg><circle /></svg>");

        assert!(errors.is_empty());
        if let TemplateChildNode::Element(el) = &root.children[0] {
            assert_eq!(el.ns, Namespace::Svg);
        }
    }

    #[test]
    fn test_compile_with_options() {
        let allocator = Bump::new();
        let opts = DomCompilerOptions {
            mode: CodegenMode::Module,
            ..Default::default()
        };
        let (_, errors, result) = compile_template_with_options(&allocator, "<div></div>", opts);

        assert!(errors.is_empty());
        // Empty div generates minimal code
        assert!(!result.code.is_empty());
    }

    #[test]
    fn test_event_handler_setup_ref_value() {
        use vize_atelier_core::options::BindingType;
        use vize_carton::FxHashMap;

        let allocator = Bump::new();
        let mut bindings_map = FxHashMap::default();
        bindings_map.insert("quoteId".into(), BindingType::SetupRef);
        bindings_map.insert("renoteTargetNote".into(), BindingType::SetupRef);
        let binding_metadata = vize_atelier_core::options::BindingMetadata {
            bindings: bindings_map,
            props_aliases: FxHashMap::default(),
            is_script_setup: true,
        };

        let opts = DomCompilerOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: true,
            inline: true,
            cache_handlers: true,
            binding_metadata: Some(binding_metadata),
            ..Default::default()
        };
        let template = r#"<button @click="quoteId = null; renoteTargetNote = null;">x</button>"#;
        let (_, errors, result) = compile_template_with_options(&allocator, template, opts);

        eprintln!(
            "=== Template Output ===\npreamble:\n{}\ncode:\n{}",
            result.preamble, result.code
        );
        assert!(errors.is_empty(), "Errors: {:?}", errors);
        let full = full_output(&result.preamble, &result.code);
        insta::assert_snapshot!(full.as_str());
    }

    #[test]
    fn test_inline_ref_class_binding_keeps_class_patch_flag() {
        use vize_atelier_core::options::{BindingMetadata, BindingType};
        use vize_carton::FxHashMap;

        let allocator = Bump::new();
        let mut bindings = FxHashMap::default();
        bindings.insert("currentTab".into(), BindingType::SetupRef);

        let options = DomCompilerOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: true,
            inline: true,
            cache_handlers: true,
            binding_metadata: Some(BindingMetadata {
                bindings,
                props_aliases: FxHashMap::default(),
                is_script_setup: true,
            }),
            ..Default::default()
        };

        let (_, errors, result) = compile_template_with_options(
            &allocator,
            r#"<button :class="['tab', { active: currentTab === 'a' }]" @click="currentTab = 'b'">A</button>"#,
            options,
        );

        assert!(errors.is_empty(), "Errors: {:?}", errors);
        let full = full_output(&result.preamble, &result.code);
        insta::assert_snapshot!(full.as_str());
    }

    #[test]
    fn test_inline_component_dynamic_prop_keeps_props_patch_flag() {
        use vize_atelier_core::options::{BindingMetadata, BindingType};
        use vize_carton::FxHashMap;

        let allocator = Bump::new();
        let mut bindings = FxHashMap::default();
        bindings.insert("message".into(), BindingType::SetupRef);

        let options = DomCompilerOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: true,
            inline: true,
            cache_handlers: true,
            binding_metadata: Some(BindingMetadata {
                bindings,
                props_aliases: FxHashMap::default(),
                is_script_setup: true,
            }),
            ..Default::default()
        };

        let (_, errors, result) = compile_template_with_options(
            &allocator,
            r#"<div><MyComponent :msg="message" /></div>"#,
            options,
        );

        assert!(errors.is_empty(), "Errors: {:?}", errors);
        let full = full_output(&result.preamble, &result.code);
        insta::assert_snapshot!(full.as_str());
    }

    #[test]
    fn test_v_if_branch_component_dynamic_prop_keeps_props_patch_flag() {
        use vize_atelier_core::options::{BindingMetadata, BindingType};
        use vize_carton::FxHashMap;

        let allocator = Bump::new();
        let mut bindings = FxHashMap::default();
        bindings.insert("show".into(), BindingType::SetupRef);
        bindings.insert("message".into(), BindingType::SetupRef);

        let options = DomCompilerOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: true,
            inline: true,
            cache_handlers: true,
            binding_metadata: Some(BindingMetadata {
                bindings,
                props_aliases: FxHashMap::default(),
                is_script_setup: true,
            }),
            ..Default::default()
        };

        let (_, errors, result) = compile_template_with_options(
            &allocator,
            r#"<div><MyComponent v-if="show" :msg="message" /></div>"#,
            options,
        );

        assert!(errors.is_empty(), "Errors: {:?}", errors);
        let full = full_output(&result.preamble, &result.code);
        insta::assert_snapshot!(full.as_str());
    }
}

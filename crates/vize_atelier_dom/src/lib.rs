//! Vue compiler for DOM platform.
//!
//! This module provides DOM-specific compilation including:
//! - DOM element and attribute validation
//! - v-model transforms for form elements
//! - v-on event modifiers
//! - v-show transform
//! - Style and class binding handling

#![allow(clippy::collapsible_match)]
#![cfg_attr(
    test,
    allow(clippy::disallowed_macros, clippy::field_reassign_with_default)
)]

pub mod options;
pub mod transforms;

pub use options::{DomCompilerOptions, element_checks, event_modifiers};
pub use transforms::{
    EventModifiers, EventOptions, MouseModifiers, PropagationModifiers, SystemModifiers, V_SHOW,
    V_TEXT, VModelModifiers, generate_html_prop, generate_html_warning, generate_key_guard,
    generate_model_props, generate_modifier_guard, generate_show_directive, generate_show_style,
    generate_text_children, generate_text_content, get_model_event, get_model_helper,
    get_model_prop, is_v_html, is_v_show, is_v_text, resolve_key_alias,
};

// Re-export core types
pub use vize_atelier_core::{
    Allocator, CompilerError, Namespace, RootNode, TemplateChildNode, ast, codegen, errors, parser,
    runtime_helpers, tokenizer, transform,
};

use vize_atelier_core::codegen::CodegenResult;
use vize_atelier_core::{
    codegen::generate,
    options::{CodegenOptions, ParserOptions, TransformOptions},
    parser::parse_with_options_and_invalid_html_self_closing,
    transform::{
        transform as do_transform, transform_with_hoisted_scope_id,
        transform_with_vue_parser_quirks, transform_with_vue_parser_quirks_and_hoisted_scope_id,
    },
};
use vize_carton::{Bump, String, profile};
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
    compile_template_inner(allocator, source, options, false, None)
}

/// Compile a Vue template for DOM with Vue parser quirk compatibility.
pub fn compile_template_with_vue_parser_quirks<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: DomCompilerOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_inner(allocator, source, options, true, None)
}

/// Compile a Vue template for DOM with an explicit scope ID for hoisted static VNodes.
#[doc(hidden)]
pub fn compile_template_with_options_and_hoisted_scope_id<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: DomCompilerOptions,
    hoisted_scope_id: Option<String>,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_inner(allocator, source, options, false, hoisted_scope_id)
}

/// Compile a Vue template for DOM with Vue parser quirks and an explicit hoisted scope ID.
#[doc(hidden)]
pub fn compile_template_with_vue_parser_quirks_and_hoisted_scope_id<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: DomCompilerOptions,
    hoisted_scope_id: Option<String>,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_inner(allocator, source, options, true, hoisted_scope_id)
}

fn compile_template_inner<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: DomCompilerOptions,
    vue_parser_quirks: bool,
    hoisted_scope_id: Option<String>,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    // Create parser options with DOM-specific settings
    let parser_opts = ParserOptions {
        is_void_tag: vize_carton::is_void_tag,
        is_native_tag: Some(vize_carton::is_native_tag),
        custom_renderer: options.custom_renderer,
        is_pre_tag: |tag| tag == "pre",
        get_namespace,
        comments: options.comments,
        ..ParserOptions::default()
    };

    // Parse
    let (mut root, errors) = profile!(
        "atelier.dom.template.parse",
        parse_with_options_and_invalid_html_self_closing(
            allocator,
            source,
            parser_opts,
            vue_parser_quirks
        )
    );

    // Parser-level diagnostics that are recoverable (e.g. duplicate
    // attribute — Vue keeps the first and continues) must NOT gate
    // codegen, or downstream callers see a 0-byte module reported as a
    // success. (#958) The recoverable diagnostics still ride along in
    // the returned errors vec so the caller can surface them as
    // warnings or test for parity.
    let fatal_count = errors.iter().filter(|e| !e.is_recoverable()).count();
    if fatal_count > 0 {
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
        custom_renderer: options.custom_renderer,
        binding_metadata: options.binding_metadata.clone(),
        ..Default::default()
    };
    // Allocate Croquis in the arena so it shares the allocator lifetime
    let analysis: Option<&Croquis> = options.croquis.map(|c| &*allocator.alloc(*c));
    profile!(
        "atelier.dom.template.transform",
        if vue_parser_quirks {
            if hoisted_scope_id.is_some() {
                transform_with_vue_parser_quirks_and_hoisted_scope_id(
                    allocator,
                    &mut root,
                    transform_opts,
                    analysis,
                    hoisted_scope_id,
                )
            } else {
                transform_with_vue_parser_quirks(allocator, &mut root, transform_opts, analysis)
            }
        } else if hoisted_scope_id.is_some() {
            transform_with_hoisted_scope_id(
                allocator,
                &mut root,
                transform_opts,
                analysis,
                hoisted_scope_id,
            )
        } else {
            do_transform(allocator, &mut root, transform_opts, analysis)
        }
    );

    // Codegen
    let codegen_opts = CodegenOptions {
        mode: options.mode,
        source_map: options.source_map,
        component_name: options.component_name,
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

/// Get the namespace for an element based on its parent.
///
/// Mirrors the HTML tree-construction namespace rules used by `@vue/compiler-dom`: a tag
/// that names a foreign root (`<svg>`/`<math>`) always (re)enters that namespace, otherwise
/// the element inherits its parent's namespace — except across the HTML integration points
/// where SVG/MathML hand their descendants back to the HTML namespace.
fn get_namespace(tag: &str, parent: Option<&str>) -> Namespace {
    if vize_carton::is_svg_tag(tag) {
        return Namespace::Svg;
    }
    if vize_carton::is_math_ml_tag(tag) {
        return Namespace::MathMl;
    }

    // Inherit namespace from the parent, honouring the integration-point boundaries.
    if let Some(parent_tag) = parent {
        // Inside SVG, <foreignObject>/<desc>/<title> switch their descendants back to HTML
        // (e.g. a <div> inside <foreignObject> must NOT be in the SVG namespace).
        let svg_to_html = matches!(parent_tag, "foreignObject" | "desc" | "title");
        if vize_carton::is_svg_tag(parent_tag) && !svg_to_html {
            return Namespace::Svg;
        }
        // Inside MathML, <annotation-xml> and the text containers (<mi>/<mo>/<mn>/<ms>/
        // <mtext>) are HTML integration points; their descendants are HTML.
        let mathml_to_html = matches!(
            parent_tag,
            "annotation-xml" | "mi" | "mo" | "mn" | "ms" | "mtext"
        );
        if vize_carton::is_math_ml_tag(parent_tag) && !mathml_to_html {
            return Namespace::MathMl;
        }
    }

    Namespace::Html
}

#[cfg(test)]
mod tests {
    use super::{
        DomCompilerOptions, Namespace, TemplateChildNode, compile_template,
        compile_template_with_options, compile_template_with_vue_parser_quirks,
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

    /// A dynamic component whose `:is` is written as `v-bind:is` must not flush an empty
    /// `{}` segment into `mergeProps`: `:is` is consumed as the component tag, so the first
    /// real merge argument is the `v-bind="obj"` spread — matching @vue/compiler-dom's
    /// `_mergeProps(obj, { ... })` rather than `_mergeProps({}, obj, { ... })`.
    #[test]
    fn test_dynamic_component_vbind_is_no_empty_merge_object() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_template(
            &allocator,
            r#"<component :is="popup.component" v-bind="popup.props" :key="popup.id" @closed="onClose"/>"#,
        );
        assert!(errors.is_empty());
        let code = result.code.as_str();
        assert!(
            code.contains("_mergeProps(popup.props, {"),
            "merge should start with the spread, not an empty object:\n{code}"
        );
        assert!(
            !code.contains("_mergeProps({ }") && !code.contains("_mergeProps({  }"),
            "no empty object literal should be flushed into mergeProps:\n{code}"
        );
    }

    #[test]
    fn test_dynamic_component_v_if_does_not_emit_is_prop() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_template(
            &allocator,
            r#"<Component :is="current" v-if="ok" :foo="foo" />"#,
        );
        assert!(errors.is_empty());
        let code = result.code.as_str();
        assert!(
            code.contains("_resolveDynamicComponent(current)"),
            "dynamic component should be resolved from the :is binding:\n{code}"
        );
        assert!(
            !code.contains("is:"),
            "v-if dynamic component branch must not pass consumed :is as a prop:\n{code}"
        );
        assert!(
            !code.contains(r#""is""#),
            "v-if dynamic component branch must not track consumed :is as a dynamic prop:\n{code}"
        );
    }

    #[test]
    fn test_template_ref_in_v_for_emits_ref_for() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_template(
            &allocator,
            r#"<span v-for="item in items" ref="itemEls"></span>"#,
        );
        assert!(errors.is_empty());
        let code = result.code.as_str();
        assert!(
            code.contains("ref_for: true"),
            "template refs inside v-for must be marked as ref_for so Vue stores an array:\n{code}"
        );
    }

    #[test]
    fn test_static_ref_matching_prop_name_stays_string_ref() {
        use vize_atelier_core::options::{BindingMetadata, BindingType};
        use vize_carton::FxHashMap;

        let allocator = Bump::new();
        let mut bindings = FxHashMap::default();
        bindings.insert("buttons".into(), BindingType::Props);

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
            r#"<button v-for="button in buttons" ref="buttons" :key="button"></button>"#,
            options,
        );

        assert!(errors.is_empty(), "Errors: {:?}", errors);
        let code = result.code.as_str();
        assert!(code.contains("ref_for: true"), "{code}");
        assert!(code.contains(r#"ref: "buttons""#), "{code}");
        assert!(
            !code.contains("ref: buttons"),
            "props bindings must not be emitted as runtime ref identifiers:\n{code}"
        );
    }

    /// Recursively find the first element with the given tag, descending through `v-if`
    /// branches and `v-for` bodies so the search works on the transformed tree as well.
    fn find_element<'a, 'b>(
        children: &'b [TemplateChildNode<'a>],
        tag: &str,
    ) -> Option<&'b super::ast::ElementNode<'a>> {
        for child in children {
            match child {
                TemplateChildNode::Element(el) => {
                    if el.tag.as_str() == tag {
                        return Some(el);
                    }
                    if let Some(found) = find_element(&el.children, tag) {
                        return Some(found);
                    }
                }
                TemplateChildNode::If(node) => {
                    for branch in &node.branches {
                        if let Some(found) = find_element(&branch.children, tag) {
                            return Some(found);
                        }
                    }
                }
                TemplateChildNode::IfBranch(branch) => {
                    if let Some(found) = find_element(&branch.children, tag) {
                        return Some(found);
                    }
                }
                TemplateChildNode::For(node) => {
                    if let Some(found) = find_element(&node.children, tag) {
                        return Some(found);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// #992: the SVG namespace must propagate from `<svg>` into every descendant so the
    /// runtime mounts them with `setAttributeNS`/the SVG namespace URI. This locks the
    /// parser-side propagation that codegen relies on (vize, like @vue/compiler-sfc, emits
    /// no namespace argument and depends on the runtime inferring it from a contiguous tree).
    #[test]
    fn test_svg_namespace_propagates_to_descendants() {
        let allocator = Bump::new();
        let (root, errors, _) = compile_template(
            &allocator,
            "<svg><g><path d=\"M0 0\"/></g><rect x=\"0\" y=\"0\"/></svg>",
        );
        assert!(errors.is_empty());

        let svg = find_element(&root.children, "svg").expect("svg present");
        assert_eq!(svg.ns, Namespace::Svg);
        assert_eq!(
            find_element(&root.children, "g").unwrap().ns,
            Namespace::Svg,
            "direct child <g> inherits svg namespace"
        );
        assert_eq!(
            find_element(&root.children, "path").unwrap().ns,
            Namespace::Svg,
            "nested <path> keeps svg namespace"
        );
        assert_eq!(
            find_element(&root.children, "rect").unwrap().ns,
            Namespace::Svg,
            "sibling <rect> inherits svg namespace"
        );
    }

    /// #992: `<foreignObject>` is the one boundary where the SVG namespace must NOT
    /// propagate further — its HTML descendants go back to the HTML namespace, while a
    /// `<rect>` sibling after the `<foreignObject>` stays in the SVG namespace.
    #[test]
    fn test_svg_foreign_object_resets_namespace() {
        let allocator = Bump::new();
        let (root, errors, _) = compile_template(
            &allocator,
            "<svg><foreignObject><div>hi</div></foreignObject><rect x=\"1\" y=\"1\"/></svg>",
        );
        assert!(errors.is_empty());

        assert_eq!(
            find_element(&root.children, "foreignObject").unwrap().ns,
            Namespace::Svg,
            "<foreignObject> itself is in the svg namespace"
        );
        assert_eq!(
            find_element(&root.children, "div").unwrap().ns,
            Namespace::Html,
            "<div> inside <foreignObject> returns to the HTML namespace"
        );
        assert_eq!(
            find_element(&root.children, "rect").unwrap().ns,
            Namespace::Svg,
            "<rect> after <foreignObject> is still in the svg namespace"
        );
    }

    /// #992: namespace propagation must survive the codegen shapes that could otherwise
    /// detach a child from its `<svg>` ancestor in the vnode tree — a `v-if` branch
    /// (re-entered via `createElementBlock`) keeps the `<rect>` nested inside the `<svg>`
    /// element call, so the runtime still threads the svg namespace into it.
    #[test]
    fn test_svg_namespace_with_v_if_branch() {
        let allocator = Bump::new();
        let (root, errors, _) = compile_template(
            &allocator,
            "<svg><rect v-if=\"show\" x=\"0\" y=\"0\"/></svg>",
        );
        assert!(errors.is_empty());
        assert_eq!(
            find_element(&root.children, "rect").unwrap().ns,
            Namespace::Svg,
            "v-if <rect> still carries the svg namespace"
        );
    }

    /// #992: lock the emitted shape for an SVG tree with a `<foreignObject>` exit. The SVG
    /// children must stay nested inside the `<svg>` `createElementVNode`/`createElementBlock`
    /// call so the runtime threads the SVG namespace into them at patch time (vize, like
    /// @vue/compiler-sfc, emits no explicit namespace argument).
    #[test]
    fn test_svg_codegen_shape_keeps_children_nested() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_template(
            &allocator,
            "<svg><foreignObject><div>hi</div></foreignObject><rect x=\"1\" y=\"1\"/></svg>",
        );
        assert!(errors.is_empty());
        let full = full_output(&result.preamble, &result.code);
        insta::assert_snapshot!(full.as_str());
    }

    #[test]
    fn test_inline_svg_dynamic_subtree_uses_own_block() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_template(
            &allocator,
            "<div><svg :width=\"w\"><g v-if=\"ok\"><rect :x=\"x\"/></g></svg></div>",
        );
        assert!(errors.is_empty());

        let code = result.code.as_str();
        assert!(
            code.contains(r#"_createElementBlock("svg""#),
            "inline <svg> must be a block so dynamic descendants patch with SVG namespace:\n{code}"
        );
        assert!(
            code.contains(r#"_createElementBlock("g""#),
            "dynamic SVG branch should keep its own block under the SVG namespace:\n{code}"
        );
    }

    #[test]
    fn test_inline_svg_descendants_inside_same_namespace_stay_vnodes() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_template(
            &allocator,
            r#"<div><svg><defs><pattern :x="0"><line :x1="w"/></pattern></defs></svg></div>"#,
        );
        assert!(errors.is_empty());

        let code = result.code.as_str();
        assert!(
            code.contains(r#"_createElementBlock("svg""#),
            "inline <svg> must still enter the SVG namespace with a block:\n{code}"
        );
        for tag in ["defs", "pattern", "line"] {
            assert!(
                code.contains(&format!(r#"_createElementVNode("{tag}""#)),
                "SVG descendants inside the same namespace should be VNodes:\n{code}"
            );
            assert!(
                !code.contains(&format!(r#"_createElementBlock("{tag}""#)),
                "SVG descendants inside the same namespace should not be blocks:\n{code}"
            );
        }
    }

    #[test]
    fn test_svg_foreign_object_namespace_exit_uses_boundary_block() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_template(
            &allocator,
            r#"<svg><foreignObject><div :id="id">hi</div></foreignObject></svg>"#,
        );
        assert!(errors.is_empty());

        let code = result.code.as_str();
        assert!(
            code.contains(r#"_createElementBlock("foreignObject""#),
            "<foreignObject> must keep its own block when descendants leave SVG namespace:\n{code}"
        );
        assert!(
            code.contains(r#"_createElementVNode("div""#),
            "HTML descendants after the namespace exit should remain VNodes:\n{code}"
        );
    }

    #[test]
    fn test_nested_svg_with_v_bind_uses_own_block() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_template(
            &allocator,
            r#"<div><svg xmlns="http://www.w3.org/2000/svg" :width="w" /></div>"#,
        );
        assert!(errors.is_empty());

        let code = result.code.as_str();
        assert!(
            code.contains(r#"_createElementBlock("svg""#),
            "nested SVG elements with dynamic props must render as blocks:\n{code}"
        );
        assert!(
            !code.contains(r#"_createElementVNode("svg""#),
            "nested SVG elements with dynamic props must not render as plain VNodes:\n{code}"
        );
    }

    #[test]
    fn test_svg_constant_bound_children_are_cached_vnodes() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_template(
            &allocator,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><rect :x="1" /><rect x="1" /></svg>"#,
        );
        assert!(errors.is_empty());

        let code = result.code.as_str();
        assert!(
            code.contains("[...(_cache[0]"),
            "static SVG children should be cached together:\n{}\n{code}",
            result.preamble
        );
        assert!(
            code.contains(r#"_createElementVNode("rect", { x: 1 }, null, -1 /* CACHED */)"#),
            "constant v-bind SVG child should compile as a cached VNode:\n{}\n{code}",
            result.preamble
        );
        assert!(
            !code.contains(r#"_createElementBlock("rect""#),
            "constant SVG children must not become block roots:\n{}\n{code}",
            result.preamble
        );
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
    fn test_compile_v_for_vue_parser_quirks_accepts_unmatched_alias_paren() {
        let allocator = Bump::new();
        let opts = DomCompilerOptions::default();
        let (_, errors, result) = compile_template_with_vue_parser_quirks(
            &allocator,
            r#"<div v-for="item) in items">{{ item }}</div>"#,
            opts,
        );

        assert!(errors.is_empty(), "Errors: {:?}", errors);
        assert!(result.code.contains("_renderList(items, (item) =>"));
    }

    #[test]
    fn test_compile_vue_parser_quirks_accepts_invalid_html_self_closing() {
        let allocator = Bump::new();
        let (_, errors, result) = compile_template_with_vue_parser_quirks(
            &allocator,
            "<div /><span></span>",
            DomCompilerOptions::default(),
        );

        assert!(errors.is_empty(), "Errors: {:?}", errors);
        assert!(!result.code.is_empty());
        assert!(result.code.contains(r#"_createElementVNode("div""#));
        assert!(result.code.contains(r#"_createElementVNode("span""#));
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
    fn test_ref_scroll_keeps_need_patch_with_need_hydration() {
        use vize_atelier_core::options::{BindingMetadata, BindingType};
        use vize_carton::FxHashMap;

        let allocator = Bump::new();
        let mut bindings = FxHashMap::default();
        bindings.insert("onScroll".into(), BindingType::SetupConst);

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
            r#"<div ref="container" @scroll="onScroll"></div>"#,
            options,
        );

        assert!(errors.is_empty(), "Errors: {:?}", errors);
        let full = full_output(&result.preamble, &result.code);
        assert!(
            full.contains("544 /* NEED_HYDRATION, NEED_PATCH */"),
            "{full}"
        );
    }

    #[test]
    fn test_ref_text_keeps_need_patch_with_text_flag() {
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
            r#"<div ref="container">{{ message }}</div>"#,
            options,
        );

        assert!(errors.is_empty(), "Errors: {:?}", errors);
        let full = full_output(&result.preamble, &result.code);
        assert!(full.contains("513 /* TEXT, NEED_PATCH */"), "{full}");
    }

    #[test]
    fn test_inline_hoisted_bare_static_attrs_are_empty_strings() {
        let allocator = Bump::new();
        let options = DomCompilerOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: true,
            inline: true,
            ..Default::default()
        };

        let (_, errors, result) = compile_template_with_options(
            &allocator,
            r#"<section><h2 sr-only font-bold flex="~ gap-1"><span block></span></h2></section>"#,
            options,
        );

        assert!(errors.is_empty(), "Errors: {:?}", errors);
        let full = full_output(&result.preamble, &result.code);
        assert!(full.contains(r#""sr-only": """#), "{full}");
        assert!(full.contains(r#""font-bold": """#), "{full}");
        assert!(full.contains(r#"block: """#), "{full}");
        assert!(!full.contains(r#""sr-only": "true""#), "{full}");
        assert!(!full.contains(r#""font-bold": "true""#), "{full}");
        assert!(!full.contains(r#"block: "true""#), "{full}");
    }

    #[test]
    fn test_inline_component_dynamic_prop_keeps_props_patch_flag() {
        use vize_atelier_core::options::{BindingMetadata, BindingType};
        use vize_carton::FxHashMap;

        let allocator = Bump::new();
        let mut bindings = FxHashMap::default();
        bindings.insert("message".into(), BindingType::SetupRef);
        bindings.insert("activeClass".into(), BindingType::SetupRef);

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
            r#"<div><MyComponent :msg="message" :class="activeClass" :full="true" /></div>"#,
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

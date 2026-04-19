//! Root-level code generation utilities.
//!
//! Generates the function signature, preamble (import/destructuring statements),
//! asset resolution (components and directives), and root text filtering.

use crate::ast::{RootNode, RuntimeHelper, TemplateChildNode};

use super::context::CodegenContext;
use super::element::helpers::is_dynamic_component_tag;
use vize_carton::{camelize, capitalize, String};

/// Check if a root-level text node is ignorable whitespace.
pub(super) fn is_ignorable_root_text(child: &TemplateChildNode<'_>) -> bool {
    matches!(child, TemplateChildNode::Text(text) if text.content.chars().all(|c| c.is_whitespace()))
}

fn imported_directive_binding_name(name: &str) -> String {
    let camel = camelize(name);
    let pascal = capitalize(&camel);
    let mut binding = String::with_capacity(1 + pascal.len());
    binding.push('v');
    binding.push_str(&pascal);
    binding
}

/// Generate preamble from a list of helpers.
pub(super) fn generate_preamble_from_helpers(
    ctx: &CodegenContext,
    helpers: &[RuntimeHelper],
) -> String {
    if helpers.is_empty() {
        return String::default();
    }

    // Pre-calculate capacity: each helper needs ~20 chars on average
    let estimated_capacity = 32 + helpers.len() * 24;
    let mut preamble = Vec::with_capacity(estimated_capacity);

    match ctx.options.mode {
        crate::options::CodegenMode::Module => {
            // ES module imports - build string directly without intermediate Vec
            preamble.extend_from_slice(b"import { ");
            for (i, h) in helpers.iter().enumerate() {
                if i > 0 {
                    preamble.extend_from_slice(b", ");
                }
                preamble.extend_from_slice(h.name().as_bytes());
                preamble.extend_from_slice(b" as ");
                preamble.extend_from_slice(ctx.helper(*h).as_bytes());
            }
            preamble.extend_from_slice(b" } from \"");
            preamble.extend_from_slice(ctx.runtime_module_name.as_bytes());
            preamble.extend_from_slice(b"\"\n");
        }
        crate::options::CodegenMode::Function => {
            // Destructuring from global - build string directly without intermediate Vec
            preamble.extend_from_slice(b"const { ");
            for (i, h) in helpers.iter().enumerate() {
                if i > 0 {
                    preamble.extend_from_slice(b", ");
                }
                preamble.extend_from_slice(h.name().as_bytes());
                preamble.extend_from_slice(b": ");
                preamble.extend_from_slice(ctx.helper(*h).as_bytes());
            }
            preamble.extend_from_slice(b" } = ");
            preamble.extend_from_slice(ctx.runtime_global_name.as_bytes());
            preamble.push(b'\n');
        }
    }

    // SAFETY: We only push valid UTF-8 strings
    unsafe { String::from_utf8_unchecked(preamble) }
}

/// Generate function signature.
pub(super) fn generate_function_signature(ctx: &mut CodegenContext) {
    if ctx.options.ssr {
        ctx.push("function ssrRender(_ctx, _push, _parent, _attrs) {");
    } else {
        match ctx.options.mode {
            crate::options::CodegenMode::Module => {
                // Module mode: include $props and $setup when binding_metadata is present
                // This is needed when script setup is used with non-inline template
                if ctx.options.binding_metadata.is_some() {
                    ctx.push(
                        "export function render(_ctx, _cache, $props, $setup, $data, $options) {",
                    );
                } else {
                    ctx.push("export function render(_ctx, _cache) {");
                }
            }
            crate::options::CodegenMode::Function => {
                // Function mode: include $props and $setup
                ctx.push("function render(_ctx, _cache, $props, $setup, $data, $options) {");
            }
        }
    }
}

/// Generate asset resolution (components, directives).
pub(super) fn generate_assets(ctx: &mut CodegenContext, root: &RootNode<'_>) {
    let mut has_resolved_assets = false;

    // Resolve components (only those not in binding metadata)
    for component in root.components.iter() {
        // Skip components that are in binding metadata (from script setup imports)
        if ctx.is_component_in_bindings(component) {
            continue;
        }

        // Skip built-in components - they are imported directly, not resolved
        if super::helpers::is_builtin_component(component).is_some() {
            continue;
        }

        // Skip dynamic component (<component :is="...">) -
        // it uses resolveDynamicComponent
        if is_dynamic_component_tag(component) {
            continue;
        }

        ctx.use_helper(RuntimeHelper::ResolveComponent);
        ctx.push("const _component_");
        ctx.push(&component.replace('-', "_"));
        ctx.push(" = ");
        ctx.push(ctx.helper(RuntimeHelper::ResolveComponent));
        ctx.push("(\"");
        ctx.push(component);
        ctx.push("\")");
        ctx.newline();
        has_resolved_assets = true;
    }

    // Resolve directives
    for directive in root.directives.iter() {
        ctx.push("const _directive_");
        ctx.push(&directive.replace('-', "_"));
        ctx.push(" = ");
        let binding_name = imported_directive_binding_name(directive);
        let uses_imported_binding = ctx
            .options
            .binding_metadata
            .as_ref()
            .is_some_and(|m| m.bindings.contains_key(binding_name.as_str()));
        if uses_imported_binding {
            ctx.push(&binding_name);
        } else {
            ctx.use_helper(RuntimeHelper::ResolveDirective);
            ctx.push(ctx.helper(RuntimeHelper::ResolveDirective));
            ctx.push("(\"");
            ctx.push(directive);
            ctx.push("\")");
        }
        ctx.newline();
        has_resolved_assets = true;
    }

    if has_resolved_assets {
        ctx.newline();
    }
}

//! Template compilation for Vue SFCs.
//!
//! This module handles compilation of `<template>` blocks,
//! supporting both DOM mode and Vapor mode.

use vize_carton::{String, ToCompactString, profile};
mod extraction;
mod string_tracking;
mod vapor;

#[cfg(test)]
mod tests;

pub(crate) use extraction::{extract_template_parts, extract_template_parts_full};
pub(crate) use vapor::compile_template_block_vapor;

use vize_carton::Bump;

use crate::types::{BindingMetadata, SfcError, SfcTemplateBlock, TemplateCompileOptions};

pub(crate) struct TemplateBlockCompileContext<'a> {
    pub(crate) scope_id: &'a str,
    pub(crate) apply_scope_id: bool,
    /// Whether the component has any `<style scoped>` block. When true, hoisted
    /// module-level static vnodes must carry the `data-v-*` attribute so scoped
    /// CSS selectors continue to match them in client builds.
    pub(crate) has_scoped: bool,
    pub(crate) is_ts: bool,
    pub(crate) inline: bool,
    pub(crate) component_name: Option<&'a str>,
    pub(crate) bindings: Option<&'a BindingMetadata>,
    pub(crate) croquis: Option<vize_croquis::analysis::Croquis>,
}

/// Compile template block
pub(crate) fn compile_template_block(
    template: &SfcTemplateBlock,
    options: &TemplateCompileOptions,
    ctx: TemplateBlockCompileContext<'_>,
    vue_parser_quirks: bool,
) -> Result<String, SfcError> {
    let TemplateBlockCompileContext {
        scope_id,
        apply_scope_id,
        has_scoped,
        is_ts,
        inline,
        component_name,
        bindings,
        croquis,
    } = ctx;
    let allocator = Bump::new();
    let scope_attr = if apply_scope_id {
        let mut attr = String::with_capacity(scope_id.len() + 7);
        attr.push_str("data-v-");
        attr.push_str(scope_id);
        Some(attr)
    } else {
        None
    };

    if options.ssr {
        let ssr_opts = vize_atelier_ssr::SsrCompilerOptions {
            scope_id: scope_attr,
            component_name: component_name.map(|name| name.to_compact_string()),
            comments: options
                .compiler_options
                .as_ref()
                .is_some_and(|opts| opts.comments),
            inline: false,
            is_ts,
            custom_renderer: options.custom_renderer,
            ssr_css_vars: options.ssr_css_vars.clone(),
            binding_metadata: bindings.cloned(),
            croquis: croquis.map(Box::new),
        };

        let (_, errors, result) = profile!(
            "atelier.sfc.template.ssr",
            if vue_parser_quirks {
                vize_atelier_ssr::compile_ssr_with_vue_parser_quirks(
                    &allocator,
                    &template.content,
                    ssr_opts,
                )
            } else {
                vize_atelier_ssr::compile_ssr_with_options(&allocator, &template.content, ssr_opts)
            }
        );

        // Recoverable parser diagnostics (e.g. duplicate attribute) must
        // not gate SFC compilation, or a single `<div id=a id=b>` produces
        // a 0-byte module marked as success. (#958)
        let fatal: Vec<_> = errors.iter().filter(|e| !e.is_recoverable()).collect();
        if !fatal.is_empty() {
            let mut message = String::from("Template compilation errors: ");
            use std::fmt::Write as _;
            let _ = write!(&mut message, "{:?}", fatal);
            return Err(SfcError {
                message,
                code: Some("TEMPLATE_ERROR".to_compact_string()),
                loc: Some(template.loc.clone()),
            });
        }

        let mut output = String::default();
        output.push_str(&result.preamble);
        output.push('\n');
        output.push_str(&result.code);
        output.push('\n');
        return Ok(output);
    }

    // Build DOM compiler options
    let mut dom_opts = options.compiler_options.clone().unwrap_or_default();
    dom_opts.mode = vize_atelier_core::options::CodegenMode::Module;
    dom_opts.prefix_identifiers = true;
    dom_opts.scope_id = scope_attr;
    // Hoisted module-level static vnodes are created at import time, when the
    // runtime's `currentScopeId` is null, so the runtime cannot stamp the
    // scoped-CSS attribute on them. Bake `data-v-*` directly into their props
    // here whenever the component owns a scoped style block.
    let hoisted_scope_attr = if has_scoped {
        let mut attr = String::with_capacity(scope_id.len() + 7);
        attr.push_str("data-v-");
        attr.push_str(scope_id);
        Some(attr)
    } else {
        None
    };
    dom_opts.ssr = options.ssr;
    dom_opts.is_ts = is_ts;
    dom_opts.custom_renderer = options.custom_renderer;
    dom_opts.component_name = component_name.map(|name| name.to_compact_string());

    // For script setup, use inline mode to match Vue's actual compiler behavior
    // Inline mode generates direct closure references (e.g., msg instead of $setup.msg)
    // which are captured in the setup() function scope
    if inline && bindings.is_some() {
        dom_opts.inline = true;
        dom_opts.hoist_static = true;
        dom_opts.cache_handlers = true;
    }

    // Pass binding metadata directly (no string conversion needed)
    dom_opts.binding_metadata = bindings.cloned();

    // Pass Croquis to DOM compiler for enhanced transforms
    if let Some(c) = croquis {
        dom_opts.croquis = Some(Box::new(c));
    }

    // Compile template
    let (_, errors, result) = profile!(
        "atelier.sfc.template.dom",
        if vue_parser_quirks {
            vize_atelier_dom::compile_template_with_vue_parser_quirks_and_hoisted_scope_id(
                &allocator,
                &template.content,
                dom_opts,
                hoisted_scope_attr,
            )
        } else {
            vize_atelier_dom::compile_template_with_options_and_hoisted_scope_id(
                &allocator,
                &template.content,
                dom_opts,
                hoisted_scope_attr,
            )
        }
    );

    // See above — drop recoverable parser diagnostics from the gating
    // check so duplicate-attribute SFCs still produce valid render code. (#958)
    let fatal: Vec<_> = errors.iter().filter(|e| !e.is_recoverable()).collect();
    if !fatal.is_empty() {
        let mut message = String::from("Template compilation errors: ");
        use std::fmt::Write as _;
        let _ = write!(&mut message, "{:?}", fatal);
        return Err(SfcError {
            message,
            code: Some("TEMPLATE_ERROR".to_compact_string()),
            loc: Some(template.loc.clone()),
        });
    }

    // Generate render function with proper imports
    let mut output = String::default();

    // Add Vue imports
    output.push_str(&result.preamble);
    output.push('\n');

    // The codegen already generates a complete function with closing brace,
    // so we just need to use it directly
    output.push_str(&result.code);
    output.push('\n');

    Ok(output)
}

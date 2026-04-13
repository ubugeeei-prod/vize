//! Template compilation for Vue SFCs.
//!
//! This module handles compilation of `<template>` blocks,
//! supporting both DOM mode and Vapor mode.

use vize_carton::{profile, String, ToCompactString};
mod extraction;
mod string_tracking;
mod vapor;

#[cfg(test)]
mod tests;

pub(crate) use extraction::{extract_template_parts, extract_template_parts_full};
pub(crate) use vapor::compile_template_block_vapor;

use vize_carton::Bump;

use crate::types::{BindingMetadata, SfcError, SfcTemplateBlock, TemplateCompileOptions};

/// Compile template block
pub(crate) fn compile_template_block(
    template: &SfcTemplateBlock,
    options: &TemplateCompileOptions,
    scope_id: &str,
    has_scoped: bool,
    is_ts: bool,
    bindings: Option<&BindingMetadata>,
    croquis: Option<vize_croquis::analysis::Croquis>,
) -> Result<String, SfcError> {
    let allocator = Bump::new();
    let scope_attr = if has_scoped {
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
            comments: options
                .compiler_options
                .as_ref()
                .is_some_and(|opts| opts.comments),
            inline: false,
            is_ts,
            ssr_css_vars: options.ssr_css_vars.clone(),
            binding_metadata: bindings.cloned(),
            croquis: croquis.map(Box::new),
        };

        let (_, errors, result) = profile!(
            "atelier.sfc.template.ssr",
            vize_atelier_ssr::compile_ssr_with_options(&allocator, &template.content, ssr_opts)
        );

        if !errors.is_empty() {
            let mut message = String::from("Template compilation errors: ");
            use std::fmt::Write as _;
            let _ = write!(&mut message, "{:?}", errors);
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
    dom_opts.ssr = options.ssr;
    dom_opts.is_ts = is_ts;

    // For script setup, use inline mode to match Vue's actual compiler behavior
    // Inline mode generates direct closure references (e.g., msg instead of $setup.msg)
    // which are captured in the setup() function scope
    if bindings.is_some() {
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
        vize_atelier_dom::compile_template_with_options(&allocator, &template.content, dom_opts)
    );

    if !errors.is_empty() {
        let mut message = String::from("Template compilation errors: ");
        use std::fmt::Write as _;
        let _ = write!(&mut message, "{:?}", errors);
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

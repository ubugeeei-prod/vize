//! Vapor mode template compilation.

use super::string_tracking::{StringTrackState, count_braces_with_state};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_vapor::{
    VaporCompilerExperimentalOptions, VaporCompilerOptions, compile_vapor_with_sfc_context,
};
use vize_s0::{Allocator, String, ToCompactString};

use crate::{
    compile_template::{
        TemplateBlockCompileContext, TemplateBlockCompileResult, recoverable_template_warnings,
    },
    types::{BindingMetadata, SfcError, SfcTemplateBlock, TemplateCompileOptions},
};

/// Compile template block using Vapor mode
pub(crate) fn compile_template_block_vapor(
    allocator: &Allocator,
    template: &SfcTemplateBlock,
    options: &TemplateCompileOptions,
    custom_elements: &CustomElementMatcher,
    ctx: TemplateBlockCompileContext<'_>,
    template_syntax: TemplateSyntaxMode,
    codegen_options: &CodegenOptions,
) -> Result<TemplateBlockCompileResult, SfcError> {
    let compiler_options = options.compiler_options.as_ref();
    let TemplateBlockCompileContext {
        scope_id,
        has_scoped,
        bindings,
        component_name,
        experimental_self_component,
        ..
    } = ctx;

    // Build Vapor compiler options
    let vapor_opts = VaporCompilerOptions {
        prefix_identifiers: false,
        ssr: false,
        binding_metadata: bindings.cloned(),
        custom_renderer: options.custom_renderer,
        experimental_in_tag_comments: compiler_options
            .is_some_and(|opts| opts.experimental_in_tag_comments),
        experimental_patterned_template: compiler_options
            .is_some_and(|opts| opts.experimental_patterned_template),
        ..Default::default()
    };
    let experimental_options = VaporCompilerExperimentalOptions {
        component_name: component_name.map(|name| name.to_compact_string()),
        self_component: experimental_self_component,
        ..VaporCompilerExperimentalOptions::default()
    };

    let scope_attr = has_scoped.then(|| vize_s0::cstr!("data-v-{scope_id}"));

    // Compile template with Vapor
    let (result, diagnostics) = compile_vapor_with_sfc_context(
        allocator,
        &template.content,
        vapor_opts,
        template_syntax,
        custom_elements.clone(),
        experimental_options,
        scope_attr.as_deref(),
    );

    if !result.error_messages.is_empty() {
        let mut message = String::from("Vapor template compilation errors: ");
        use std::fmt::Write as _;
        let _ = write!(&mut message, "{:?}", result.error_messages);
        return Err(SfcError {
            message,
            code: Some("VAPOR_TEMPLATE_ERROR".to_compact_string()),
            loc: Some(template.loc.clone()),
        });
    }

    // Process the Vapor output to extract imports and render function
    let code = transform_vapor_template_output(
        &result.code,
        template,
        bindings,
        codegen_options.runtime_module_name.as_str(),
    )?;

    Ok(TemplateBlockCompileResult {
        code,
        warnings: recoverable_template_warnings(&diagnostics),
        sections: None,
    })
}

fn rewrite_vapor_import(line: &str, runtime_module_name: &str) -> String {
    let (source, replacement) = if line.contains("'vue/vapor'") {
        ("'vue/vapor'", vize_s0::cstr!("'{runtime_module_name}'"))
    } else if line.contains("\"vue/vapor\"") {
        ("\"vue/vapor\"", vize_s0::cstr!("\"{runtime_module_name}\""))
    } else if line.contains("'vue'") {
        ("'vue'", vize_s0::cstr!("'{runtime_module_name}'"))
    } else if line.contains("\"vue\"") {
        ("\"vue\"", vize_s0::cstr!("\"{runtime_module_name}\""))
    } else {
        return line.to_compact_string();
    };
    line.replace(source, replacement.as_str()).into()
}

fn is_render_signature(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("export function render(")
        || trimmed.starts_with("function render(")
        || trimmed.starts_with("export default")
}

pub(super) fn transform_vapor_template_output(
    code: &str,
    template: &SfcTemplateBlock,
    bindings: Option<&BindingMetadata>,
    runtime_module_name: &str,
) -> Result<String, SfcError> {
    let mut lines = code.lines().peekable();
    let mut output = String::default();

    while let Some(&line) = lines.peek() {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") {
            output.push_str(&rewrite_vapor_import(line, runtime_module_name));
            output.push('\n');
            lines.next();
            continue;
        }
        if trimmed.is_empty() {
            lines.next();
            continue;
        }
        break;
    }

    let mut render_line = None;
    for line in lines.by_ref() {
        if is_render_signature(line.trim()) {
            render_line = Some(line);
            break;
        }

        output.push_str(line);
        output.push('\n');
    }

    let Some(render_line) = render_line else {
        return Err(SfcError {
            message: "Vapor template output is missing a render function".to_compact_string(),
            code: Some("VAPOR_TEMPLATE_ERROR".to_compact_string()),
            loc: Some(template.loc.clone()),
        });
    };

    output.push_str("function render(_ctx, $props, $emit, $attrs, $slots) {\n");

    let mut brace_state = StringTrackState::default();
    let mut brace_depth = count_braces_with_state(render_line, &mut brace_state);

    for line in lines {
        if brace_depth <= 0 {
            break;
        }
        let next_depth = brace_depth + count_braces_with_state(line, &mut brace_state);
        if !(next_depth == 0 && line.trim() == "}") {
            if let Some(rewritten) = rewrite_bound_component_resolution(line, bindings) {
                output.push_str(&rewritten);
            } else {
                output.push_str(line);
            }
            output.push('\n');
        }
        brace_depth = next_depth;
    }

    output.push_str("}\n");

    Ok(output)
}

fn rewrite_bound_component_resolution(
    line: &str,
    bindings: Option<&BindingMetadata>,
) -> Option<String> {
    let bindings = bindings?;
    let trimmed = line.trim_start();
    if !trimmed.starts_with("const _component_") {
        return None;
    }

    let (declaration, resolve_call) = trimmed.split_once(" = _resolveComponent(\"")?;
    let (tag, _) = resolve_call.split_once("\")")?;
    let binding_name = resolve_component_binding_name(bindings, tag)?;

    let indent_len = line.len().saturating_sub(trimmed.len());
    let binding_expr = {
        let mut expr = String::with_capacity(binding_name.len() + 5);
        expr.push_str("_ctx.");
        expr.push_str(&binding_name);
        expr
    };

    let mut rewritten = String::with_capacity(line.len() + binding_expr.len());
    rewritten.push_str(line.get(..indent_len)?);
    rewritten.push_str(declaration);
    rewritten.push_str(" = ");
    rewritten.push_str(&binding_expr);
    Some(rewritten)
}

fn resolve_component_binding_name(bindings: &BindingMetadata, tag: &str) -> Option<String> {
    let resolve_base = |name: &str| {
        if bindings.bindings.contains_key(name) {
            return Some(name.to_compact_string());
        }

        let camel = camelize_component_name(name);
        if bindings.bindings.contains_key(camel.as_str()) {
            return Some(camel);
        }

        let pascal = capitalize_component_name(camel.as_str());
        if bindings.bindings.contains_key(pascal.as_str()) {
            return Some(pascal);
        }

        None
    };

    if let Some((base, suffix)) = tag.split_once('.') {
        let resolved_base = resolve_base(base)?;
        let mut resolved = String::with_capacity(resolved_base.len() + suffix.len() + 1);
        resolved.push_str(resolved_base.as_str());
        resolved.push('.');
        resolved.push_str(suffix);
        return Some(resolved);
    }

    resolve_base(tag)
}

fn camelize_component_name(tag: &str) -> String {
    let mut result = String::with_capacity(tag.len());
    let mut uppercase_next = false;
    for ch in tag.chars() {
        if ch == '-' {
            uppercase_next = true;
            continue;
        }

        if uppercase_next {
            result.push(ch.to_ascii_uppercase());
            uppercase_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

fn capitalize_component_name(tag: &str) -> String {
    let mut chars = tag.chars();
    let Some(first) = chars.next() else {
        return String::default();
    };

    let mut result = String::with_capacity(tag.len());
    result.push(first.to_ascii_uppercase());
    for ch in chars {
        result.push(ch);
    }
    result
}

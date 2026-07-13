//! Vapor mode template compilation.

mod bindings;
mod output;
mod shared;

pub(crate) use shared::compile_template_block_vapor_from_syntax;

use bindings::rewrite_bound_component_resolution;
use output::{
    VaporTemplateModule, is_render_signature, rewrite_vapor_import, vapor_module_sections,
};

use super::string_tracking::{StringTrackState, count_braces_with_state};
use vize_atelier_vapor::{
    VaporCompilerOptions, compile_vapor_with_template_syntax_and_diagnostics,
};
use vize_carton::{Bump, String, ToCompactString};
use vize_relief::CodegenOptions;
use vize_relief::TemplateSyntaxMode;

use crate::{
    compile::fallbacks::record_unsupported_vapor_shape,
    compile::output_module::AtelierOutputMaps,
    compile_template::{TemplateBlockCompileResult, recoverable_template_warnings},
    types::{BindingMetadata, SfcError, SfcTemplateBlock, TemplateCompileOptions},
};

/// Borrowed inputs shared by direct and parse-once Vapor template compilation.
#[derive(Clone, Copy)]
pub(crate) struct VaporTemplateCompileContext<'a> {
    pub(crate) scope_id: &'a str,
    pub(crate) has_scoped: bool,
    pub(crate) options: &'a TemplateCompileOptions,
    pub(crate) template_syntax: TemplateSyntaxMode,
    pub(crate) codegen_options: &'a CodegenOptions,
}

/// Compile template block using Vapor mode
pub(crate) fn compile_template_block_vapor(
    template: &SfcTemplateBlock,
    context: VaporTemplateCompileContext<'_>,
    bindings: Option<&BindingMetadata>,
) -> Result<TemplateBlockCompileResult, SfcError> {
    let VaporTemplateCompileContext {
        scope_id,
        has_scoped,
        options,
        template_syntax,
        codegen_options,
    } = context;
    let allocator = Bump::new();
    let compiler_options = options.compiler_options.as_ref();

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

    // Compile template with Vapor
    let (result, diagnostics) = compile_vapor_with_template_syntax_and_diagnostics(
        &allocator,
        &template.content,
        vapor_opts,
        template_syntax,
    );

    if !result.error_messages.is_empty() {
        // Vapor could not lower this template shape: record the shared
        // unsupported-shape fallback before surfacing the hard error.
        record_unsupported_vapor_shape();
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
    let scope_attr = if has_scoped {
        let mut attr = String::with_capacity(scope_id.len() + 7);
        attr.push_str("data-v-");
        attr.push_str(scope_id);
        attr
    } else {
        String::default()
    };

    let output = transform_vapor_template_module(
        &result.code,
        has_scoped.then_some(scope_attr.as_str()),
        template,
        bindings,
        codegen_options.runtime_module_name.as_str(),
    )?;

    Ok(TemplateBlockCompileResult {
        code: output.code,
        warnings: recoverable_template_warnings(&diagnostics),
        sections: None,
        module_sections: Some(output.module_sections),
        maps: AtelierOutputMaps::default(),
    })
}

/// Add scope ID to template string
pub(super) fn add_scope_id_to_template(template_line: &str, scope_id: &str) -> String {
    // Find the template string content and add scope_id to the first element
    if let Some(start) = template_line.find("\"<")
        && let Some(end) = template_line.rfind(">\"")
    {
        let prefix = &template_line[..start + 2]; // up to and including "<"
        let content = &template_line[start + 2..end + 1]; // element content
        let suffix = &template_line[end + 1..]; // closing quote and paren

        // Find end of first tag name
        if let Some(tag_end) = content.find(|c: char| c.is_whitespace() || c == '>') {
            let tag_name = &content[..tag_end];
            let rest = &content[tag_end..];

            // Insert scope_id attribute after tag name
            let mut result = String::with_capacity(
                prefix.len() + tag_name.len() + scope_id.len() + rest.len() + suffix.len() + 1,
            );
            result.push_str(prefix);
            result.push_str(tag_name);
            result.push(' ');
            result.push_str(scope_id);
            result.push_str(rest);
            result.push_str(suffix);
            return result;
        }
    }
    template_line.to_compact_string()
}

#[cfg(test)]
pub(super) fn transform_vapor_template_output(
    code: &str,
    scope_attr: Option<&str>,
    template: &SfcTemplateBlock,
    bindings: Option<&BindingMetadata>,
    runtime_module_name: &str,
) -> Result<String, SfcError> {
    transform_vapor_template_module(code, scope_attr, template, bindings, runtime_module_name)
        .map(|module| module.code)
}

fn transform_vapor_template_module(
    code: &str,
    scope_attr: Option<&str>,
    template: &SfcTemplateBlock,
    bindings: Option<&BindingMetadata>,
    runtime_module_name: &str,
) -> Result<VaporTemplateModule, SfcError> {
    let lines: Vec<&str> = code.lines().collect();
    let mut imports = String::default();
    let mut hoists = String::default();
    let mut functions = String::default();
    let mut pending_separator = String::default();
    let mut index = 0usize;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        if trimmed.starts_with("import ") {
            imports.push_str(&rewrite_vapor_import(line, runtime_module_name));
            imports.push('\n');
            index += 1;
            continue;
        }
        if trimmed.is_empty() {
            index += 1;
            continue;
        }
        break;
    }

    let mut found_render = false;
    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        if is_render_signature(trimmed) {
            found_render = true;
            break;
        }

        if trimmed.is_empty() {
            pending_separator.push_str(line);
            pending_separator.push('\n');
            index += 1;
            continue;
        }

        hoists.push_str(&pending_separator);
        pending_separator.clear();

        if trimmed.starts_with("const t") && trimmed.contains("_template(") {
            if let Some(scope_id) = scope_attr {
                hoists.push_str(&add_scope_id_to_template(line, scope_id));
            } else {
                hoists.push_str(line);
            }
            hoists.push('\n');
        } else {
            hoists.push_str(line);
            hoists.push('\n');
        }
        index += 1;
    }

    if !found_render {
        return Err(SfcError {
            message: "Vapor template output is missing a render function".to_compact_string(),
            code: Some("VAPOR_TEMPLATE_ERROR".to_compact_string()),
            loc: Some(template.loc.clone()),
        });
    }

    functions.push_str("function render(_ctx, $props, $emit, $attrs, $slots) {\n");

    let mut brace_state = StringTrackState::default();
    let mut brace_depth = count_braces_with_state(lines[index], &mut brace_state);
    index += 1;

    while index < lines.len() && brace_depth > 0 {
        let line = lines[index];
        let next_depth = brace_depth + count_braces_with_state(line, &mut brace_state);
        if !(next_depth == 0 && line.trim() == "}") {
            if let Some(rewritten) = rewrite_bound_component_resolution(line, bindings) {
                functions.push_str(&rewritten);
            } else {
                functions.push_str(line);
            }
            functions.push('\n');
        }
        brace_depth = next_depth;
        index += 1;
    }

    functions.push_str("}\n");

    // Vapor assembly preserves the blank separator before the render function,
    // but that separator is not part of the hoist section. Inline assembly
    // expects hoists to contain only movable module declarations.
    let module_sections = vapor_module_sections(
        imports.len(),
        hoists.len(),
        pending_separator.len(),
        functions.len(),
    );
    let mut module_code = String::with_capacity(
        imports.len() + hoists.len() + pending_separator.len() + functions.len(),
    );
    module_code.push_str(&imports);
    module_code.push_str(&hoists);
    module_code.push_str(&pending_separator);
    module_code.push_str(&functions);

    Ok(VaporTemplateModule {
        code: module_code,
        module_sections,
    })
}

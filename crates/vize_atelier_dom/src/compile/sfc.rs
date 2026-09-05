use super::{
    compile_template_inner_with_sections,
    pipeline::{self, DomCompilePipelineOptions},
    stage_options,
};
use crate::options::DomCompilerOptions;
use vize_atelier_core::{
    CompilerError, RootNode,
    codegen::{CodegenResult, CodegenResultWithSections},
    options::{CodegenOptions, CustomElementMatcher, TemplateSyntaxMode},
};
use vize_s0::{Allocator, String, profile};

pub(super) fn compile_template_inner_for_sfc_with_sections<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
) -> (Vec<CompilerError>, CodegenResultWithSections) {
    let codegen_opts = stage_options::codegen_options(&options, codegen_options.clone());
    let use_s2_emit = stage_options::s2_emit_supported(
        &options,
        &codegen_opts,
        !custom_elements.is_empty(),
        template_syntax,
        options.croquis.is_some(),
        pipeline::S2EmitSelection::RequireSections,
    );

    let mut force_compat_sections = false;

    if use_s2_emit && s2_sfc_fast_path_supported_source(source) {
        let binding_table = stage_options::s2_binding_table(options.binding_metadata.as_ref());
        let s2_options = stage_options::s2_emit_options(
            &options,
            &codegen_opts,
            binding_table.as_ref(),
            hoisted_scope_id.as_deref(),
        );
        if let Ok(result) = profile!(
            "atelier.dom.template.s2_codegen_sfc_fast",
            stage_options::emit_s2(allocator, source, options.dialect, &s2_options)
        ) {
            return (Vec::new(), result);
        }
        force_compat_sections = true;
    } else if use_s2_emit {
        force_compat_sections = true;
    }

    let pipeline_options = if force_compat_sections {
        DomCompilePipelineOptions::require_sections_compat(custom_elements, codegen_options)
    } else {
        DomCompilePipelineOptions::require_sections(custom_elements, codegen_options)
    };

    let (_, errors, result) = compile_template_inner_with_sections(
        allocator,
        source,
        options,
        template_syntax,
        hoisted_scope_id,
        pipeline_options,
    );
    (errors, result)
}

pub(super) fn compile_template_inner_for_sfc<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    let (root, errors, result) = compile_template_inner_with_sections(
        allocator,
        source,
        options,
        template_syntax,
        hoisted_scope_id,
        DomCompilePipelineOptions::require_sections_compat(custom_elements, codegen_options),
    );
    (root, errors, result.into_result())
}

fn s2_sfc_fast_path_supported_source(source: &str) -> bool {
    !source_contains_non_void_native_self_closing_tag(source)
}

fn source_contains_non_void_native_self_closing_tag(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut tags = Vec::new();
    let mut index = 0;

    while let Some(offset) = source[index..].find('<') {
        let name_start = index + offset + 1;
        if name_start >= bytes.len() {
            break;
        }

        if bytes[name_start] == b'/' {
            let closing_name_start = name_start + 1;
            let closing_name_end = scan_tag_name(bytes, closing_name_start);
            if closing_name_end > closing_name_start {
                pop_closed_tag(&mut tags, &source[closing_name_start..closing_name_end]);
                index = closing_name_end;
                continue;
            }
        }

        if matches!(bytes[name_start], b'!' | b'?') {
            index = name_start + 1;
            continue;
        }

        let name_end = scan_tag_name(bytes, name_start);
        if name_end == name_start {
            index = name_start + 1;
            continue;
        }

        let name = &source[name_start..name_end];
        let namespace = tag_namespace(name, tags.last().copied());
        let self_closing = tag_closes_self_closing(bytes, name_end);
        if namespace == SourceNamespace::Html
            && is_plain_native_html_tag_name(name)
            && !is_html_void_tag_name(name)
            && !is_allowed_self_closing_special_tag_name(name)
            && self_closing
        {
            return true;
        }

        if !self_closing {
            tags.push(name);
        }

        index = name_end;
    }

    false
}

fn pop_closed_tag(tags: &mut Vec<&str>, name: &str) {
    if tags.last().is_some_and(|tag| *tag == name) {
        tags.pop();
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SourceNamespace {
    Html,
    Foreign,
}

fn tag_namespace(tag: &str, parent: Option<&str>) -> SourceNamespace {
    if vize_s0::is_svg_tag(tag) || vize_s0::is_math_ml_tag(tag) {
        return SourceNamespace::Foreign;
    }

    let Some(parent) = parent else {
        return SourceNamespace::Html;
    };

    let svg_to_html = matches!(parent, "foreignObject" | "desc" | "title");
    if vize_s0::is_svg_tag(parent) && !svg_to_html {
        return SourceNamespace::Foreign;
    }

    let mathml_to_html = matches!(
        parent,
        "annotation-xml" | "mi" | "mo" | "mn" | "ms" | "mtext"
    );
    if vize_s0::is_math_ml_tag(parent) && !mathml_to_html {
        return SourceNamespace::Foreign;
    }

    SourceNamespace::Html
}

fn scan_tag_name(bytes: &[u8], start: usize) -> usize {
    let mut end = start;
    while end < bytes.len() && is_tag_name_byte(bytes[end]) {
        end += 1;
    }
    end
}

fn is_tag_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':')
}

fn is_plain_native_html_tag_name(name: &str) -> bool {
    name.bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn is_html_void_tag_name(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn is_allowed_self_closing_special_tag_name(name: &str) -> bool {
    matches!(name, "component" | "slot")
}

fn tag_closes_self_closing(bytes: &[u8], start: usize) -> bool {
    let mut index = start;
    let mut quote = None;
    let mut last_non_whitespace = None;

    while index < bytes.len() {
        let byte = bytes[index];

        if let Some(quote_byte) = quote {
            if byte == quote_byte {
                quote = None;
            }
            index += 1;
            continue;
        }

        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'>' => return last_non_whitespace == Some(b'/'),
            b if b.is_ascii_whitespace() => {}
            _ => last_non_whitespace = Some(byte),
        }

        index += 1;
    }

    false
}

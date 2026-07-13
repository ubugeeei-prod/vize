use vize_armature::parse_with_options_and_template_syntax;
use vize_atelier_core::{
    codegen::{CodegenResult, CodegenResultWithSections, generate_with_sections},
    lane::{
        transform as do_transform, transform_with_hoisted_scope_id,
        transform_with_template_syntax_quirks,
        transform_with_template_syntax_quirks_and_hoisted_scope_id,
    },
};
use vize_carton::{Bump, String, profile};
use vize_croquis::Croquis;
use vize_relief::{
    CodegenOptions, CompilerError, ParserOptions, RootNode, TemplateSyntaxMode, TransformOptions,
};

use crate::{namespace::get_namespace, options::DomCompilerOptions};

pub(super) fn compile_template_inner<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    codegen_options: CodegenOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    let (root, errors, codegen_result) = compile_template_inner_with_sections(
        allocator,
        source,
        options,
        template_syntax,
        hoisted_scope_id,
        codegen_options,
    );
    (root, errors, codegen_result.into_result())
}

pub(super) fn compile_template_inner_with_sections<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    codegen_options: CodegenOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResultWithSections) {
    let parser_opts = ParserOptions {
        is_void_tag: vize_carton::is_void_tag,
        is_native_tag: Some(vize_carton::is_native_tag),
        custom_renderer: options.custom_renderer,
        is_pre_tag: |tag| tag == "pre",
        get_namespace,
        comments: options.comments,
        experimental_in_tag_comments: options.experimental_in_tag_comments,
        dialect: options.dialect,
        ..ParserOptions::default()
    };
    let (root, errors) = profile!(
        "atelier.dom.template.parse",
        parse_with_options_and_template_syntax(allocator, source, parser_opts, template_syntax)
    );
    compile_template_root_with_sections(
        allocator,
        root,
        errors.to_vec(),
        options,
        template_syntax,
        hoisted_scope_id,
        codegen_options,
    )
}

pub(super) fn compile_template_root_with_sections<'a>(
    allocator: &'a Bump,
    mut root: RootNode<'a>,
    mut errors: Vec<CompilerError>,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    codegen_options: CodegenOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResultWithSections) {
    let fatal_count = errors
        .iter()
        .filter(|error| !error.is_recoverable())
        .count();
    if fatal_count > 0 {
        let codegen_result = CodegenResult {
            code: String::default(),
            preamble: String::default(),
            map: None,
        };
        return (
            root,
            errors,
            CodegenResultWithSections {
                result: codegen_result,
                sections: None,
            },
        );
    }

    let transform_opts = TransformOptions {
        prefix_identifiers: options.prefix_identifiers,
        hoist_static: options.hoist_static,
        cache_handlers: options.cache_handlers,
        scope_id: options.scope_id.clone(),
        ssr: options.ssr,
        is_ts: options.is_ts,
        inline: options.inline,
        custom_renderer: options.custom_renderer,
        experimental_patterned_template: options.experimental_patterned_template,
        binding_metadata: options.binding_metadata.clone(),
        dialect: options.dialect,
        ..Default::default()
    };
    let analysis: Option<&Croquis> = options.croquis.map(|value| &*allocator.alloc(*value));
    let transform_errors = profile!(
        "atelier.dom.template.transform",
        transform(
            allocator,
            &mut root,
            transform_opts,
            analysis,
            template_syntax,
            hoisted_scope_id,
        )
    );
    errors.extend(transform_errors.errors);

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
        ..codegen_options
    };
    let codegen_result = profile!(
        "atelier.dom.template.codegen",
        generate_with_sections(&root, &transform_errors.hoists, codegen_opts)
    );
    (root, errors, codegen_result)
}

fn transform<'a>(
    allocator: &'a Bump,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
    syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
) -> vize_atelier_core::lane::TransformResult<'a> {
    if syntax.is_quirks() {
        if hoisted_scope_id.is_some() {
            return transform_with_template_syntax_quirks_and_hoisted_scope_id(
                allocator,
                root,
                options,
                analysis,
                hoisted_scope_id,
            );
        }
        return transform_with_template_syntax_quirks(allocator, root, options, analysis);
    }
    if hoisted_scope_id.is_some() {
        return transform_with_hoisted_scope_id(
            allocator,
            root,
            options,
            analysis,
            hoisted_scope_id,
        );
    }
    do_transform(allocator, root, options, analysis)
}

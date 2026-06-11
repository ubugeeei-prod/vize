//! DOM template compilation: parse, transform, and codegen entry points.

use vize_atelier_core::codegen::CodegenResult;
use vize_atelier_core::{
    CompilerError, RootNode,
    codegen::generate,
    options::{CodegenOptions, ParserOptions, TemplateSyntaxMode, TransformOptions},
    parser::parse_with_options_and_template_syntax,
    transform::{
        transform as do_transform, transform_with_hoisted_scope_id,
        transform_with_template_syntax_quirks,
        transform_with_template_syntax_quirks_and_hoisted_scope_id,
    },
};
use vize_carton::{Bump, String, profile};
use vize_croquis::Croquis;

use crate::namespace::get_namespace;
use crate::options::DomCompilerOptions;

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
    compile_template_inner(
        allocator,
        source,
        options,
        TemplateSyntaxMode::Standard,
        None,
    )
}

/// Compile a Vue template for DOM with Vue parser quirk compatibility.
#[deprecated(note = "use compile_template_with_template_syntax instead")]
pub fn compile_template_with_vue_parser_quirks<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: DomCompilerOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_inner(allocator, source, options, TemplateSyntaxMode::Quirks, None)
}

/// Compile a Vue template for DOM with an explicit template syntax mode.
#[doc(hidden)]
pub fn compile_template_with_template_syntax<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_inner(allocator, source, options, template_syntax, None)
}

/// Compile a Vue template for DOM with an explicit scope ID for hoisted static VNodes.
#[doc(hidden)]
pub fn compile_template_with_options_and_hoisted_scope_id<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: DomCompilerOptions,
    hoisted_scope_id: Option<String>,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_inner(
        allocator,
        source,
        options,
        TemplateSyntaxMode::Standard,
        hoisted_scope_id,
    )
}

/// Compile a Vue template for DOM with Vue parser quirks and an explicit hoisted scope ID.
#[doc(hidden)]
#[deprecated(note = "use compile_template_with_template_syntax_and_hoisted_scope_id instead")]
pub fn compile_template_with_vue_parser_quirks_and_hoisted_scope_id<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: DomCompilerOptions,
    hoisted_scope_id: Option<String>,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_inner(
        allocator,
        source,
        options,
        TemplateSyntaxMode::Quirks,
        hoisted_scope_id,
    )
}

/// Compile a Vue template for DOM with template syntax mode and hoisted scope ID.
#[doc(hidden)]
pub fn compile_template_with_template_syntax_and_hoisted_scope_id<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_inner(
        allocator,
        source,
        options,
        template_syntax,
        hoisted_scope_id,
    )
}

fn compile_template_inner<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
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
        parse_with_options_and_template_syntax(allocator, source, parser_opts, template_syntax)
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
    let template_syntax_quirks = template_syntax.is_quirks();
    // Allocate Croquis in the arena so it shares the allocator lifetime
    let analysis: Option<&Croquis> = options.croquis.map(|c| &*allocator.alloc(*c));
    let transform_errors = profile!(
        "atelier.dom.template.transform",
        if template_syntax_quirks {
            if hoisted_scope_id.is_some() {
                transform_with_template_syntax_quirks_and_hoisted_scope_id(
                    allocator,
                    &mut root,
                    transform_opts,
                    analysis,
                    hoisted_scope_id,
                )
            } else {
                transform_with_template_syntax_quirks(
                    allocator,
                    &mut root,
                    transform_opts,
                    analysis,
                )
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

    // Surface transform diagnostics (e.g. invalid expressions) alongside
    // parse errors instead of dropping them — the official compiler reports
    // both through the same `errors` channel.
    let mut errors = errors.to_vec();
    errors.extend(transform_errors);

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

    (root, errors, codegen_result)
}

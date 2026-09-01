//! DOM template compilation: parse, transform, and codegen entry points.

pub mod custom_elements;

use vize_atelier_core::codegen::{CodegenResult, CodegenResultWithSections};
use vize_atelier_core::{
    CompilerError, RootNode,
    codegen::generate_with_sections,
    lane::transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id,
    options::{CodegenOptions, CustomElementMatcher, TemplateSyntaxMode},
    parser::parse_with_options_custom_elements_and_template_syntax,
};
use vize_croquis::Croquis;
use vize_s0::{Allocator, String, profile};

mod stage_options;

use crate::options::DomCompilerOptions;

/// Compile a Vue template for DOM with default options
pub fn compile_template<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_with_options(allocator, source, DomCompilerOptions::default())
}

/// Compile a Vue template for DOM with custom options
pub fn compile_template_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_inner(
        allocator,
        source,
        options,
        TemplateSyntaxMode::Standard,
        None,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
    )
}

/// Compile a Vue template for DOM with Vue parser quirk compatibility.
#[deprecated(note = "use compile_template_with_template_syntax instead")]
pub fn compile_template_with_vue_parser_quirks<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_inner(
        allocator,
        source,
        options,
        TemplateSyntaxMode::Quirks,
        None,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
    )
}

/// Compile a Vue template for DOM with an explicit template syntax mode.
#[doc(hidden)]
pub fn compile_template_with_template_syntax<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_inner(
        allocator,
        source,
        options,
        template_syntax,
        None,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
    )
}

/// Compile a Vue template with adapter-provided codegen defaults.
///
/// DOM-owned settings such as mode, source maps, and binding metadata still
/// take precedence. This hook lets binding facades provide emission-only
/// settings (for example runtime names and the source-map filename) without
/// growing [`DomCompilerOptions`] and breaking downstream struct literals.
#[doc(hidden)]
pub fn compile_template_with_template_syntax_and_codegen_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    codegen_options: CodegenOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_inner(
        allocator,
        source,
        options,
        template_syntax,
        None,
        CustomElementMatcher::default(),
        codegen_options,
    )
}

/// Compile a Vue template for DOM with an explicit scope ID for hoisted static VNodes.
#[doc(hidden)]
pub fn compile_template_with_options_and_hoisted_scope_id<'a>(
    allocator: &'a Allocator,
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
        CustomElementMatcher::default(),
        CodegenOptions::default(),
    )
}

/// Compile a Vue template for DOM with Vue parser quirks and an explicit hoisted scope ID.
#[doc(hidden)]
#[deprecated(note = "use compile_template_with_template_syntax_and_hoisted_scope_id instead")]
pub fn compile_template_with_vue_parser_quirks_and_hoisted_scope_id<'a>(
    allocator: &'a Allocator,
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
        CustomElementMatcher::default(),
        CodegenOptions::default(),
    )
}

/// Compile a Vue template for DOM with template syntax mode and hoisted scope ID.
#[doc(hidden)]
pub fn compile_template_with_template_syntax_and_hoisted_scope_id<'a>(
    allocator: &'a Allocator,
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
        CustomElementMatcher::default(),
        CodegenOptions::default(),
    )
}

/// Compile a Vue template for DOM with template syntax mode, hoisted scope ID,
/// and emission-recorded codegen section boundaries.
#[doc(hidden)]
pub fn compile_template_with_template_syntax_and_hoisted_scope_id_with_sections<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResultWithSections) {
    compile_template_inner_with_sections(
        allocator,
        source,
        options,
        template_syntax,
        hoisted_scope_id,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
    )
}

/// Compile a Vue template with section metadata and adapter-provided codegen
/// defaults. See [`compile_template_with_template_syntax_and_codegen_options`].
#[doc(hidden)]
pub fn compile_template_with_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options<
    'a,
>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    codegen_options: CodegenOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResultWithSections) {
    compile_template_inner_with_sections(
        allocator,
        source,
        options,
        template_syntax,
        hoisted_scope_id,
        CustomElementMatcher::default(),
        codegen_options,
    )
}

fn compile_template_inner<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    let (root, errors, codegen_result) = compile_template_inner_with_sections(
        allocator,
        source,
        options,
        template_syntax,
        hoisted_scope_id,
        custom_elements,
        codegen_options,
    );
    (root, errors, codegen_result.into_result())
}

fn compile_template_inner_with_sections<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResultWithSections) {
    let parser_opts = stage_options::parser_options(&options);

    // Parse
    let (mut root, errors) = profile!(
        "atelier.dom.template.parse",
        parse_with_options_custom_elements_and_template_syntax(
            allocator,
            source,
            parser_opts,
            custom_elements.clone(),
            template_syntax,
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
        return (
            root,
            errors.to_vec(),
            CodegenResultWithSections {
                result: codegen_result,
                sections: None,
            },
        );
    }

    let transform_opts = stage_options::transform_options(&options);
    let template_syntax_quirks = template_syntax.is_quirks();
    // Park the summary on the allocator so it shares the allocator lifetime.
    let analysis: Option<&Croquis> = options.croquis.map(|c| allocator.alloc_owned(*c));
    let transform_errors = profile!(
        "atelier.dom.template.transform",
        transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id(
            allocator,
            &mut root,
            transform_opts,
            analysis,
            custom_elements,
            template_syntax_quirks,
            hoisted_scope_id,
        )
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
        // Compound dynamic `v-bind` / `v-on` keys (`:[prefix+suffix]`) only
        // walk identifiers when this flag is set. Transform already receives
        // it; omitting it here left SFC module-mode render functions with
        // bare `prefix+suffix` and a runtime ReferenceError.
        prefix_identifiers: options.prefix_identifiers,
        ..codegen_options
    };
    let codegen_result = profile!(
        "atelier.dom.template.codegen",
        generate_with_sections(&root, codegen_opts)
    );

    (root, errors, codegen_result)
}

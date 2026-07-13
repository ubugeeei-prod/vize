//! DOM template compilation: parse, transform, and codegen entry points.

mod engine;

use vize_atelier_core::codegen::{CodegenResult, CodegenResultWithSections};
use vize_carton::{Bump, String};
use vize_relief::{CodegenOptions, CompilerError, RootNode, TemplateSyntaxMode};

use crate::options::DomCompilerOptions;
use engine::{
    compile_template_inner, compile_template_inner_with_sections,
    compile_template_root_with_sections,
};

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
        CodegenOptions::default(),
    )
}

/// Compile a Vue template for DOM with Vue parser quirk compatibility.
#[deprecated(note = "use compile_template_with_template_syntax instead")]
pub fn compile_template_with_vue_parser_quirks<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: DomCompilerOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_inner(
        allocator,
        source,
        options,
        TemplateSyntaxMode::Quirks,
        None,
        CodegenOptions::default(),
    )
}

/// Compile a Vue template for DOM with an explicit template syntax mode.
#[doc(hidden)]
pub fn compile_template_with_template_syntax<'a>(
    allocator: &'a Bump,
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
    allocator: &'a Bump,
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
        codegen_options,
    )
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
        CodegenOptions::default(),
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
        CodegenOptions::default(),
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
        CodegenOptions::default(),
    )
}

/// Compile a Vue template for DOM with template syntax mode, hoisted scope ID,
/// and emission-recorded codegen section boundaries.
#[doc(hidden)]
pub fn compile_template_with_template_syntax_and_hoisted_scope_id_with_sections<'a>(
    allocator: &'a Bump,
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
        CodegenOptions::default(),
    )
}

/// Compile a Vue template with section metadata and adapter-provided codegen
/// defaults. See [`compile_template_with_template_syntax_and_codegen_options`].
#[doc(hidden)]
pub fn compile_template_with_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options<
    'a,
>(
    allocator: &'a Bump,
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
        codegen_options,
    )
}

/// Transform and emit a template from an already parsed Relief root.
///
/// This is the production bridge for Atlas syntax products: parsing and parse
/// diagnostics are shared upstream, while each backend still applies its own
/// transform and codegen options to an independently materialized arena tree.
#[doc(hidden)]
pub fn compile_template_root_with_template_syntax_and_hoisted_scope_id_with_sections<'a>(
    allocator: &'a Bump,
    root: RootNode<'a>,
    parse_errors: Vec<CompilerError>,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResultWithSections) {
    compile_template_root_with_sections(
        allocator,
        root,
        parse_errors,
        options,
        template_syntax,
        hoisted_scope_id,
        CodegenOptions::default(),
    )
}

/// Transform and emit an already parsed Relief root with adapter codegen defaults.
#[doc(hidden)]
pub fn compile_template_root_with_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options<
    'a,
>(
    allocator: &'a Bump,
    root: RootNode<'a>,
    parse_errors: Vec<CompilerError>,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    codegen_options: CodegenOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResultWithSections) {
    compile_template_root_with_sections(
        allocator,
        root,
        parse_errors,
        options,
        template_syntax,
        hoisted_scope_id,
        codegen_options,
    )
}

use super::{
    compile_template_inner, compile_template_inner_with_sections,
    pipeline::DomCompilePipelineOptions,
    sfc::{compile_template_inner_for_sfc, compile_template_inner_for_sfc_with_sections},
};
use crate::DomCompilerOptions;
use vize_atelier_core::{
    CompilerError, RootNode,
    codegen::{CodegenResult, CodegenResultWithSections},
    options::{CodegenOptions, CustomElementMatcher, TemplateSyntaxMode},
};
use vize_s0::{Allocator, String};

/// Compile with declarative custom-element patterns without growing public options.
#[doc(hidden)]
pub fn compile_template_with_custom_elements_and_template_syntax_and_codegen_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_inner(
        allocator,
        source,
        options,
        template_syntax,
        None,
        custom_elements,
        codegen_options,
    )
}

/// Compile with section metadata and declarative custom-element patterns.
#[doc(hidden)]
pub fn compile_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options<
    'a,
>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResultWithSections) {
    compile_template_inner_with_sections(
        allocator,
        source,
        options,
        template_syntax,
        hoisted_scope_id,
        DomCompilePipelineOptions::require_sections(custom_elements, codegen_options),
    )
}

/// Compile with declarative custom-element patterns and an SFC hoisted scope ID.
#[doc(hidden)]
pub fn compile_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_and_codegen_options<
    'a,
>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
) -> (RootNode<'a>, Vec<CompilerError>, CodegenResult) {
    compile_template_inner_for_sfc(
        allocator,
        source,
        options,
        template_syntax,
        hoisted_scope_id,
        custom_elements,
        codegen_options,
    )
}

/// Compile an SFC template block with section metadata and custom-element patterns.
///
/// This entry returns only diagnostics and codegen because SFC assembly does
/// not consume the DOM `RootNode`. That lets the fully supported S2 sections
/// lane avoid constructing the legacy transformed AST solely to throw it away;
/// unsupported or diagnostic inputs still fall back to the ordinary section
/// path so warning/error behavior stays with the shipped compiler.
#[doc(hidden)]
pub fn compile_sfc_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options<
    'a,
>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
) -> (Vec<CompilerError>, CodegenResultWithSections) {
    compile_template_inner_for_sfc_with_sections(
        allocator,
        source,
        options,
        template_syntax,
        hoisted_scope_id,
        custom_elements,
        codegen_options,
    )
}

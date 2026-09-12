//! Public Vapor compiler entrypoints.

use vize_atelier_core::{
    CompilerError,
    options::{CustomElementMatcher, TemplateSyntaxMode},
};
use vize_carton::Allocator;

use super::{
    VaporCompileResult, VaporCompilerExperimentalOptions, VaporCompilerOptions, compile_vapor_inner,
};

/// Compile a Vue template to Vapor mode
pub fn compile_vapor<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
) -> VaporCompileResult {
    compile_vapor_inner(
        allocator,
        source,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        VaporCompilerExperimentalOptions::default(),
    )
    .0
}

/// Compile a Vue template to Vapor mode with opt-in experimental codegen
/// context.
#[doc(hidden)]
pub fn compile_vapor_with_experimental_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
    experimental_options: VaporCompilerExperimentalOptions,
) -> VaporCompileResult {
    compile_vapor_inner(
        allocator,
        source,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        experimental_options,
    )
    .0
}

/// Compile a Vue template to Vapor mode with Vue parser quirk compatibility.
#[deprecated(note = "use compile_vapor_with_template_syntax instead")]
pub fn compile_vapor_with_vue_parser_quirks<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
) -> VaporCompileResult {
    compile_vapor_inner(
        allocator,
        source,
        options,
        TemplateSyntaxMode::Quirks,
        CustomElementMatcher::default(),
        VaporCompilerExperimentalOptions::default(),
    )
    .0
}

/// Compile a Vue template to Vapor mode with an explicit template syntax mode.
#[doc(hidden)]
pub fn compile_vapor_with_template_syntax<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
) -> VaporCompileResult {
    compile_vapor_inner(
        allocator,
        source,
        options,
        template_syntax,
        CustomElementMatcher::default(),
        VaporCompilerExperimentalOptions::default(),
    )
    .0
}

/// Compile a Vue template to Vapor mode with an explicit template syntax mode
/// and opt-in experimental codegen context.
#[doc(hidden)]
pub fn compile_vapor_with_template_syntax_and_experimental_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    experimental_options: VaporCompilerExperimentalOptions,
) -> VaporCompileResult {
    compile_vapor_inner(
        allocator,
        source,
        options,
        template_syntax,
        CustomElementMatcher::default(),
        experimental_options,
    )
    .0
}

/// Compile with declarative custom-element patterns.
#[doc(hidden)]
pub fn compile_vapor_with_custom_elements_and_template_syntax<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
) -> VaporCompileResult {
    compile_vapor_inner(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        VaporCompilerExperimentalOptions::default(),
    )
    .0
}

/// Compile with declarative custom-element patterns and opt-in experimental
/// codegen context.
#[doc(hidden)]
pub fn compile_vapor_with_custom_elements_template_syntax_and_experimental_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: VaporCompilerExperimentalOptions,
) -> VaporCompileResult {
    compile_vapor_inner(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        experimental_options,
    )
    .0
}

/// Compile a Vue template to Vapor mode and return parser diagnostics.
#[doc(hidden)]
pub fn compile_vapor_with_diagnostics<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    compile_vapor_inner(
        allocator,
        source,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        VaporCompilerExperimentalOptions::default(),
    )
}

/// Compile a Vue template to Vapor mode with Vue parser quirks and return parser diagnostics.
#[doc(hidden)]
#[deprecated(note = "use compile_vapor_with_template_syntax_and_diagnostics instead")]
pub fn compile_vapor_with_vue_parser_quirks_and_diagnostics<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    compile_vapor_inner(
        allocator,
        source,
        options,
        TemplateSyntaxMode::Quirks,
        CustomElementMatcher::default(),
        VaporCompilerExperimentalOptions::default(),
    )
}

/// Compile a Vue template to Vapor mode with template syntax mode and return parser diagnostics.
#[doc(hidden)]
pub fn compile_vapor_with_template_syntax_and_diagnostics<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    compile_vapor_inner(
        allocator,
        source,
        options,
        template_syntax,
        CustomElementMatcher::default(),
        VaporCompilerExperimentalOptions::default(),
    )
}

/// Compile with template syntax, diagnostics, and declarative custom-element patterns.
#[doc(hidden)]
pub fn compile_vapor_with_custom_elements_template_syntax_and_diagnostics<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    compile_vapor_inner(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        VaporCompilerExperimentalOptions::default(),
    )
}

/// Compile with template syntax, diagnostics, custom-element patterns, and
/// opt-in experimental codegen context.
#[doc(hidden)]
pub fn compile_vapor_with_custom_elements_template_syntax_diagnostics_and_experimental_options<
    'a,
>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: VaporCompilerExperimentalOptions,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    compile_vapor_inner(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        experimental_options,
    )
}

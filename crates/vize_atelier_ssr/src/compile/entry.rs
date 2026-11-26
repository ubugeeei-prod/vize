//! Public default and parser-mode SSR entry points.

use super::{
    Allocator, CompilerError, CustomElementMatcher, RootNode, SsrCodegenResult,
    SsrCompilerExperimentalOptions, SsrCompilerOptions, TemplateSyntaxMode, compile_ssr_inner,
};

/// Compile a Vue template for SSR with default options
pub fn compile_ssr<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_with_options(allocator, source, SsrCompilerOptions::default())
}

/// Compile a Vue template for SSR with custom options
pub fn compile_ssr_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(
        allocator,
        source,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        SsrCompilerExperimentalOptions::default(),
        true,
    )
}

/// Compile a Vue template for SSR with Vue parser quirk compatibility.
#[deprecated(note = "use compile_ssr_with_template_syntax instead")]
pub fn compile_ssr_with_vue_parser_quirks<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(
        allocator,
        source,
        options,
        TemplateSyntaxMode::Quirks,
        CustomElementMatcher::default(),
        SsrCompilerExperimentalOptions::default(),
        true,
    )
}

/// Compile a Vue template for SSR with an explicit template syntax mode.
#[doc(hidden)]
pub fn compile_ssr_with_template_syntax<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(
        allocator,
        source,
        options,
        template_syntax,
        CustomElementMatcher::default(),
        SsrCompilerExperimentalOptions::default(),
        true,
    )
}

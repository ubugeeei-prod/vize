//! The SSR template compile, through the direct lane when it is switched on.

use vize_atelier_core::{
    CodegenOptions, CompilerError,
    options::{CustomElementMatcher, TemplateSyntaxMode},
};
use vize_atelier_ssr::{
    SsrCodegenResult, SsrCompilerExperimentalOptions, SsrCompilerOptions, SsrSfcContext,
};
use vize_l0::Allocator;
use vize_l0::dump::capture::StageCapture;

/// Compile `source` for SSR, asking for the direct lane when
/// `VIZE_SSR_DIRECT_LANE=1` and the source passes the SFC fast path's test.
#[expect(clippy::too_many_arguments, reason = "the SSR template compile inputs")]
pub(super) fn compile(
    allocator: &Allocator,
    source: &str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: SsrCompilerExperimentalOptions,
    context: SsrSfcContext,
    codegen_options: &CodegenOptions,
) -> (Vec<CompilerError>, SsrCodegenResult) {
    let direct = vize_atelier_ssr::sfc_direct_lane_enabled()
        && vize_atelier_dom::sfc_template_source_is_direct(source, &options.directive_contracts);
    // A requested map is composed into the component's, and only the composed
    // copy of the SSR compile records the links that needs.
    let compile = if experimental_options.source_map {
        vize_atelier_ssr::composed::compile_ssr_sfc_template
    } else {
        vize_atelier_ssr::compile_ssr_sfc_template
    };
    compile(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        experimental_options,
        context,
        codegen_options,
        direct,
    )
}

/// Compile `source` for SSR into `capture`, on the full path.
#[expect(clippy::too_many_arguments, reason = "the SSR template compile inputs")]
pub(super) fn compile_captured(
    allocator: &Allocator,
    source: &str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: SsrCompilerExperimentalOptions,
    context: SsrSfcContext,
    codegen_options: &CodegenOptions,
    capture: &mut StageCapture,
) -> (Vec<CompilerError>, SsrCodegenResult) {
    // Only the composed copy of the SSR compile records the links a map
    // composed into the component's needs.
    let compile = if experimental_options.source_map {
        vize_atelier_ssr::composed::compile_ssr_with_sfc_context_codegen_options_and_capture
    } else {
        vize_atelier_ssr::compile_ssr_with_sfc_context_codegen_options_and_capture
    };
    let (_, errors, result) = compile(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        experimental_options,
        context,
        codegen_options,
        capture,
    );
    (errors, result)
}

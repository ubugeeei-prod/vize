use crate::{
    SsrCodegenContext, SsrCodegenResult, SsrCompilerExperimentalOptions, SsrCompilerOptions,
    l4::{self, SsrL4Request, SsrL4Selection},
};
use vize_atelier_core::{
    CompilerError, ErrorCode, Namespace, RootNode,
    lane::transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id,
    options::{CustomElementMatcher, TemplateSyntaxMode},
    parser::parse_with_options_custom_elements_and_template_syntax,
};
use vize_l0::{
    Allocator, String, cstr,
    dump::capture::{CaptureOutcome, CaptureSink},
    level::Level,
    profile,
};

pub use crate::l4::compile_l2_to_ssr;

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

/// Compile SSR with declarative custom-element patterns.
#[doc(hidden)]
pub fn compile_ssr_with_custom_elements_and_template_syntax<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        SsrCompilerExperimentalOptions::default(),
        true,
    )
}

/// Compile SSR with declarative custom-element patterns and opt-in
/// experimental codegen context.
#[doc(hidden)]
pub fn compile_ssr_with_custom_elements_template_syntax_and_experimental_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: SsrCompilerExperimentalOptions,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        experimental_options,
        true,
    )
}

/// Compile SSR with an explicit template syntax mode and opt-in experimental
/// codegen context.
#[doc(hidden)]
pub fn compile_ssr_with_template_syntax_and_experimental_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    experimental_options: SsrCompilerExperimentalOptions,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(
        allocator,
        source,
        options,
        template_syntax,
        CustomElementMatcher::default(),
        experimental_options,
        true,
    )
}

/// Compile an SFC template with its own scoped slotted-style metadata.
/// Existing direct-template entries retain Vue's default slotted behavior.
#[doc(hidden)]
pub fn compile_ssr_with_sfc_slotted_context<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: SsrCompilerExperimentalOptions,
    slotted: bool,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        experimental_options,
        slotted,
    )
}

/// Compile an SFC template and observe only stages that emitted its module.
#[doc(hidden)]
#[expect(
    clippy::too_many_arguments,
    reason = "SFC options plus an opt-in capture sink"
)]
pub fn compile_ssr_with_sfc_slotted_context_and_capture<'a, C: CaptureSink>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: SsrCompilerExperimentalOptions,
    slotted: bool,
    capture: &mut C,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner_captured(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        experimental_options,
        slotted,
        capture,
    )
}

mod inner;
#[cfg(any(test, feature = "legacy-differential"))]
pub(crate) use inner::{SsrLane, compile_ssr_on_lane};
use inner::{compile_ssr_inner, compile_ssr_inner_captured};

pub(crate) fn get_namespace(tag: &str, parent: Option<&str>) -> Namespace {
    if vize_l0::is_svg_tag(tag) {
        return Namespace::Svg;
    }
    if vize_l0::is_math_ml_tag(tag) {
        return Namespace::MathMl;
    }
    if let Some(parent_tag) = parent {
        if vize_l0::is_svg_tag(parent_tag) && tag != "foreignObject" {
            return Namespace::Svg;
        }
        if vize_l0::is_math_ml_tag(parent_tag) && tag != "annotation-xml" && tag != "foreignObject"
        {
            return Namespace::MathMl;
        }
    }
    Namespace::Html
}

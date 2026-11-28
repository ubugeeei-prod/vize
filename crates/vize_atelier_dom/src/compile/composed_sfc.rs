//! The SFC template compile whose render map is composed into the component's.

use super::sfc::compile_template_for_sfc_composable;
use crate::DomCompilerOptions;
use vize_atelier_core::{
    CompilerError,
    codegen::CodegenResultWithSections,
    options::{
        CodegenExperimentalOptions, CodegenOptions, CustomElementMatcher, TemplateSyntaxMode,
    },
};
use vize_l0::dump::capture::{NoCapture, StageCapture};
use vize_l0::{Allocator, String};

/// [`super::custom_elements::compile_sfc_template_with_preflight_and_environment`]
/// for a template whose map the SFC assembly composes into the component's:
/// the same module and diagnostics, with a map that carries the links that
/// composition needs.
#[doc(hidden)]
#[expect(
    clippy::too_many_arguments,
    reason = "existing SFC inputs and invocation-local validation"
)]
pub fn compile_composed_sfc_template_with_preflight_and_environment<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<String>,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
    codegen_experimental_options: CodegenExperimentalOptions,
    no_slotted: bool,
    preflight: Option<&mut dyn vize_l4::targets::dom::artifact::SurfacePreflight>,
    capture: Option<&mut StageCapture>,
    development: bool,
) -> (Vec<CompilerError>, CodegenResultWithSections) {
    if let Some(capture) = capture {
        super::capture_options::record_dom_options(
            capture,
            &options,
            template_syntax,
            hoisted_scope_id.as_deref(),
            &custom_elements,
            &codegen_options,
            &codegen_experimental_options,
        );
        compile_template_for_sfc_composable(
            allocator,
            source,
            options,
            template_syntax,
            hoisted_scope_id,
            custom_elements,
            codegen_options,
            codegen_experimental_options,
            no_slotted,
            development,
            capture,
            preflight,
            true,
        )
    } else {
        compile_template_for_sfc_composable(
            allocator,
            source,
            options,
            template_syntax,
            hoisted_scope_id,
            custom_elements,
            codegen_options,
            codegen_experimental_options,
            no_slotted,
            development,
            &mut NoCapture,
            preflight,
            true,
        )
    }
}

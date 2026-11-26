//! The adapter entry point that compiles one SFC for several targets.

use super::SfcScriptOutputMode;
use super::root_match::prepare_root_patterned_template;
use crate::compile::targets::{TargetsCall, compile_sfc_targets_inner};
use crate::types::{
    SfcCompileExperimentalOptions, SfcCompileOptions, SfcCompileResult, SfcDescriptor, SfcError,
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};

/// What one target of a multi-target compile sets; every other option is the
/// call's.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SfcCompileTarget {
    /// `options.template.ssr`.
    pub ssr: bool,
    /// `options.vapor`.
    pub vapor: bool,
    pub script_output: SfcScriptOutputMode,
}

/// The results of a multi-target compile.
#[doc(hidden)]
#[derive(Debug, Default)]
pub struct SfcTargetsCompileResult {
    /// One result per target, in target order. Each equals the result of
    /// compiling that target on its own, except as `dependencies_shared` says.
    pub results: Vec<Result<SfcCompileResult, SfcError>>,
    /// Every target compiled, and every target compiled the first target's
    /// `dependencies`: a later result leaves them out and is empty.
    pub dependencies_shared: bool,
}

/// Compile `descriptor` for each of `targets`: `options` with the target's
/// `ssr` and `vapor`, assembled in the target's script output mode, against
/// the project selected by the host. The analysis no target choice affects
/// runs once for all of them.
#[doc(hidden)]
#[expect(
    clippy::too_many_arguments,
    reason = "adapter compile inputs and project authority"
)]
pub fn compile_sfc_for_adapter_targets_with_project(
    descriptor: &SfcDescriptor,
    options: &SfcCompileOptions,
    targets: &[SfcCompileTarget],
    template_syntax: TemplateSyntaxMode,
    custom_elements: &CustomElementMatcher,
    codegen_options: &CodegenOptions,
    experimental_options: &SfcCompileExperimentalOptions,
    nuxt_page_meta: bool,
    project: Option<&crate::script::SfcTypeProject>,
) -> SfcTargetsCompileResult {
    let refused = |error: SfcError| SfcTargetsCompileResult {
        results: targets.iter().map(|_| Err(error.clone())).collect(),
        dependencies_shared: false,
    };
    let compiler_options = options.template.compiler_options.as_ref();
    let descriptor = match prepare_root_patterned_template(
        descriptor,
        compiler_options.is_some_and(|options| options.experimental_patterned_template),
        compiler_options.is_some_and(|options| options.experimental_in_tag_comments),
        template_syntax,
    ) {
        Ok(descriptor) => descriptor,
        Err(error) => return refused(error),
    };
    let (descriptor, dependencies, pug_source_map, pug_warnings) =
        match super::pug::prepare_pug_template(&descriptor) {
            Ok(prepared) => prepared,
            Err(error) => return refused(error),
        };
    let mut compiled = compile_sfc_targets_inner(
        &descriptor,
        options,
        targets,
        &TargetsCall {
            template_syntax,
            custom_elements,
            codegen_options,
            experimental_options,
            nuxt_page_meta,
            type_project: project,
            pug_source_map: pug_source_map.as_ref(),
        },
    );
    // As a separate compile does: the template's includes come first, and its
    // warnings last.
    let shared = compiled.dependencies_shared;
    for (index, result) in compiled.results.iter_mut().enumerate() {
        let Ok(result) = result else {
            continue;
        };
        if !(shared && index > 0) && !dependencies.is_empty() {
            let mut all = dependencies.clone();
            all.append(&mut result.dependencies);
            result.dependencies = all;
        }
        result.warnings.extend(pug_warnings.iter().cloned());
    }
    compiled
}

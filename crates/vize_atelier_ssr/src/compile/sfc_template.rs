//! The SFC SSR template entry and its direct lane.

use vize_atelier_core::codegen::{ComposedRenderMap, PlainRenderMap, RenderMapMode};
use vize_atelier_core::{
    CompilerError,
    options::{CustomElementMatcher, TemplateSyntaxMode},
};
use vize_l0::Allocator;

use super::{SsrLane, compile_ssr_on_lane_with_runtime};
use crate::{
    SsrCodegenResult, SsrCompilerExperimentalOptions, SsrCompilerOptions, SsrSfcContext,
    l4::{self, SsrL4Request, SsrL4Selection},
};

/// Compile one SFC template block for SSR, returning what the SFC compiler
/// keeps: the diagnostics and the generated module. No template tree comes
/// back, which is what lets a compile skip the legacy parser.
///
/// `direct` asks for the direct lane: the template is emitted
/// straight from L1, L2 and the L4 plan, with no legacy parse and no legacy
/// transform, when those two would have passed it without a diagnostic. The
/// caller sets it only when [`sfc_direct_lane_enabled`] holds and the source
/// passes the SFC fast path's source test, which rules out the markup the
/// legacy parser reports on. Every other template takes the path of
/// [`crate::compile_ssr_with_sfc_context_and_codegen_options`].
#[doc(hidden)]
#[expect(
    clippy::too_many_arguments,
    reason = "SFC metadata, existing codegen options and the lane request"
)]
pub fn compile_ssr_sfc_template(
    allocator: &Allocator,
    source: &str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: SsrCompilerExperimentalOptions,
    context: SsrSfcContext,
    codegen_options: &vize_atelier_core::CodegenOptions,
    direct: bool,
) -> (Vec<CompilerError>, SsrCodegenResult) {
    let compile = if experimental_options.source_map {
        // The SFC assembly composes this map into the component's.
        sfc_template_with_map::<ComposedRenderMap>
    } else {
        sfc_template_with_map::<PlainRenderMap>
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

#[expect(
    clippy::too_many_arguments,
    reason = "SFC metadata, existing codegen options and the lane request"
)]
fn sfc_template_with_map<M: RenderMapMode>(
    allocator: &Allocator,
    source: &str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: SsrCompilerExperimentalOptions,
    context: SsrSfcContext,
    codegen_options: &vize_atelier_core::CodegenOptions,
    direct: bool,
) -> (Vec<CompilerError>, SsrCodegenResult) {
    #[cfg(feature = "legacy-differential")]
    let lane = crate::differential::production_lane();
    #[cfg(not(feature = "legacy-differential"))]
    let lane = SsrLane::Selected;
    let runtime_module_name = codegen_options.runtime_module_name.as_str();
    if direct
        && lane == SsrLane::Selected
        && let Some(result) = emit_on_direct_lane::<M>(
            allocator,
            source,
            &options,
            template_syntax,
            &custom_elements,
            &experimental_options,
            context,
            runtime_module_name,
        )
    {
        return (Vec::new(), result);
    }
    let (_, errors, result) = compile_ssr_on_lane_with_runtime::<M>(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        experimental_options,
        context,
        lane,
        runtime_module_name,
    );
    (errors, result)
}

/// Emit `source` on the direct lane, or `None` when the full path owns it.
///
/// Debug builds also compile the template on the full path and assert that
/// it reports nothing and emits the same module: the direct lane may only
/// take a compile the legacy parser and transform had nothing to say about.
#[expect(
    clippy::too_many_arguments,
    reason = "SFC metadata and the generated runtime"
)]
pub(crate) fn emit_on_direct_lane<M: RenderMapMode>(
    allocator: &Allocator,
    source: &str,
    options: &SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: &CustomElementMatcher,
    experimental_options: &SsrCompilerExperimentalOptions,
    context: SsrSfcContext,
    runtime_module_name: &str,
) -> Option<SsrCodegenResult> {
    let selection = l4::select_ssr_lane_direct::<M>(
        allocator,
        source,
        &SsrL4Request {
            options,
            experimental: experimental_options,
            context,
            runtime_module_name,
            template_syntax,
            has_custom_elements: !custom_elements.is_empty(),
            frozen: None,
        },
    );
    let SsrL4Selection::Emitted(result) = selection else {
        return None;
    };
    #[cfg(feature = "legacy-differential")]
    crate::differential::record_emitted::<M>();
    #[cfg(debug_assertions)]
    {
        let full_path = || {
            let oracle = Allocator::new();
            let (_, errors, full) = compile_ssr_on_lane_with_runtime::<M>(
                &oracle,
                source,
                options.clone(),
                template_syntax,
                custom_elements.clone(),
                experimental_options.clone(),
                context,
                SsrLane::Selected,
                runtime_module_name,
            );
            (errors, full)
        };
        #[cfg(feature = "legacy-differential")]
        let (errors, full) =
            crate::differential::unrecorded::<M, _>(|| l4::as_oracle_run::<M, _>(full_path));
        #[cfg(not(feature = "legacy-differential"))]
        let (errors, full) = l4::as_oracle_run::<M, _>(full_path);
        debug_assert!(
            errors.is_empty(),
            "the direct SSR lane emitted a template the full path diagnoses: {errors:?}\n{source}"
        );
        debug_assert!(
            full.code == result.code && full.preamble == result.preamble && full.map == result.map,
            "the direct SSR lane and the full path emitted different modules for\n{source}"
        );
    }
    Some(result)
}

/// The rollout switch of the SSR direct lane, read once per process:
/// `VIZE_SSR_DIRECT_LANE=1`. Off by default until the corpora have run with
/// it on.
#[doc(hidden)]
#[must_use]
pub fn sfc_direct_lane_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED
        .get_or_init(|| std::env::var_os("VIZE_SSR_DIRECT_LANE").is_some_and(|value| value == "1"))
}

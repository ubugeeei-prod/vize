//! SSR compile lane and its optional native stage observation.

use super::{
    Allocator, CaptureOutcome, CaptureSink, CompilerError, CustomElementMatcher, ErrorCode, Level,
    NoCapture, RootNode, SsrCodegenContext, SsrCodegenResult, SsrCompilerExperimentalOptions,
    SsrCompilerOptions, SsrL4Request, SsrL4Selection, String, TemplateSyntaxMode, cstr, l4,
    parse_with_options_custom_elements_and_template_syntax, profile,
    transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id,
};

pub(super) fn compile_ssr_inner<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: SsrCompilerExperimentalOptions,
    slotted: bool,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_inner_captured(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        experimental_options,
        slotted,
        &mut NoCapture,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "SFC options plus an opt-in capture sink"
)]
pub(super) fn compile_ssr_inner_captured<'a, C: CaptureSink>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: SsrCompilerExperimentalOptions,
    slotted: bool,
    capture: &mut C,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    #[cfg(feature = "legacy-differential")]
    let lane = crate::differential::production_lane();
    #[cfg(not(feature = "legacy-differential"))]
    let lane = SsrLane::Selected;
    compile_ssr_on_lane_captured(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        experimental_options,
        slotted,
        lane,
        capture,
    )
}

/// Which emitter owns a compile. Production always asks the L4 selector;
/// the differential battery pins the legacy walker on the same input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SsrLane {
    Selected,
    #[cfg(any(test, feature = "legacy-differential"))]
    LegacyOnly,
}

#[expect(
    clippy::too_many_arguments,
    reason = "private SFC metadata is separate from public options"
)]
#[cfg(any(test, feature = "legacy-differential"))]
pub(crate) fn compile_ssr_on_lane<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: SsrCompilerExperimentalOptions,
    slotted: bool,
    lane: SsrLane,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    compile_ssr_on_lane_captured(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        experimental_options,
        slotted,
        lane,
        &mut NoCapture,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "SFC metadata, lane and opt-in capture"
)]
fn compile_ssr_on_lane_captured<'a, C: CaptureSink>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SsrCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: SsrCompilerExperimentalOptions,
    slotted: bool,
    lane: SsrLane,
    capture: &mut C,
) -> (RootNode<'a>, Vec<CompilerError>, SsrCodegenResult) {
    let codegen_options = options.clone();
    let parser_opts = crate::stage_options::parser_options(&options);

    let (mut root, errors) = profile!(
        "atelier.ssr.template.parse",
        parse_with_options_custom_elements_and_template_syntax(
            allocator,
            source,
            parser_opts,
            custom_elements.clone(),
            template_syntax,
        )
    );
    if errors.iter().any(|e| !e.is_recoverable()) {
        capture.finish(|| CaptureOutcome::Rejected(String::from("SSR parser rejected template")));
        return (
            root,
            errors.to_vec(),
            SsrCodegenResult {
                code: String::default(),
                preamble: String::default(),
                map: None,
            },
        );
    }

    let selection = match lane {
        SsrLane::Selected => l4::select_ssr_lane_captured(
            allocator,
            source,
            &SsrL4Request {
                options: &options,
                experimental: &experimental_options,
                slotted,
                template_syntax,
                has_custom_elements: !custom_elements.is_empty(),
            },
            capture,
        ),
        #[cfg(any(test, feature = "legacy-differential"))]
        SsrLane::LegacyOnly => SsrL4Selection::Legacy(l4::LegacyReason::Options),
    };
    #[cfg(feature = "legacy-differential")]
    if lane == SsrLane::Selected {
        crate::differential::record_verdict(&selection);
    }

    let transform_opts = crate::stage_options::transform_options(&codegen_options);
    let transform_errors = profile!(
        "atelier.ssr.template.transform",
        transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id(
            allocator,
            &mut root,
            transform_opts,
            options.croquis.map(|c| allocator.alloc_owned(*c)),
            custom_elements,
            template_syntax.is_quirks(),
            None,
        )
    );

    let mut errors = errors.to_vec();
    errors.extend(transform_errors);
    let codegen_result = match selection {
        SsrL4Selection::Emitted(result) => {
            if !result.code.is_empty() {
                // Match SFC render-chunk assembly. Later SFC script
                // composition is a separate product step.
                capture.page(Level::L4, "emit", || {
                    let mut module =
                        String::with_capacity(result.preamble.len() + result.code.len() + 2);
                    module.push_str(&result.preamble);
                    module.push('\n');
                    module.push_str(&result.code);
                    module.push('\n');
                    module
                });
            }
            capture.finish(|| {
                if result.code.is_empty() {
                    CaptureOutcome::Rejected(String::from("SSR emitter returned empty render code"))
                } else {
                    CaptureOutcome::Accepted
                }
            });
            result
        }
        other => {
            let legacy_reason = match &other {
                SsrL4Selection::Legacy(reason) => Some(*reason),
                SsrL4Selection::Rejected(_) => None,
                SsrL4Selection::Emitted(_) => None,
            };
            match other {
                SsrL4Selection::Rejected(diagnostics) => {
                    errors.extend(diagnostics.into_iter().map(|diagnostic| {
                        CompilerError::with_message(ErrorCode::ExtendPoint, diagnostic, None)
                    }));
                }
                SsrL4Selection::Legacy(_) => {}
                SsrL4Selection::Emitted(_) => {}
            }
            let mut codegen_ctx = SsrCodegenContext::new_with_experimental_options(
                allocator,
                &codegen_options,
                source,
                experimental_options,
            );
            codegen_ctx.slotted = slotted;
            let result = profile!("atelier.ssr.template.codegen", codegen_ctx.generate(&root));
            capture.finish(|| {
                let reason = legacy_reason
                    .map(|reason| cstr!("{reason:?}"))
                    .unwrap_or_else(|| String::from("native SSR bridge rejected"));
                if result.code.is_empty() {
                    CaptureOutcome::Rejected(reason)
                } else {
                    CaptureOutcome::Legacy(reason)
                }
            });
            result
        }
    };

    (root, errors, codegen_result)
}

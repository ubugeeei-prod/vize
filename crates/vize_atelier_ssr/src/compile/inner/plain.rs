//! Ordinary SSR compile path with the original no-sink call signatures.

use super::{
    Allocator, CompilerError, CustomElementMatcher, ErrorCode, RootNode, SsrCodegenContext,
    SsrCodegenResult, SsrCompilerExperimentalOptions, SsrCompilerOptions, SsrL4Request,
    SsrL4Selection, SsrLane, String, TemplateSyntaxMode, l4,
    parse_with_options_custom_elements_and_template_syntax, profile,
    transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id,
};

#[expect(
    clippy::too_many_arguments,
    reason = "private SFC metadata is separate from public options"
)]
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
        SsrLane::Selected => l4::select_ssr_lane(
            allocator,
            source,
            &SsrL4Request {
                options: &options,
                experimental: &experimental_options,
                slotted,
                template_syntax,
                has_custom_elements: !custom_elements.is_empty(),
            },
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
        SsrL4Selection::Emitted(result) => result,
        other => {
            if let SsrL4Selection::Rejected(diagnostics) = other {
                errors.extend(diagnostics.into_iter().map(|diagnostic| {
                    CompilerError::with_message(ErrorCode::ExtendPoint, diagnostic, None)
                }));
            }
            let mut codegen_ctx = SsrCodegenContext::new_with_experimental_options(
                allocator,
                &codegen_options,
                source,
                experimental_options,
            );
            codegen_ctx.slotted = slotted;
            profile!("atelier.ssr.template.codegen", codegen_ctx.generate(&root))
        }
    };

    (root, errors, codegen_result)
}

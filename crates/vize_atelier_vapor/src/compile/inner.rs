//! Vapor compile lane and its optional native stage observation.

use super::{
    Allocator, CaptureOutcome, CaptureSink, CompilerError, CustomElementMatcher, Level, NoCapture,
    String, TemplateSyntaxMode, TransformOptions, VaporCompileResult,
    VaporCompilerExperimentalOptions, VaporCompilerOptions, VaporL3BridgeOptions,
    VaporL3BridgeStatus, VaporSourceSpans, cstr, l3, native,
    parse_with_options_custom_elements_and_template_syntax, parser_options,
    transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id, vapor_lower,
};

pub(super) fn compile_vapor_inner<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: VaporCompilerExperimentalOptions,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    compile_vapor_inner_scoped(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        experimental_options,
        None,
    )
}

pub(super) fn compile_vapor_inner_scoped<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: VaporCompilerExperimentalOptions,
    scope_id: Option<&str>,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    compile_vapor_inner_scoped_captured(
        allocator,
        source,
        options,
        template_syntax,
        custom_elements,
        experimental_options,
        scope_id,
        &mut NoCapture,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "SFC options plus an opt-in capture sink"
)]
pub(super) fn compile_vapor_inner_scoped_captured<'a, C: CaptureSink>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: VaporCompilerExperimentalOptions,
    scope_id: Option<&str>,
    capture: &mut C,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    vize_carton::ensure_sufficient_stack(|| {
        compile_vapor_inner_with_stack(
            allocator,
            source,
            options,
            template_syntax,
            custom_elements,
            experimental_options,
            scope_id,
            capture,
        )
    })
}

#[expect(
    clippy::too_many_arguments,
    reason = "SFC options plus an opt-in capture sink"
)]
fn compile_vapor_inner_with_stack<'a, C: CaptureSink>(
    allocator: &'a Allocator,
    source: &'a str,
    options: VaporCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    experimental_options: VaporCompilerExperimentalOptions,
    scope_id: Option<&str>,
    capture: &mut C,
) -> (VaporCompileResult, std::vec::Vec<CompilerError>) {
    #[cfg(feature = "davinci-benchmark")]
    let options = benchmark::apply(options);
    capture.effective_option("template_syntax", || cstr!("{template_syntax:?}"));
    capture.effective_option("ssr", || cstr!("{}", options.ssr));
    capture.effective_option("inline", || cstr!("{}", options.inline));
    capture.effective_option("custom_renderer", || cstr!("{}", options.custom_renderer));
    capture.effective_option("prefix_identifiers", || {
        cstr!("{}", options.prefix_identifiers)
    });
    capture.effective_option("scope_id", || {
        scope_id
            .map(String::from)
            .unwrap_or_else(|| String::from("<none>"))
    });
    capture.effective_option("source_map", || {
        cstr!("{}", experimental_options.source_map)
    });
    capture.effective_option("custom_elements", || {
        cstr!("{}", !custom_elements.is_empty())
    });
    // The native lane parses the source through L1 itself. It admits only
    // sources the legacy parser reports nothing for (L1 keeps the tokenizer's
    // codes and L2 refuses every recovery rule; `s3/tests/parser_agreement.rs`
    // pins this over the fixture corpus and its malformed variants), so an
    // admitted source never builds the legacy tree it would discard.
    let l3_bridge_status = l3::lower_source_for_vapor_captured(
        allocator,
        source,
        VaporL3BridgeOptions {
            ssr: options.ssr,
            custom_renderer: options.custom_renderer,
            experimental_in_tag_comments: options.experimental_in_tag_comments,
            experimental_patterned_template: options.experimental_patterned_template,
            template_syntax,
            has_custom_elements: !custom_elements.is_empty(),
            // Without prefixing, the retained lane keeps expression text as
            // authored and binding metadata only steers the shared generator.
            prefixed_binding_metadata: options.binding_metadata.is_some()
                && options.prefix_identifiers,
            // L3 lowering currently implements only Vue's default condense
            // mode; preserve must use the parser-backed retained lane.
            retained_lane: options.davinci_retained_lane
                || vize_atelier_core::parser::current_whitespace_strategy(
                    vize_atelier_core::WhitespaceStrategy::Condense,
                ) == vize_atelier_core::WhitespaceStrategy::Preserve
                || vize_atelier_core::parser::current_legacy_line_breaks(),
            inline: options.inline,
        },
        capture,
    );
    // A map-requesting compile also carries the authored anchors generation
    // needs beyond the IR (Davinci P3-9).
    let source_map = experimental_options.source_map;
    let emit = |ir: &crate::ir::RootIRNode<'a>, errors, spans: Option<&VaporSourceSpans>| {
        generate(ir, &options, &experimental_options, errors, spans)
    };
    if let VaporL3BridgeStatus::Accepted(artifact) = l3_bridge_status {
        debug_assert!(
            parse_with_options_custom_elements_and_template_syntax(
                allocator,
                source,
                parser_options(&options),
                custom_elements.clone(),
                template_syntax,
            )
            .1
            .is_empty(),
            "the native Vapor lane admitted a source the legacy parser diagnoses"
        );
        let result = native::emit_accepted(
            allocator,
            source,
            artifact,
            scope_id,
            source_map,
            |ir, spans| emit(ir, Vec::new(), spans),
        );
        if result.error_messages.is_empty() && !result.code.is_empty() {
            // This is the Vapor backend module before the SFC adapter's
            // import/render rewrite, not the whole SFC script module.
            capture.page(Level::L4, "emit", || result.code.clone());
        }
        capture.finish(|| {
            if result.error_messages.is_empty() && !result.code.is_empty() {
                CaptureOutcome::Accepted
            } else {
                CaptureOutcome::Rejected(result.error_messages.first().cloned().unwrap_or_default())
            }
        });
        return (result, std::vec::Vec::new());
    }

    let (mut root, errors) = parse_with_options_custom_elements_and_template_syntax(
        allocator,
        source,
        parser_options(&options),
        custom_elements.clone(),
        template_syntax,
    );
    let parser_diagnostics = errors.to_vec();

    let fatal: std::vec::Vec<_> = errors.iter().filter(|e| !e.is_recoverable()).collect();
    if !fatal.is_empty() {
        capture.finish(|| CaptureOutcome::Rejected(String::from("Vapor parser rejected template")));
        return (
            VaporCompileResult {
                code: String::default(),
                templates: Vec::new(),
                map: None,
                error_messages: fatal.iter().map(|e| e.message.clone()).collect(),
            },
            parser_diagnostics,
        );
    }

    // A diagnosed source keeps the legacy lane whatever the bridge concluded.
    let l3_bridge_status = if parser_diagnostics.is_empty() {
        l3_bridge_status
    } else {
        VaporL3BridgeStatus::Legacy(l3::LegacyReason::SurfaceSemantics)
    };
    let legacy_reason = match &l3_bridge_status {
        VaporL3BridgeStatus::Legacy(reason) => Some(*reason),
        VaporL3BridgeStatus::Rejected(_) | VaporL3BridgeStatus::Accepted(_) => None,
    };
    l3::record_selection(&l3_bridge_status);
    match l3_bridge_status {
        // Every accepted artifact returned above, including emission failures.
        VaporL3BridgeStatus::Accepted(_) => {}
        VaporL3BridgeStatus::Rejected(error_messages) => {
            capture.finish(|| {
                CaptureOutcome::Rejected(error_messages.first().cloned().unwrap_or_default())
            });
            return (
                VaporCompileResult {
                    code: String::default(),
                    templates: Vec::new(),
                    map: None,
                    error_messages,
                },
                parser_diagnostics,
            );
        }
        VaporL3BridgeStatus::Legacy(_) => {}
    }

    // The explicitly selected legacy route retains its complete transforms.
    let binding_metadata = options.binding_metadata.clone();
    let transform_opts = TransformOptions {
        prefix_identifiers: options.prefix_identifiers,
        ssr: options.ssr,
        binding_metadata: binding_metadata.clone(),
        inline: options.inline,
        vapor: true,
        custom_renderer: options.custom_renderer,
        experimental_patterned_template: options.experimental_patterned_template,
        ..Default::default()
    };
    let transform_errors =
        transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id(
            allocator,
            &mut root,
            transform_opts,
            None,
            custom_elements,
            template_syntax.is_quirks(),
            None,
        );
    let fatal: Vec<_> = transform_errors
        .iter()
        .filter(|error| !error.is_recoverable())
        .collect();
    if !fatal.is_empty() {
        capture
            .finish(|| CaptureOutcome::Rejected(String::from("Vapor transform rejected template")));
        let mut diagnostics = parser_diagnostics;
        diagnostics.extend(transform_errors.iter().cloned());
        return (
            VaporCompileResult {
                code: String::default(),
                templates: Vec::new(),
                map: None,
                error_messages: fatal.iter().map(|error| error.message.clone()).collect(),
            },
            diagnostics,
        );
    }

    // Lower to Vapor IR.
    let (ir, transform_diagnostics, template_spans) =
        vapor_lower::transform_to_ir_with_spans(allocator, &root, source, scope_id, source_map);
    let spans = template_spans.map(|templates| VaporSourceSpans::collect(&root, templates));
    let result = emit(&ir, transform_diagnostics, spans.as_ref());
    capture.finish(|| {
        if result.code.is_empty() || !result.error_messages.is_empty() {
            CaptureOutcome::Rejected(result.error_messages.first().cloned().unwrap_or_default())
        } else {
            CaptureOutcome::Legacy(cstr!(
                "{:?}",
                legacy_reason.unwrap_or(l3::LegacyReason::Options)
            ))
        }
    });
    (result, parser_diagnostics)
}

fn generate(
    ir: &crate::ir::RootIRNode<'_>,
    options: &VaporCompilerOptions,
    experimental_options: &VaporCompilerExperimentalOptions,
    error_messages: Vec<String>,
    spans: Option<&VaporSourceSpans>,
) -> VaporCompileResult {
    let result = crate::generate::generate_vapor_with_spans(
        ir,
        options.binding_metadata.as_ref(),
        crate::generate::VaporGenerateOptions::default(),
        crate::generate::VaporGenerateExperimentalOptions {
            component_name: experimental_options.component_name.as_deref(),
            self_component: experimental_options.self_component,
            source_map: experimental_options.source_map,
            source_map_filename: experimental_options.source_map_filename.as_deref(),
        },
        spans,
    );

    VaporCompileResult {
        code: result.code,
        templates: result.templates,
        map: result.map,
        error_messages,
    }
}

//! Canon checks over frontend-owned descriptor, Relief, Croquis, and Flow products.

use vize_atelier_core::atelier_output::AtelierFallback;
use vize_atelier_sfc::{SfcDescriptorArtifact, croquis::SfcCroquisOptions};
use vize_carton::{Bump, cstr, profiler::global_profiler};
use vize_croquis::CroquisDocument;
use vize_flow::FlowGraph;
use vize_relief::ReliefArtifact;

use crate::{
    options_api_setup_spread, script_parse::collect_script_parse_diagnostics,
    virtual_ts::generate_virtual_ts_with_offsets_legacy_vue2,
};

use super::{
    SfcTypeCheckRequest,
    analysis::{SfcTypeCheckResult, SfcTypeDiagnostic, SfcTypeSeverity},
    checks::{
        check_emits_typing, check_fallthrough_attrs, check_invalid_exports, check_props_typing,
        check_reactivity, check_setup_context, check_template_bindings,
    },
    virtual_ts::generate_virtual_ts_with_scopes,
};

pub(super) fn type_check_from_artifacts(
    request: &SfcTypeCheckRequest,
    descriptor: &SfcDescriptorArtifact,
    relief: Option<&ReliefArtifact>,
    document: &CroquisDocument,
    flow: &FlowGraph,
) -> SfcTypeCheckResult {
    #[cfg(not(target_arch = "wasm32"))]
    let start_time = std::time::Instant::now();
    let options = &request.options;
    let mut result = SfcTypeCheckResult::empty();
    let descriptor = match descriptor.as_result() {
        Ok(descriptor) => descriptor,
        Err(error) => {
            result.add_diagnostic(SfcTypeDiagnostic {
                severity: SfcTypeSeverity::Error,
                message: cstr!("Failed to parse SFC: {}", error.message),
                start: 0,
                end: 0,
                code: Some("parse-error".into()),
                help: None,
                related: Vec::new(),
            });
            record_virtual_ts_projection(options.include_virtual_ts, false);
            return result;
        }
    };

    let has_script_parse_errors = collect_script_diagnostics(descriptor, &mut result);
    let template_offset = descriptor
        .template
        .as_ref()
        .map_or(0, |template| template.loc.start as u32);
    let has_template_parse_errors = collect_template_diagnostics(
        descriptor.template.is_some(),
        template_offset,
        relief,
        &mut result,
    );
    let allocator = Bump::new();
    let template_ast = relief
        .filter(|syntax| !syntax.has_fatal_diagnostics())
        .map(|syntax| syntax.snapshot().materialize(&allocator));
    let (script_content, script_offset) = vize_atelier_sfc::croquis::script_content_for_descriptor(
        descriptor,
        SfcCroquisOptions::full(),
    );
    let summary = document.analysis();
    let mode = request.mode;
    let options_api = matches!(mode, vize_atelier_sfc::SfcCroquisMode::OptionsApi);
    let legacy_vue2 = matches!(mode, vize_atelier_sfc::SfcCroquisMode::LegacyVue2);
    run_checks(
        summary,
        script_content.as_deref(),
        script_offset,
        template_offset,
        has_script_parse_errors,
        has_template_parse_errors,
        options_api || legacy_vue2,
        options,
        &mut result,
    );

    let virtual_ts_available = !has_template_parse_errors && !has_script_parse_errors;
    if options.include_virtual_ts && virtual_ts_available {
        result.virtual_ts = Some(generate_virtual_ts(
            summary,
            script_content.as_deref(),
            script_offset,
            template_ast.as_ref(),
            template_offset,
            options_api,
            legacy_vue2,
        ));
    }
    record_virtual_ts_projection(options.include_virtual_ts, virtual_ts_available);
    let _mapped_flow_nodes = flow.nodes().len();
    #[cfg(not(target_arch = "wasm32"))]
    {
        result.analysis_time_ms = Some(start_time.elapsed().as_secs_f64() * 1000.0);
    }
    result
}

fn collect_script_diagnostics(
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
    result: &mut SfcTypeCheckResult,
) -> bool {
    let mut found = false;
    for script in [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
        .into_iter()
        .flatten()
    {
        let diagnostics = collect_script_parse_diagnostics(
            &script.content,
            script.loc.start as u32,
            script.lang.as_deref(),
        );
        found |= !diagnostics.is_empty();
        for diagnostic in diagnostics {
            result.add_diagnostic(SfcTypeDiagnostic {
                severity: SfcTypeSeverity::Error,
                message: cstr!("Script parse error: {}", diagnostic.message),
                start: diagnostic.start,
                end: diagnostic.end,
                code: Some("script-parse-error".into()),
                help: None,
                related: Vec::new(),
            });
        }
    }
    found
}

fn collect_template_diagnostics(
    has_template: bool,
    template_offset: u32,
    relief: Option<&ReliefArtifact>,
    result: &mut SfcTypeCheckResult,
) -> bool {
    if !has_template {
        return false;
    }
    let Some(relief) = relief else {
        return true;
    };
    let mut found = false;
    for error in relief
        .parse_diagnostics()
        .iter()
        .filter(|error| !error.is_recoverable())
    {
        found = true;
        let (start, end) = error
            .loc
            .as_ref()
            .map_or((template_offset, template_offset), |loc| {
                (
                    template_offset.saturating_add(loc.start.offset),
                    template_offset.saturating_add(loc.end.offset),
                )
            });
        result.add_diagnostic(SfcTypeDiagnostic {
            severity: SfcTypeSeverity::Error,
            message: cstr!("Template parse error: {}", error.message),
            start,
            end: end.max(start.saturating_add(1)),
            code: Some("template-parse-error".into()),
            help: None,
            related: Vec::new(),
        });
    }
    found
}

#[allow(clippy::too_many_arguments)]
fn run_checks(
    summary: &vize_croquis::Croquis,
    script_content: Option<&str>,
    script_offset: u32,
    template_offset: u32,
    script_errors: bool,
    template_errors: bool,
    options_api: bool,
    options: &super::SfcTypeCheckOptions,
    result: &mut SfcTypeCheckResult,
) {
    if options.check_props && !script_errors {
        check_props_typing(summary, script_offset, result, options.strict);
    }
    if options.check_emits && !script_errors {
        check_emits_typing(summary, script_offset, result, options.strict);
    }
    if options.check_template_bindings && !template_errors && !script_errors {
        let suppress = options_api_setup_spread::suppresses_template_undefined_refs(
            options_api,
            script_content,
        );
        check_template_bindings(summary, template_offset, result, options.strict, suppress);
    }
    if options.check_reactivity && !script_errors {
        check_reactivity(summary, script_offset, result, options.strict);
    }
    if options.check_setup_context && !script_errors {
        check_setup_context(summary, script_offset, result);
    }
    if options.check_invalid_exports && !script_errors {
        check_invalid_exports(summary, script_offset, result);
    }
    if options.check_fallthrough_attrs {
        check_fallthrough_attrs(summary, result, options.strict);
    }
}

#[allow(clippy::too_many_arguments)]
fn generate_virtual_ts(
    summary: &vize_croquis::Croquis,
    script_content: Option<&str>,
    script_offset: u32,
    template_ast: Option<&vize_relief::RootNode<'_>>,
    template_offset: u32,
    options_api: bool,
    legacy_vue2: bool,
) -> vize_carton::String {
    let options = &crate::virtual_ts::VirtualTsOptions::default();
    if legacy_vue2 {
        generate_virtual_ts_with_offsets_legacy_vue2(
            summary,
            script_content,
            template_ast,
            script_offset,
            template_offset,
            options,
        )
        .code
    } else if options_api {
        crate::virtual_ts::generate_virtual_ts_with_offsets_options_api(
            summary,
            script_content,
            template_ast,
            script_offset,
            template_offset,
            options,
        )
        .code
    } else {
        generate_virtual_ts_with_scopes(
            summary,
            script_content,
            script_offset,
            template_ast,
            template_offset,
        )
    }
}

fn record_virtual_ts_projection(requested: bool, available: bool) {
    if requested && !available {
        global_profiler().record_counter(AtelierFallback::VirtualTsSkipped.profile_counter(), 1);
    }
}

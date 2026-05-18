//! Main SFC type checking runner.
//!
//! Orchestrates parsing, analysis, and virtual TypeScript generation
//! for a Vue Single File Component.

use vize_carton::Bump;
use vize_carton::cstr;

use crate::script_parse::collect_script_parse_diagnostics;

use super::{
    analysis::{SfcTypeCheckOptions, SfcTypeCheckResult, SfcTypeDiagnostic, SfcTypeSeverity},
    checks::{
        check_emits_typing, check_fallthrough_attrs, check_invalid_exports, check_props_typing,
        check_reactivity, check_setup_context, check_template_bindings,
    },
    virtual_ts::generate_virtual_ts_with_scopes,
};

/// Perform type checking on a Vue SFC.
///
/// This performs AST-based type analysis using croquis for semantic analysis.
/// It checks:
/// - Props typing (defineProps)
/// - Emits typing (defineEmits)
/// - Template binding references
///
/// For full TypeScript type checking with Corsa, use `TypeCheckService`.
pub fn type_check_sfc(source: &str, options: &SfcTypeCheckOptions) -> SfcTypeCheckResult {
    use vize_atelier_core::parser::parse;
    use vize_atelier_sfc::{
        SfcParseOptions,
        croquis::{SfcCroquisOptions, analyze_sfc_descriptor_with_context},
        parse_sfc,
    };

    // Use Instant for timing on native, skip on WASM
    #[cfg(not(target_arch = "wasm32"))]
    let start_time = std::time::Instant::now();

    let mut result = SfcTypeCheckResult::empty();

    // Parse SFC
    let parse_opts = SfcParseOptions {
        filename: options.filename.clone(),
        ..Default::default()
    };

    let descriptor = match parse_sfc(source, parse_opts) {
        Ok(d) => d,
        Err(e) => {
            result.add_diagnostic(SfcTypeDiagnostic {
                severity: SfcTypeSeverity::Error,
                message: cstr!("Failed to parse SFC: {}", e.message),
                start: 0,
                end: 0,
                code: Some("parse-error".into()),
                help: None,
                related: Vec::new(),
            });
            return result;
        }
    };

    // Create allocator for template parsing
    let allocator = Bump::new();

    let mut has_script_parse_errors = false;
    if let Some(ref script) = descriptor.script {
        let script_diagnostics =
            collect_script_parse_diagnostics(&script.content, script.loc.start as u32);
        if !script_diagnostics.is_empty() {
            has_script_parse_errors = true;
            add_script_parse_diagnostics(script_diagnostics, &mut result);
        }
    }
    if let Some(ref script_setup) = descriptor.script_setup {
        let script_diagnostics =
            collect_script_parse_diagnostics(&script_setup.content, script_setup.loc.start as u32);
        if !script_diagnostics.is_empty() {
            has_script_parse_errors = true;
            add_script_parse_diagnostics(script_diagnostics, &mut result);
        }
    }

    // Analyze template and get AST
    let mut has_template_parse_errors = false;
    let (template_offset, template_ast) = if let Some(ref template) = descriptor.template {
        let template_offset = template.loc.start as u32;
        let (root, errors) = parse(&allocator, &template.content);
        if errors.is_empty() {
            (template_offset, Some(root))
        } else {
            has_template_parse_errors = true;
            for error in errors {
                let (start, end) = error
                    .loc
                    .as_ref()
                    .map(|loc| {
                        (
                            template_offset + loc.start.offset,
                            template_offset + loc.end.offset,
                        )
                    })
                    .unwrap_or((template_offset, template_offset));
                result.add_diagnostic(SfcTypeDiagnostic {
                    severity: SfcTypeSeverity::Error,
                    message: cstr!("Template parse error: {}", error.message),
                    start,
                    end: end.max(start + 1),
                    code: Some("template-parse-error".into()),
                    help: None,
                    related: Vec::new(),
                });
            }
            (template_offset, None)
        }
    } else {
        (0, None)
    };

    let analysis = analyze_sfc_descriptor_with_context(
        &descriptor,
        template_ast.as_ref(),
        SfcCroquisOptions::full(),
    );
    let script_content = analysis.script_content;
    let script_offset = analysis.script_offset;
    let summary = analysis.croquis;

    // Check props typing
    if options.check_props && !has_script_parse_errors {
        check_props_typing(&summary, script_offset, &mut result, options.strict);
    }

    // Check emits typing
    if options.check_emits && !has_script_parse_errors {
        check_emits_typing(&summary, script_offset, &mut result, options.strict);
    }

    // Check template bindings
    if options.check_template_bindings && !has_template_parse_errors && !has_script_parse_errors {
        check_template_bindings(&summary, template_offset, &mut result, options.strict);
    }

    // Check reactivity loss
    if options.check_reactivity && !has_script_parse_errors {
        check_reactivity(&summary, script_offset, &mut result, options.strict);
    }

    // Check setup context violations
    if options.check_setup_context && !has_script_parse_errors {
        check_setup_context(&summary, script_offset, &mut result);
    }

    // Check invalid exports in <script setup>
    if options.check_invalid_exports && !has_script_parse_errors {
        check_invalid_exports(&summary, script_offset, &mut result);
    }

    // Check fallthrough attrs
    if options.check_fallthrough_attrs {
        check_fallthrough_attrs(&summary, &mut result, options.strict);
    }

    // Generate virtual TypeScript with scope information if requested
    if options.include_virtual_ts && !has_template_parse_errors && !has_script_parse_errors {
        result.virtual_ts = Some(generate_virtual_ts_with_scopes(
            &summary,
            script_content.as_deref(),
            script_offset,
            template_ast.as_ref(),
            template_offset,
        ));
    }

    // Record analysis time on native only
    #[cfg(not(target_arch = "wasm32"))]
    {
        result.analysis_time_ms = Some(start_time.elapsed().as_secs_f64() * 1000.0);
    }

    result
}

fn add_script_parse_diagnostics(
    diagnostics: Vec<crate::script_parse::ScriptParseDiagnostic>,
    result: &mut SfcTypeCheckResult,
) {
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

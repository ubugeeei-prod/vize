//! Main SFC type checking runner.
//!
//! Orchestrates parsing, analysis, and virtual TypeScript generation
//! for a Vue Single File Component.

use vize_carton::cstr;
use vize_carton::Bump;

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
    use vize_atelier_sfc::{parse_sfc, SfcParseOptions};
    use vize_croquis::{Analyzer, AnalyzerOptions};

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

    // Get script content for virtual TS generation
    let script_content = descriptor
        .script_setup
        .as_ref()
        .map(|s| s.content.as_ref())
        .or_else(|| descriptor.script.as_ref().map(|s| s.content.as_ref()));

    // Create allocator for template parsing
    let allocator = Bump::new();

    // Create analyzer with full options
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());

    // Analyze script and get offset
    let script_offset: u32 = if let Some(ref script_setup) = descriptor.script_setup {
        analyzer.analyze_script_setup(&script_setup.content);
        script_setup.loc.start as u32
    } else if let Some(ref script) = descriptor.script {
        analyzer.analyze_script_plain(&script.content);
        script.loc.start as u32
    } else {
        0
    };

    // Analyze template and get AST
    let (template_offset, template_ast) = if let Some(ref template) = descriptor.template {
        let (root, _errors) = parse(&allocator, &template.content);
        analyzer.analyze_template(&root);
        (template.loc.start as u32, Some(root))
    } else {
        (0, None)
    };

    // Get analysis summary with scopes
    let summary = analyzer.finish();

    // Check props typing
    if options.check_props {
        check_props_typing(&summary, script_offset, &mut result, options.strict);
    }

    // Check emits typing
    if options.check_emits {
        check_emits_typing(&summary, script_offset, &mut result, options.strict);
    }

    // Check template bindings
    if options.check_template_bindings {
        check_template_bindings(&summary, template_offset, &mut result, options.strict);
    }

    // Check reactivity loss
    if options.check_reactivity {
        check_reactivity(&summary, script_offset, &mut result, options.strict);
    }

    // Check setup context violations
    if options.check_setup_context {
        check_setup_context(&summary, script_offset, &mut result);
    }

    // Check invalid exports in <script setup>
    if options.check_invalid_exports {
        check_invalid_exports(&summary, script_offset, &mut result);
    }

    // Check fallthrough attrs
    if options.check_fallthrough_attrs {
        check_fallthrough_attrs(&summary, &mut result, options.strict);
    }

    // Generate virtual TypeScript with scope information if requested
    if options.include_virtual_ts {
        result.virtual_ts = Some(generate_virtual_ts_with_scopes(
            &summary,
            script_content,
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

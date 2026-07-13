use vize_atelier_sfc::SfcCroquisMode;
use vize_atlas::Compilation;
use vize_carton::cstr;

use super::{
    SfcTypeCheckOptions, SfcTypeCheckProduct, SfcTypeCheckRequest, SfcTypeCheckResult,
    SfcTypeDiagnostic, SfcTypeSeverity, install_sfc_typecheck_request,
    register_sfc_typecheck_provider,
};

pub fn type_check_sfc(source: &str, options: &SfcTypeCheckOptions) -> SfcTypeCheckResult {
    type_check_sfc_impl(source, options, false, false)
}

/// Perform type checking on a Vue SFC with Vue 3 Options API binding resolution
/// enabled (opt-in, standard build — no `legacy` feature required).
pub fn type_check_sfc_with_options_api(
    source: &str,
    options: &SfcTypeCheckOptions,
) -> SfcTypeCheckResult {
    type_check_sfc_impl(source, options, true, false)
}

/// Perform type checking on a Vue SFC with Vue 2.7 / Nuxt 2 compatibility enabled.
pub fn type_check_sfc_with_legacy_vue2(
    source: &str,
    options: &SfcTypeCheckOptions,
) -> SfcTypeCheckResult {
    type_check_sfc_impl(source, options, false, true)
}

fn type_check_sfc_impl(
    source: &str,
    options: &SfcTypeCheckOptions,
    options_api: bool,
    legacy_vue2: bool,
) -> SfcTypeCheckResult {
    let mode = if legacy_vue2 {
        SfcCroquisMode::LegacyVue2
    } else if options_api {
        SfcCroquisMode::OptionsApi
    } else {
        SfcCroquisMode::Full
    };
    let request = SfcTypeCheckRequest::new(options.clone(), mode);
    let mut compilation = Compilation::new();
    let result: Result<_, vize_carton::String> = (|| {
        vize_atelier_sfc::register_atlas_providers(&mut compilation)
            .map_err(|error| cstr!("{error}"))?;
        register_sfc_typecheck_provider(&mut compilation).map_err(|error| cstr!("{error}"))?;
        let source_name = if options.filename.ends_with(".vue") {
            options.filename.as_str()
        } else {
            "anonymous.vue"
        };
        let source = compilation
            .add_source(source_name, source)
            .map_err(|error| cstr!("{error}"))?;
        install_sfc_typecheck_request(&mut compilation, source, request)
            .map_err(|error| cstr!("{error}"))?;
        compilation
            .query::<SfcTypeCheckProduct>(source)
            .map(|outcome| outcome.value().clone())
            .map_err(|error| cstr!("{error}"))
    })();
    result.unwrap_or_else(graph_error_result)
}

fn graph_error_result(error: impl std::fmt::Display) -> SfcTypeCheckResult {
    let mut result = SfcTypeCheckResult::empty();
    result.add_diagnostic(SfcTypeDiagnostic {
        severity: SfcTypeSeverity::Error,
        message: cstr!("Typecheck artifact graph failed: {error}"),
        start: 0,
        end: 0,
        code: Some("artifact-graph-error".into()),
        help: None,
        related: Vec::new(),
    });
    result
}

//! Fallback diagnostics emitted by the SFC compiler.

use crate::types::{SfcDescriptor, SfcError};
use vize_atelier_core::source_atlas::SourceAtlasFallback;
use vize_carton::{ToCompactString, profiler::global_profiler};

pub(super) fn push_vapor_ssr_fallback_warning(
    descriptor: &SfcDescriptor,
    warnings: &mut Vec<SfcError>,
) {
    record_atelier_fallback(SourceAtlasFallback::VaporSsr);
    warnings.push(create_vapor_ssr_fallback_warning(descriptor));
}

/// Record an Atelier fallback fact without changing compiler output.
///
/// Fallback facts are intentionally cheaper and narrower than diagnostics:
/// they are profile observations about how a lane finished its work. A caller
/// should emit a user-facing warning only when the fallback changes semantics or
/// target support, such as Vapor SSR. Missing `AtelierOutput` sections are no
/// longer profile fallbacks; they are internal contract errors.
pub(super) fn record_atelier_fallback(fallback: SourceAtlasFallback) {
    global_profiler().record_counter(fallback.profile_counter(), 1);
}

fn create_vapor_ssr_fallback_warning(descriptor: &SfcDescriptor) -> SfcError {
    SfcError {
        message: "SFC Vapor SSR is not supported yet; falling back to standard SSR output."
            .to_compact_string(),
        code: Some("VAPOR_SSR_FALLBACK".to_compact_string()),
        loc: descriptor
            .template
            .as_ref()
            .map(|template| template.loc.clone()),
    }
}

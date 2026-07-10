//! Fallback diagnostics emitted by the SFC compiler.

use crate::types::{SfcCompileOptions, SfcDescriptor, SfcError};
use vize_atlas::{SourceAtlasFallback, SourceAtlasTarget};
use vize_carton::{ToCompactString, profiler::global_profiler};
use vize_rendu::RenduCapabilities;

/// Push the Vapor SSR fallback warning when the requested render route mixes
/// Vapor and SSR. The decision goes through the shared Rendu capability facts
/// so the Vapor lane consumes the same rule the atlas reports.
pub(super) fn apply_vapor_ssr_fallback(
    descriptor: &SfcDescriptor,
    options: &SfcCompileOptions,
    vapor_requested: bool,
    warnings: &mut Vec<SfcError>,
) {
    let mut route = RenduCapabilities::empty();
    if vapor_requested {
        route = route.with_target(SourceAtlasTarget::Vapor);
    }
    if options.template.ssr {
        route = route.with_target(SourceAtlasTarget::Ssr);
    }
    if route.vapor_route_fallback() == Some(SourceAtlasFallback::VaporSsr) {
        push_vapor_ssr_fallback_warning(descriptor, warnings);
    }
}

fn push_vapor_ssr_fallback_warning(descriptor: &SfcDescriptor, warnings: &mut Vec<SfcError>) {
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
pub(crate) fn record_atelier_fallback(fallback: SourceAtlasFallback) {
    global_profiler().record_counter(fallback.profile_counter(), 1);
}

/// Record that the Vapor Atelier could not lower a template shape.
///
/// Routes the decision through the shared `RenduCapabilities::vapor_support`
/// gate so the unsupported-shape fallback is reported the same way every Vapor
/// lane reports it, instead of naming the fallback variant inline.
pub(crate) fn record_unsupported_vapor_shape() {
    let vapor = RenduCapabilities::empty().with_target(SourceAtlasTarget::Vapor);
    if let Some(fallback) = vapor.vapor_support(false).fallback() {
        record_atelier_fallback(fallback);
    }
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

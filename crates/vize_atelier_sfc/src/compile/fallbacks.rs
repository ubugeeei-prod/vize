//! Fallback diagnostics emitted by the SFC compiler.

use crate::types::{SfcCompileOptions, SfcDescriptor, SfcError};
use vize_carton::{ToCompactString, profiler::global_profiler};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum SfcFallback {
    VaporSsr,
    UnsupportedVaporShape,
}

impl SfcFallback {
    const fn profile_counter(self) -> &'static str {
        match self {
            Self::VaporSsr => "atelier.profile.fallback.vapor_ssr",
            Self::UnsupportedVaporShape => "atelier.profile.fallback.vapor_unsupported_shape",
        }
    }
}

/// Push the legacy-pipeline warning when one compile request mixes Vapor and SSR.
/// The graph-native pipeline records this as a provider outcome instead.
pub(super) fn apply_vapor_ssr_fallback(
    descriptor: &SfcDescriptor,
    options: &SfcCompileOptions,
    vapor_requested: bool,
    warnings: &mut Vec<SfcError>,
) {
    if vapor_requested && options.template.ssr {
        push_vapor_ssr_fallback_warning(descriptor, warnings);
    }
}

fn push_vapor_ssr_fallback_warning(descriptor: &SfcDescriptor, warnings: &mut Vec<SfcError>) {
    record_atelier_fallback(SfcFallback::VaporSsr);
    warnings.push(create_vapor_ssr_fallback_warning(descriptor));
}

/// Record an Atelier fallback fact without changing compiler output.
///
/// Fallback facts are intentionally cheaper and narrower than diagnostics:
/// they are profile observations about how a lane finished its work. A caller
/// should emit a user-facing warning only when the fallback changes semantics or
/// target support, such as Vapor SSR. Missing `AtelierOutput` sections are no
/// longer profile fallbacks; they are internal contract errors.
pub(crate) fn record_atelier_fallback(fallback: SfcFallback) {
    global_profiler().record_counter(fallback.profile_counter(), 1);
}

/// Record that the Vapor Atelier could not lower a template shape.
///
pub(crate) fn record_unsupported_vapor_shape() {
    record_atelier_fallback(SfcFallback::UnsupportedVaporShape);
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

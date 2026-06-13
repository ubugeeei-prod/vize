//! Fallback diagnostics emitted by the SFC compiler.

use crate::types::{SfcDescriptor, SfcError};
use vize_atelier_core::source_atlas::SourceAtlasFallback;
use vize_carton::{ToCompactString, profiler::global_profiler};

pub(super) fn push_vapor_ssr_fallback_warning(
    descriptor: &SfcDescriptor,
    warnings: &mut Vec<SfcError>,
) {
    global_profiler().record_counter(SourceAtlasFallback::VaporSsr.profile_counter(), 1);
    warnings.push(create_vapor_ssr_fallback_warning(descriptor));
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

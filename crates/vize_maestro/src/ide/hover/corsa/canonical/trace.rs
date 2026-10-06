//! Passive provenance for the existing real-editor failure capture.

use std::sync::OnceLock;
use vize_canon::{CorsaBridgeError, LspHover};

use crate::ide::{IdeContext, corsa_support::CanonicalVirtualDocument};

fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os("VIZE_LSP_TRACE_NATIVE_SCOPE").as_deref()
            == Some(std::ffi::OsStr::new("1"))
    })
}

pub(super) fn refused(ctx: &IdeContext<'_>, stage: &str, error: Option<&CorsaBridgeError>) {
    if enabled() {
        tracing::info!(
            target: "vize_maestro::ide::hover::canonical_trace",
            uri = %ctx.uri,
            offset = ctx.offset,
            stage,
            error = ?error,
            "canonical hover observed existing refusal"
        );
    }
}

pub(super) fn document(ctx: &IdeContext<'_>, document: &CanonicalVirtualDocument) {
    if enabled() {
        tracing::info!(
            target: "vize_maestro::ide::hover::canonical_trace",
            uri = %ctx.uri,
            offset = ctx.offset,
            request_uri = %document.request_uri,
            authored_source = ?ctx.content,
            generated_source = ?document.virtual_result.code,
            source_mappings = ?document.virtual_result.source_mappings,
            import_source_map = ?document.virtual_result.import_source_map,
            "canonical hover observed opened document"
        );
    }
}

pub(super) fn query(ctx: &IdeContext<'_>, line: u32, character: u32) {
    if enabled() {
        tracing::info!(
            target: "vize_maestro::ide::hover::canonical_trace",
            uri = %ctx.uri,
            offset = ctx.offset,
            line,
            character,
            "canonical hover observed native query"
        );
    }
}

pub(super) fn answer(ctx: &IdeContext<'_>, hover: &LspHover) {
    if enabled() {
        tracing::info!(
            target: "vize_maestro::ide::hover::canonical_trace",
            uri = %ctx.uri,
            offset = ctx.offset,
            hover = ?hover,
            "canonical hover observed complete native result"
        );
    }
}

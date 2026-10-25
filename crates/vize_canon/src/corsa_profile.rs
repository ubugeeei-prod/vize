//! Checker request phases in the shared profiling export.
//!
//! These externally timed samples supplement the enclosing Vize spans. Transport
//! includes queueing, wire I/O and upstream work; it is not a worker CPU sample.
//! Only fixed labels enter the profiler, including for unknown API methods.

#![expect(
    clippy::disallowed_types,
    reason = "Corsa owns its SharedProfiler callback across request and snapshot-release threads"
)]

use corsa::api::{ApiProfileEvent, ApiProfilePhase, ApiProfiler, ApiSpawnConfig};
use std::sync::Arc;
use vize_l0::profiler::{Profiler, SpanAttribution, global_profiler};

/// Product that owns a checker API session.
#[derive(Clone, Copy, Debug)]
pub enum RequestOrigin {
    /// Shared checker sessions used by batch checking and editor integrations.
    Canon,
    /// Type-aware lint sessions.
    Patina,
}

impl RequestOrigin {
    fn label(self) -> &'static str {
        match self {
            Self::Canon => "canon",
            Self::Patina => "patina",
        }
    }
}

/// Install request phase recording only when profiling is already enabled.
///
/// The disabled path keeps the original config and installs no hook, clock or
/// allocation. A caller's existing profiler always takes precedence.
pub fn with_request_profiling(config: ApiSpawnConfig, origin: RequestOrigin) -> ApiSpawnConfig {
    configure(config, origin, global_profiler().is_enabled())
}

fn configure(config: ApiSpawnConfig, origin: RequestOrigin, enabled: bool) -> ApiSpawnConfig {
    if !enabled || config.profiler.is_some() {
        return config;
    }
    config.with_profiler(Arc::new(RequestProfiler(origin)))
}

struct RequestProfiler(RequestOrigin);

impl ApiProfiler for RequestProfiler {
    fn on_profile(&self, event: &ApiProfileEvent) {
        record_event(global_profiler(), self.0, event);
    }
}

fn record_event(profiler: &Profiler, origin: RequestOrigin, event: &ApiProfileEvent) {
    let name = match event.phase {
        ApiProfilePhase::SerializeParams => "corsa.request.serialize_params",
        ApiProfilePhase::Transport => "corsa.request.transport",
        ApiProfilePhase::DeserializeResponse => "corsa.request.deserialize_response",
        ApiProfilePhase::DecodeBinary => "corsa.request.decode_binary",
    };
    let transport = match event.transport.as_str() {
        "jsonrpc" => "jsonrpc",
        "msgpack" => "msgpack",
        _ => "other",
    };
    profiler.record_attributed(
        name,
        SpanAttribution::new()
            .with_stage(origin.label())
            .with_pass(method_label(event.method.as_str()))
            .with_block(transport),
        event.duration,
    );
}

fn method_label(method: &str) -> &'static str {
    // Static match arms bound cardinality and never retain dynamic server text.
    macro_rules! known {
        ($($method:literal),* $(,)?) => {
            match method { $($method => $method,)* _ => "other" }
        };
    }
    known!(
        "initialize",
        "describeCapabilities",
        "parseConfigFile",
        "updateSnapshot",
        "release",
        "releaseSnapshot",
        "getDefaultProjectForFile",
        "getSourceFile",
        "getDiagnostics",
        "getSyntacticDiagnostics",
        "getSemanticDiagnostics",
        "getSuggestionDiagnostics",
        "getConfigFileParsingDiagnostics",
        "getGlobalDiagnostics",
        "getSymbolAtPosition",
        "getSymbolsAtPositions",
        "getTypeOfSymbol",
        "getTypesOfSymbols",
        "getTypeAtPosition",
        "getTypesAtPositions",
        "getTypeAtLocation",
        "getTypeAtLocations",
        "getPropertiesOfType",
        "getSignaturesOfType",
        "getReturnTypeOfSignature",
        "typeToString",
        "getBaseConstraintOfType",
        "getNonNullableType",
        "isTypeAssignableTo",
        "getTypeOfSymbolAtLocation",
        "getExportsOfModule",
        "getHoverAtPosition",
        "getCompletionAtPosition",
        "getDefinitionAtPosition",
        "getReferencesAtPosition",
        "getRenameAtPosition",
    )
}

#[cfg(test)]
mod tests;

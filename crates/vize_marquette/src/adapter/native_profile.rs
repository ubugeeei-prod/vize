use super::AdapterCapabilitySupport;
use crate::CapabilityDefinition;

/// Current contract version shared by the native-engine capability profile.
pub const NATIVE_ENGINE_CAPABILITY_VERSION: u32 = 1;

/// Closed set of native-engine capabilities and application-side summaries.
///
/// The order is part of the language-neutral profile and follows the execution
/// boundary from rendering through lifecycle management.
const NATIVE_ENGINE_CAPABILITIES: [(&str, &str); 8] = [
    (
        "native.rendering",
        "Render native view trees produced from compiled SFC output.",
    ),
    (
        "native.events",
        "Dispatch typed native input and lifecycle events into the application.",
    ),
    (
        "native.layout",
        "Measure and position native nodes with deterministic layout results.",
    ),
    (
        "native.text",
        "Shape, measure, and render text with locale and accessibility metadata.",
    ),
    (
        "native.images",
        "Resolve, decode, cache, and draw application image assets.",
    ),
    (
        "native.animation",
        "Drive declarative native animations with deterministic completion signals.",
    ),
    (
        "native.accessibility",
        "Expose native semantics, focus, and assistive technology metadata.",
    ),
    (
        "native.lifecycle",
        "Report application, screen, and host runtime lifecycle transitions.",
    ),
];

/// Stable capability identifiers required from a native rendering engine.
///
/// The order is part of the language-neutral profile and follows the execution
/// boundary from rendering through lifecycle management.
pub const NATIVE_ENGINE_CAPABILITY_IDS: [&str; 8] = [
    NATIVE_ENGINE_CAPABILITIES[0].0,
    NATIVE_ENGINE_CAPABILITIES[1].0,
    NATIVE_ENGINE_CAPABILITIES[2].0,
    NATIVE_ENGINE_CAPABILITIES[3].0,
    NATIVE_ENGINE_CAPABILITIES[4].0,
    NATIVE_ENGINE_CAPABILITIES[5].0,
    NATIVE_ENGINE_CAPABILITIES[6].0,
    NATIVE_ENGINE_CAPABILITIES[7].0,
];

/// Creates the canonical application-side native-engine capability definitions.
///
/// The returned definitions are the requirements matched against
/// [`native_engine_capability_profile`].
pub fn native_engine_capability_definitions() -> Vec<CapabilityDefinition> {
    NATIVE_ENGINE_CAPABILITIES
        .map(|(id, description)| CapabilityDefinition {
            id: id.into(),
            description: description.into(),
            version: NATIVE_ENGINE_CAPABILITY_VERSION,
        })
        .to_vec()
}

/// Creates the canonical version-one native-engine capability profile.
///
/// A fresh list is returned so an adapter can extend its manifest without
/// mutating the shared profile seen by another consumer.
pub fn native_engine_capability_profile() -> Vec<AdapterCapabilitySupport> {
    NATIVE_ENGINE_CAPABILITY_IDS
        .map(|id| AdapterCapabilitySupport {
            id: id.into(),
            min_version: NATIVE_ENGINE_CAPABILITY_VERSION,
            max_version: NATIVE_ENGINE_CAPABILITY_VERSION,
        })
        .to_vec()
}

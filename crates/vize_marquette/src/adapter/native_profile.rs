use super::AdapterCapabilitySupport;

/// Current contract version shared by the native-engine capability profile.
pub const NATIVE_ENGINE_CAPABILITY_VERSION: u32 = 1;

/// Closed set of capability identifiers required from a native rendering engine.
///
/// The order is part of the language-neutral profile and follows the execution
/// boundary from rendering through lifecycle management.
pub const NATIVE_ENGINE_CAPABILITY_IDS: [&str; 8] = [
    "native.rendering",
    "native.events",
    "native.layout",
    "native.text",
    "native.images",
    "native.animation",
    "native.accessibility",
    "native.lifecycle",
];

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

//! Versioned adapter capability manifests and deterministic negotiation.

mod compatibility;
mod model;
mod native_profile;
mod negotiate;
mod validate;

pub use compatibility::{AdapterCapabilityCompatibilityReport, compare_adapter_capabilities};
pub use model::{
    ADAPTER_CAPABILITY_FORMAT_VERSION, AdapterCapabilityDiagnostic,
    AdapterCapabilityDiagnosticCode, AdapterCapabilityManifest, AdapterCapabilityMismatch,
    AdapterCapabilityMismatchCode, AdapterCapabilityNegotiation, AdapterCapabilitySupport,
};
pub use native_profile::{
    NATIVE_ENGINE_CAPABILITY_IDS, NATIVE_ENGINE_CAPABILITY_VERSION,
    native_engine_capability_definitions, native_engine_capability_profile,
};
pub use negotiate::negotiate_adapter_capabilities;
pub use validate::validate_adapter_capability_manifest;

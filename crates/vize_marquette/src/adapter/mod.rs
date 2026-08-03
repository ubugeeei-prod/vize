//! Versioned adapter capability manifests and deterministic negotiation.

mod compatibility;
mod model;
mod negotiate;
mod validate;

pub use compatibility::compare_adapter_capabilities;
pub use model::{
    ADAPTER_CAPABILITY_FORMAT_VERSION, AdapterCapabilityDiagnostic,
    AdapterCapabilityDiagnosticCode, AdapterCapabilityManifest, AdapterCapabilityMismatch,
    AdapterCapabilityMismatchCode, AdapterCapabilityNegotiation, AdapterCapabilitySupport,
};
pub use negotiate::negotiate_adapter_capabilities;
pub use validate::validate_adapter_capability_manifest;

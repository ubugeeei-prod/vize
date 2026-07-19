//! Versioned application marquettes for Vize.
//!
//! **Experimental:** serialized contracts are versioned, but Rust API
//! compatibility is not guaranteed until the marquette model is validated by
//! multiple target adapters.
//!
//! `vize_marquette` defines the language-neutral graph shared by the web,
//! native, desktop, terminal, backend, transport, testing, and gallery
//! surfaces. The crate intentionally owns contracts rather than runtime
//! implementations. Target-specific crates consume this model and report
//! their capabilities through it.
//!
//! # Design guarantees
//!
//! - Serialized names are stable and independent from Rust type names.
//! - Validation diagnostics have stable codes and authored graph paths.
//! - Canonical serialization is deterministic for equivalent contracts.
//! - Compatibility analysis distinguishes additive and breaking changes.
//! - Every reference is validated before an adapter may execute a plan.
//!
//! # Example
//!
//! ```
//! use vize_marquette::{
//!     ApplicationContract, Backend, BackendFamily, Environment,
//!     EnvironmentConsumer, RuntimeFamily, Target,
//! };
//!
//! let mut contract = ApplicationContract::new("example");
//! contract.targets.insert(Target::Web);
//! contract.environments.push(Environment::new(
//!     "client",
//!     Target::Web,
//!     EnvironmentConsumer::Client,
//!     RuntimeFamily::Browser,
//! ));
//! contract.backends.push(Backend::new(
//!     "server",
//!     BackendFamily::Rust,
//! ));
//!
//! assert!(contract.validate().is_empty());
//! ```

mod canonical;
mod compatibility;
mod model;
mod validate;

#[cfg(test)]
mod model_tests;

pub use canonical::{CanonicalContractError, canonical_json, contract_fingerprint};
pub use compatibility::{
    CompatibilityChange, CompatibilityChangeKind, CompatibilityReport, compare_contracts,
};
pub use model::{
    ApplicationContract, Backend, BackendFamily, CONTRACT_FORMAT_VERSION, CapabilityDefinition,
    Environment, EnvironmentConsumer, Protocol, ProtocolFamily, RenderingMode, Route,
    RuntimeFamily, Target,
};
pub use validate::{ContractDiagnostic, DiagnosticSeverity, validate_contract};

/// Canonical JSON Schema for serialized application contracts.
///
/// The schema is embedded so CLI, editor, backend generator, and AI tooling
/// consumers can expose the exact contract without resolving a filesystem path.
pub const APPLICATION_CONTRACT_JSON_SCHEMA: &str =
    include_str!("../schema/application-contract.schema.json");

#[cfg(test)]
mod schema_tests {
    #[test]
    fn embedded_schema_is_valid_json_and_tracks_the_current_format() {
        let schema: serde_json::Value =
            serde_json::from_str(super::APPLICATION_CONTRACT_JSON_SCHEMA).unwrap();
        assert_eq!(
            schema["$defs"]["contract"]["properties"]["formatVersion"]["const"],
            1
        );
    }
}

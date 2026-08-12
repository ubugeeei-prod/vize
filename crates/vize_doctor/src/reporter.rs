//! Provider-neutral report rendering contracts.
//!
//! Reporters translate one normalized [`crate::DoctorReport`] into a human,
//! CI, editor, or AI-facing representation. Integrations register reporters
//! in an explicitly owned [`ReporterSet`]; the crate deliberately has no
//! global registry, environment-dependent discovery, or mutable singleton.

mod contract;
mod execution;
mod json;
mod registry;
mod sarif;

#[cfg(test)]
mod tests;

pub use contract::{
    DOCTOR_REPORTER_CONTRACT_VERSION, ReporterAudience, ReporterCapability, ReporterContractError,
    ReporterDescriptor, ReporterTransport,
};
pub use execution::{
    DoctorReporter, ReporterError, ReporterErrorKind, ReporterFailure, ReporterOutput,
    ReporterReceipt, render_report,
};
pub use json::JsonReporter;
pub use registry::{ReporterRegistrationError, ReporterSet};
pub use sarif::{SarifMissingSourcePolicy, SarifReporter, SarifSource, SarifSourceError};

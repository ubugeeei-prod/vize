//! Application-contract validation.
//!
//! Validation is split across focused submodules: the diagnostic module owns the
//! stable diagnostic types, the rules module runs the top-level contract checks,
//! and the helpers module holds the shared reference and identifier utilities.

mod diagnostic;
mod helpers;
mod rules;

#[cfg(test)]
mod tests;

pub use diagnostic::{ContractDiagnostic, DiagnosticSeverity};
pub use rules::validate_contract;

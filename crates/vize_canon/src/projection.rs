//! Consumer-side projection services shared by every checker surface.
//!
//! [`assemble`] is P4-5a's one diagnostic post-pass: `vize check` and the
//! Maestro session both hand it the checker's finished diagnostics for one
//! authored file together with the [`crate::virtual_ts::ProjectionMapping`] of
//! every projected document, and render what it returns.

pub mod assemble;
pub mod target;

pub use target::{ExprClassCounts, S2Projection, project_sfc};

pub use assemble::{
    AssembledDiagnostic, AssembledOrigin, AssemblyPolicy, AuthoredSource, FinishedDiagnostic,
    ProjectedDocument, assemble_diagnostics,
};

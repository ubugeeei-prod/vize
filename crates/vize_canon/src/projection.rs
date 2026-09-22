//! Consumer-side projection services shared by every checker surface.
//!
//! [`assemble`] is P4-5a's one diagnostic post-pass: `vize check` and the
//! Maestro session both hand it the checker's finished diagnostics for one
//! authored file together with the [`crate::virtual_ts::ProjectionMapping`] of
//! every projected document, and render what it returns.
//! [`expr`] projects template expressions through the S4 emission document
//! into those same rows (P4-5b).

pub mod assemble;
mod expr;

pub use assemble::{
    AssembledDiagnostic, AssembledOrigin, AssemblyPolicy, AuthoredSource, FinishedDiagnostic,
    ProjectedDocument, assemble_diagnostics,
};
pub use expr::project_template_expressions;

//! The resident tier (Davinci P5-4a): long-lived processes — Maestro,
//! `check-server`, watch modes — run the stages as **salsa** queries keyed by
//! the P5-1a stage artifact keys (charter #10, the rust-analyzer/rustc
//! two-tier precedent).
//!
//! **Experimental:** the API may change in any alpha release.
//!
//! - [`artifact`] — the owned per-block artifacts (S0 source block, S1
//!   surface, S2 page) and the pure stage functions that compute them. They
//!   are the clean path TS-42 compares the tier with.
//! - [`db`] — [`ResidentDatabase`]: file texts and project config as inputs,
//!   [`Block`](db::Block) as the firewall (content and position as separate
//!   tracked fields), and the `sfc_blocks` → `s1_block` / `s2_page` queries.
//! - [`accounting`] — cache-hit accounting read from salsa's event stream
//!   (TS-46).
//! - [`equivalence`] — TS-42: edit scripts run through the database, every
//!   served artifact compared with the clean path after every step.
//!
//! The one-shot CLI never links this crate: `vize build`/`fmt`/`lint` stay on
//! the fused non-salsa pipeline, and `tests/tooling/davinci-resident-salsa.test.ts`
//! proves no crate outside the resident tier depends on `salsa`.

pub mod accounting;
pub mod artifact;
pub mod db;
pub mod equivalence;
pub mod snapshot;

pub use accounting::{Accounting, QueryCounts};
pub use artifact::{
    BlockArtifacts, BlockKind, BlockSource, PageArtifact, StageConfig, SurfaceArtifact,
    compute_file_artifacts,
};
pub use db::{ResidentDatabase, SourceFile};

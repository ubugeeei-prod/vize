//! Shared Davinci test infrastructure.
//!
//! Keeping cross-crate fixtures and validators in a regular dependency avoids
//! relative `#[path]` includes while preserving one source of truth.

#[expect(
    clippy::panic,
    reason = "test support: the corpus gate fails the calling test with a diagnostic by panicking"
)]
pub mod corpus;
pub mod schema;
pub mod surface_fixture;

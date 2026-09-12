//! S3 - the Davinci reactivity IR (codename Impeto): the backend convergence
//! layer for DOM and Vapor.
//!
//! **Experimental:** the stage API may change in any alpha release; record
//! intentional breaking changes in the release notes.
//!
//! Impeto is flat and id-based by design. S2 owns the semantic tree; S3 owns
//! update ordering, effect grouping, and backend decisions that must be shared
//! rather than rediscovered by each emitter. Ordering constraints are explicit
//! [`op::StateEdge`] values, regions are named by [`op::RegionId`], and the
//! phase validator rejects unresolved edges, malformed region nesting, and
//! effect scopes that leak.
//!
//! The crate is `no_std + alloc` from birth so S3 artifacts can be printed,
//! validated, and replayed on the same portability lane as the earlier Davinci
//! stage libraries.

#![no_std]

extern crate alloc;

pub mod folio;
pub mod op;
pub mod verify;

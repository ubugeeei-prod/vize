//! The Davinci extension-contract host (phase 6).
//!
//! **Experimental:** the host API and the `vize:contracts` WIT package may
//! change in any alpha release until contracts GA; record intentional
//! breaking changes in the release notes.
//!
//! Charter #15's two tiers meet here. External input dialects are
//! component-model guests of the `input-dialect` world in `contracts/wit/`,
//! reached over a serialized contract; the first-party Vue dialect
//! ([`vue::VueDialect`]) implements the same [`InputDialectGuest`] trait
//! compiled in, with no transport at all. Either way the host treats the
//! guest's answer as untrusted input and accepts it through one path:
//!
//! 1. [`Session::open`] calls `get-capability` once and negotiates it
//!    ([`handshake::negotiate`]): the protocol version must match, features
//!    must be sorted and unique, and the page features the world requires
//!    must be offered — each refusal has one exact message.
//! 2. [`Session::lower_block`] sends one whole block (coarse-grained: the
//!    canonical ABI copies once per block, never per node) and accepts the
//!    answer ([`accept`]): both pages at a schema version this host reads,
//!    both pages canonical (`print(parse(text)) == text`), the S1 page's
//!    tokens tiling the block source exactly (TS-19's fidelity law, now
//!    enforced at the boundary), and every diagnostic span inside the block.
//!
//! The S1 page ([`surface_page`]) is the lossless surface tree as a folio
//! page of block-relative token offsets; the S2 page is the existing
//! disegno page ([`vize_s2::folio::S2Folio`]). Hosting modes live beside
//! the contract: [`outproc`] runs a guest in a child process over the
//! [`wire`] protocol, and the `extension-host` feature adds `wasm`, the
//! wasmtime component host the child process uses.

pub mod accept;
pub mod contract;
pub mod expression;
pub mod handshake;
pub mod outproc;
pub mod output;
pub mod session;
pub mod surface_page;
pub mod vue;
#[cfg(feature = "extension-host")]
pub mod wasm;
pub mod wire;

pub use accept::{Accepted, accept};
pub use contract::{
    Capability, Diagnostic, DiagnosticPart, GuestError, GuestLimits, InputDialectGuest,
    LoweredBlock, Page, PartKind, Severity, SourceBlock, Span, Stage, Witness,
};
pub use handshake::{HandshakeError, Negotiated, negotiate, negotiate_for};
pub use session::{ContractError, Session};
pub use surface_page::{SurfacePage, TileError};

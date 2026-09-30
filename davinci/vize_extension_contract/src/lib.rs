//! Transport-free extension records, negotiation and canonical acceptance.
//!
//! First-party dialects use this crate without acquiring any component runtime.
//! `vize_extension_host` reexports this API and adds external guest transports.

pub mod accept;
pub mod contract;
pub mod expression;
pub mod handshake;
pub mod surface_page;
pub mod typed_expression;

pub use accept::{Accepted, accept};
pub use contract::{
    Capability, Diagnostic, DiagnosticPart, GuestError, GuestLimits, InputDialectGuest,
    LoweredBlock, Page, PartKind, Severity, SourceBlock, Span, Stage, Witness,
};
pub use handshake::{HandshakeError, Negotiated, negotiate, negotiate_for};
pub use surface_page::{SurfacePage, TileError};

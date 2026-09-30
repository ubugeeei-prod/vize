//! Transitional imports; negotiation is owned by L0.
//! Remove this module with the old contract package after its producer-owned
//! diagnostic conversions and acceptance paths have moved to their levels.

pub use vize_l0::extension::handshake::{HandshakeError, Negotiated, negotiate, negotiate_for};

//! A negotiated conversation with one output-target guest.

use core::fmt;

use super::{
    AcceptedEmit, EmitError, EmitRequest, OutputTargetGuest, REQUIRED_FEATURES, accept_emitted,
};
use crate::contract::GuestError;
use crate::handshake::{HandshakeError, Negotiated, negotiate_for};

/// Why an output-world exchange failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputError {
    /// The guest never answered.
    Guest(GuestError),
    /// The capability offer did not negotiate.
    Handshake(HandshakeError),
    /// The guest answered and the host refused the answer.
    Emit(EmitError),
}

impl fmt::Display for OutputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Guest(error) => error.fmt(f),
            Self::Handshake(error) => write!(f, "handshake refused: {error}"),
            Self::Emit(error) => write!(f, "emission refused: {error}"),
        }
    }
}

/// An output-target guest whose capability negotiated.
pub struct OutputSession<G> {
    guest: G,
    negotiated: Negotiated,
}

impl<G> fmt::Debug for OutputSession<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OutputSession")
            .field("negotiated", &self.negotiated)
            .finish_non_exhaustive()
    }
}

impl<G: OutputTargetGuest> OutputSession<G> {
    /// Call `get-capability` once and negotiate it for this world.
    ///
    /// # Errors
    ///
    /// The guest's failure or the handshake refusal.
    pub fn open(mut guest: G) -> Result<Self, OutputError> {
        let offer = guest.get_capability().map_err(OutputError::Guest)?;
        let negotiated =
            negotiate_for(&offer, REQUIRED_FEATURES).map_err(OutputError::Handshake)?;
        Ok(Self { guest, negotiated })
    }

    /// The negotiated capability.
    #[must_use]
    pub fn negotiated(&self) -> &Negotiated {
        &self.negotiated
    }

    /// Emit one template through the guest and accept the answer.
    ///
    /// # Errors
    ///
    /// The guest's failure or the host's refusal of its answer.
    pub fn emit(&mut self, request: &EmitRequest) -> Result<AcceptedEmit, OutputError> {
        let emitted = self.guest.emit(request).map_err(OutputError::Guest)?;
        accept_emitted(request, emitted).map_err(OutputError::Emit)
    }
}

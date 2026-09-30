//! A negotiated conversation with one expression-dialect guest.

use core::fmt;

use super::{
    AcceptedAnalysis, AnalysisError, ExpressionBatch, ExpressionDialectGuest, REQUIRED_FEATURES,
    accept_analysis,
};
use crate::contract::GuestError;
use crate::handshake::{HandshakeError, Negotiated, negotiate_for};

/// Why an expression-world exchange failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionError {
    /// The guest never answered.
    Guest(GuestError),
    /// The capability offer did not negotiate.
    Handshake(HandshakeError),
    /// The guest answered and the host refused the answer.
    Analysis(AnalysisError),
}

impl fmt::Display for ExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Guest(error) => error.fmt(f),
            Self::Handshake(error) => write!(f, "handshake refused: {error}"),
            Self::Analysis(error) => write!(f, "analysis refused: {error}"),
        }
    }
}

/// An expression-dialect guest whose capability negotiated.
pub struct ExpressionSession<G> {
    guest: G,
    negotiated: Negotiated,
}

impl<G> fmt::Debug for ExpressionSession<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExpressionSession")
            .field("negotiated", &self.negotiated)
            .finish_non_exhaustive()
    }
}

impl<G: ExpressionDialectGuest> ExpressionSession<G> {
    /// Call `get-capability` once and negotiate it for this world.
    ///
    /// # Errors
    ///
    /// The guest's failure or the handshake refusal.
    pub fn open(mut guest: G) -> Result<Self, ExpressionError> {
        let offer = guest.get_capability().map_err(ExpressionError::Guest)?;
        let negotiated =
            negotiate_for(&offer, REQUIRED_FEATURES).map_err(ExpressionError::Handshake)?;
        Ok(Self { guest, negotiated })
    }

    /// The negotiated capability.
    #[must_use]
    pub fn negotiated(&self) -> &Negotiated {
        &self.negotiated
    }

    /// Analyze one batch through the guest and accept the answer.
    ///
    /// # Errors
    ///
    /// The guest's failure or the host's refusal of its answer.
    pub fn analyze(
        &mut self,
        batch: &ExpressionBatch,
    ) -> Result<AcceptedAnalysis, ExpressionError> {
        let analysis = self.guest.analyze(batch).map_err(ExpressionError::Guest)?;
        accept_analysis(batch, analysis).map_err(ExpressionError::Analysis)
    }
}

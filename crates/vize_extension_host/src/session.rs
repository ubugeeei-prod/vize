//! A negotiated conversation with one input-dialect guest.

use core::fmt;

use vize_s0::String;

use crate::accept::{AcceptError, Accepted, accept};
use crate::contract::{GuestError, InputDialectGuest, LANG_FEATURE_PREFIX, SourceBlock};
use crate::handshake::{HandshakeError, Negotiated, negotiate};

/// Why a contract exchange failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractError {
    /// The guest never answered.
    Guest(GuestError),
    /// The capability offer did not negotiate.
    Handshake(HandshakeError),
    /// The block's `lang` is not one the guest declared; the guest was not
    /// called.
    UndeclaredLang(String),
    /// The guest answered and the host refused the answer.
    Accept(AcceptError),
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Guest(error) => error.fmt(f),
            Self::Handshake(error) => write!(f, "handshake refused: {error}"),
            Self::UndeclaredLang(lang) => write!(
                f,
                "the guest does not declare feature \"{LANG_FEATURE_PREFIX}{lang}\""
            ),
            Self::Accept(error) => write!(f, "answer refused: {error}"),
        }
    }
}

impl From<GuestError> for ContractError {
    fn from(error: GuestError) -> Self {
        Self::Guest(error)
    }
}

/// A guest whose capability negotiated.
pub struct Session<G> {
    guest: G,
    negotiated: Negotiated,
}

impl<G> fmt::Debug for Session<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session")
            .field("negotiated", &self.negotiated)
            .finish_non_exhaustive()
    }
}

impl<G: InputDialectGuest> Session<G> {
    /// Call `get-capability` once and negotiate it.
    ///
    /// # Errors
    ///
    /// [`ContractError::Guest`] when the call fails, or
    /// [`ContractError::Handshake`] when the offer is refused.
    pub fn open(mut guest: G) -> Result<Self, ContractError> {
        let offer = guest.get_capability()?;
        let negotiated = negotiate(&offer).map_err(ContractError::Handshake)?;
        Ok(Self { guest, negotiated })
    }

    /// The negotiated capability.
    #[must_use]
    pub fn negotiated(&self) -> &Negotiated {
        &self.negotiated
    }

    /// Lower one block through the guest and accept the answer.
    ///
    /// # Errors
    ///
    /// [`ContractError::UndeclaredLang`] before calling the guest, then the
    /// guest's own failure or the host's refusal of its answer.
    pub fn lower_block(&mut self, block: &SourceBlock) -> Result<Accepted, ContractError> {
        if let Some(lang) = &block.lang
            && !self.negotiated.lowers_lang(lang)
        {
            return Err(ContractError::UndeclaredLang(lang.clone()));
        }
        let lowered = self.guest.lower_block(block)?;
        accept(block, lowered).map_err(ContractError::Accept)
    }

    /// Give the guest back.
    pub fn into_guest(self) -> G {
        self.guest
    }
}

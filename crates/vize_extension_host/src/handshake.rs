//! Capability negotiation: the `handshake` interface's host half.
//!
//! The rules, in the order they are checked (the first failure is the
//! refusal, and its message is exact):
//!
//! 1. the protocol version equals [`PROTOCOL_VERSION`];
//! 2. the feature list is sorted by byte order and has no duplicates, so an
//!    offer has one spelling and negotiation is deterministic;
//! 3. every feature the world requires ([`REQUIRED_FEATURES`]) is offered.
//!
//! Features this host does not know are accepted and set aside as
//! [`Negotiated::ignored`]: offering more is additive, never a refusal.

use core::fmt;

use vize_s0::String;

use crate::contract::{
    Capability, LANG_FEATURE_PREFIX, PROTOCOL_VERSION, REQUIRED_FEATURES, S1_PAGE_FEATURE,
    S2_PAGE_FEATURE,
};

/// Why a capability offer was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeError {
    /// The guest speaks a protocol version this host does not.
    ProtocolMismatch { host: u32, guest: u32 },
    /// `features[index]` does not sort strictly after its predecessor.
    UnsortedFeatures {
        index: usize,
        feature: String,
        previous: String,
    },
    /// A feature the world requires is absent.
    MissingFeature(&'static str),
}

impl fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProtocolMismatch { host, guest } => write!(
                f,
                "protocol version mismatch: the host speaks {host}, the guest offered {guest}"
            ),
            Self::UnsortedFeatures {
                index,
                feature,
                previous,
            } => write!(
                f,
                "features must be sorted and unique: feature {index} {feature:?} does not sort after {previous:?}"
            ),
            Self::MissingFeature(feature) => {
                write!(f, "the guest does not offer required feature {feature:?}")
            }
        }
    }
}

/// A negotiated capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Negotiated {
    /// The protocol version both sides speak.
    pub protocol_version: u32,
    /// The `lang` values the guest declared through `lang:<value>` features.
    pub langs: Vec<String>,
    /// Offered features this host does not know, in offer order.
    pub ignored: Vec<String>,
}

impl Negotiated {
    /// Whether the guest declared `lang:<lang>`.
    #[must_use]
    pub fn lowers_lang(&self, lang: &str) -> bool {
        self.langs.iter().any(|declared| declared == lang)
    }
}

/// Negotiate a guest's offer against this host.
///
/// # Errors
///
/// The first rule the offer breaks, as a [`HandshakeError`].
pub fn negotiate(offer: &Capability) -> Result<Negotiated, HandshakeError> {
    if offer.protocol_version != PROTOCOL_VERSION {
        return Err(HandshakeError::ProtocolMismatch {
            host: PROTOCOL_VERSION,
            guest: offer.protocol_version,
        });
    }
    for (index, pair) in offer.features.windows(2).enumerate() {
        if pair[0].as_bytes() >= pair[1].as_bytes() {
            return Err(HandshakeError::UnsortedFeatures {
                index: index + 1,
                feature: pair[1].clone(),
                previous: pair[0].clone(),
            });
        }
    }
    for required in REQUIRED_FEATURES {
        if !offer.features.iter().any(|feature| feature == required) {
            return Err(HandshakeError::MissingFeature(required));
        }
    }
    let mut langs = Vec::new();
    let mut ignored = Vec::new();
    for feature in &offer.features {
        if let Some(lang) = feature.strip_prefix(LANG_FEATURE_PREFIX) {
            langs.push(String::from(lang));
        } else if feature != S1_PAGE_FEATURE && feature != S2_PAGE_FEATURE {
            ignored.push(feature.clone());
        }
    }
    Ok(Negotiated {
        protocol_version: PROTOCOL_VERSION,
        langs,
        ignored,
    })
}

use vize_canon::CorsaBridgeError;
use vize_s0::String;

use crate::ide::corsa_support::CanonicalProjectOpenError;

use super::Answer;

/// Why the canonical Corsa route could not return a trustworthy answer.
///
/// The fallback policy is part of the error because the existing production
/// contract distinguishes a missing primary route (try the legacy route) from
/// an invalid linked answer (fail closed instead of mixing rename identities).
#[derive(Debug)]
pub(in crate::ide::rename) enum CanonicalFailure {
    FallbackBridge(CorsaBridgeError),
    AuthoritativeBridge(CorsaBridgeError),
    InvalidResponse {
        operation: &'static str,
        message: String,
    },
    UnmappedResponse(&'static str),
}

impl std::fmt::Display for CanonicalFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FallbackBridge(error) | Self::AuthoritativeBridge(error) => {
                write!(formatter, "{error}")
            }
            Self::InvalidResponse { operation, message } => {
                write!(formatter, "invalid Corsa {operation} response: {message}")
            }
            Self::UnmappedResponse(operation) => {
                write!(formatter, "unmapped Corsa {operation} response")
            }
        }
    }
}

impl std::error::Error for CanonicalFailure {}

impl CanonicalFailure {
    pub(super) fn from_project_open(error: CanonicalProjectOpenError) -> Self {
        match error {
            CanonicalProjectOpenError::Primary(error) => Self::FallbackBridge(error),
            CanonicalProjectOpenError::Importer(error) => Self::AuthoritativeBridge(error),
        }
    }

    pub(super) fn into_lenient_answer<T>(self) -> Answer<T> {
        match self {
            Self::FallbackBridge(_) => Answer::Unavailable,
            Self::AuthoritativeBridge(_)
            | Self::InvalidResponse { .. }
            | Self::UnmappedResponse(_) => Answer::Available(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use vize_canon::CorsaBridgeError;

    use super::*;

    #[test]
    fn project_open_phase_preserves_the_outer_fallback_boundary() {
        let primary = CanonicalFailure::from_project_open(CanonicalProjectOpenError::Primary(
            CorsaBridgeError::Timeout,
        ));
        assert!(matches!(
            primary.into_lenient_answer::<()>(),
            Answer::Unavailable
        ));

        let importer = CanonicalFailure::from_project_open(CanonicalProjectOpenError::Importer(
            CorsaBridgeError::Timeout,
        ));
        assert!(matches!(
            importer.into_lenient_answer::<()>(),
            Answer::Available(None)
        ));
    }
}

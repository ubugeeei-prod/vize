//! What one consumer used from a summary, and the fingerprints it saw.
//!
//! Recompilation of the consumer is skipped when every recorded fingerprint
//! is still the summary's fingerprint for that declaration. A declaration
//! the consumer did not record cannot invalidate it.

use alloc::vec::Vec;

use vize_s0::String;

use super::{DeclarationId, Facet, Fingerprint, SfcSummary, SummaryError};

/// The declarations one consumer used, each with the fingerprint it saw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usage {
    consumer: String,
    used: Vec<(DeclarationId, Fingerprint)>,
}

impl Usage {
    /// Record `used` as read from `summary` by `consumer`.
    ///
    /// # Errors
    ///
    /// An empty consumer name, a declaration the summary does not export,
    /// or the same declaration twice.
    pub fn record(
        consumer: &str,
        summary: &SfcSummary,
        used: &[(Facet, &str)],
    ) -> Result<Self, SummaryError> {
        if consumer.is_empty() {
            return Err(SummaryError::EmptyConsumer);
        }
        let mut recorded = Vec::with_capacity(used.len());
        for &(facet, name) in used {
            if recorded
                .iter()
                .any(|(id, _): &(DeclarationId, Fingerprint)| {
                    id.facet() == facet && id.name() == name
                })
            {
                return Err(SummaryError::DuplicateUse {
                    facet,
                    name: String::from(name),
                });
            }
            let Some(fp) = summary.fingerprint(facet, name) else {
                return Err(SummaryError::Unknown {
                    facet,
                    name: String::from(name),
                });
            };
            recorded.push((DeclarationId::new(facet, name), fp));
        }
        Ok(Self {
            consumer: String::from(consumer),
            used: recorded,
        })
    }

    /// The consumer's name.
    #[must_use]
    pub fn consumer(&self) -> &str {
        &self.consumer
    }

    /// Whether every recorded fingerprint is unchanged in `next`.
    #[must_use]
    pub fn is_fresh(&self, next: &SfcSummary) -> bool {
        self.used
            .iter()
            .all(|(id, fp)| next.fingerprint(id.facet(), id.name()) == Some(*fp))
    }
}

impl SfcSummary {
    /// Consumers in `usages` order whose recorded fingerprints are not all
    /// still this summary's. Everyone else is left out.
    #[must_use]
    pub fn invalidated<'a>(&self, usages: &'a [Usage]) -> Vec<&'a str> {
        usages
            .iter()
            .filter(|usage| !usage.is_fresh(self))
            .map(Usage::consumer)
            .collect()
    }
}

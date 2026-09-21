//! [`Verdict`] — the epistemic axis every three-valued fact carries.
//!
//! `assurance.md`: "Every semantic fact is `proven / refuted / unknown` …
//! error-severity diagnostics fire only on `proven`". P3-2 introduced the
//! axis for the S3 reactivity lattice in `vize_impeto`; P4-6a moves it here so
//! the lattice, every later fact group and the witness verifier share **one**
//! type instead of each growing its own. `vize_impeto::lattice::Verdict`
//! re-exports this one, spellings unchanged.

/// Epistemic axis for a fact, orthogonal to the fact's value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Verdict {
    /// The fact is proved. The only verdict a witness link may rest on.
    Proven,
    /// The fact's negation is proved.
    Refuted,
    /// Neither is proved. Produces silence or an advisory, never an error.
    Unknown,
}

impl Verdict {
    /// Stable folio spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proven => "proven",
            Self::Refuted => "refuted",
            Self::Unknown => "unknown",
        }
    }

    /// Parse the stable folio spelling.
    #[must_use]
    pub const fn from_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"proven" => Some(Self::Proven),
            b"refuted" => Some(Self::Refuted),
            b"unknown" => Some(Self::Unknown),
            _ => None,
        }
    }

    /// Whether a consumer may use the value as proof.
    #[must_use]
    pub const fn is_proven(self) -> bool {
        matches!(self, Self::Proven)
    }
}

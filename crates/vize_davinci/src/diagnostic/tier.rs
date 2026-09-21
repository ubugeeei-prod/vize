//! Precision tiers as rule metadata — `assurance.md`'s fourth verdict rule.
//!
//! "Every rule declares `exact` / `sound` / `complete` / `heuristic`.
//! Heuristic rules are barred from error severity by policy, the tier renders
//! in the docs, and the declared domain is part of the rule's contract — 'no
//! FN' always means _no FN within the declared domain_, and shrinking the
//! domain silently is a breaking change."
//!
//! The policy is a compile error here, not a review comment. A
//! [`RuleContract`] is declared in a `const` or `static` item — Patina's
//! `rule_contracts.rs` table is one — and [`RuleContract::new`] panics in
//! const evaluation when the tier does not admit the severity, so the canary
//! rule that tries error on a heuristic verdict does not build:
//!
//! ```compile_fail,E0080
//! use vize_davinci::diagnostic::{Domain, RuleContract, Severity, Tier};
//!
//! static CANARY: RuleContract = RuleContract::new(
//!     Tier::Heuristic,
//!     Domain::new("any template the author wrote"),
//!     Severity::Error,
//! );
//! ```
//!
//! Its twin, identical but for the severity, builds — which is what pins the
//! canary's failure to the heuristic-error pairing (stable rustdoc does not
//! check the error code):
//!
//! ```
//! use vize_davinci::diagnostic::{Domain, RuleContract, Severity, Tier};
//!
//! static HEURISTIC: RuleContract = RuleContract::new(
//!     Tier::Heuristic,
//!     Domain::new("any template the author wrote"),
//!     Severity::Warning,
//! );
//! assert_eq!(HEURISTIC.clamp(Severity::Error), Severity::Warning);
//! ```

use super::Severity;

/// How much a rule's verdicts claim over its declared [`Domain`].
///
/// Ordered from the strongest claim to none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    /// A decidable domain: zero false positives **and** zero false
    /// negatives, required and testable (HTML content model, template CFG,
    /// structural rules).
    Exact,
    /// No false negatives over the declared domain.
    Sound,
    /// No false positives over the declared domain.
    Complete,
    /// Neither guarantee. Barred from [`Severity::Error`].
    Heuristic,
}

impl Tier {
    /// Every tier, from the strongest claim to none.
    pub const ALL: [Tier; 4] = [Tier::Exact, Tier::Sound, Tier::Complete, Tier::Heuristic];

    /// The stable lowercase spelling — what docs, `--explain` pages and the
    /// rule-parity matrix's `tier` column print.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Tier::Exact => "exact",
            Tier::Sound => "sound",
            Tier::Complete => "complete",
            Tier::Heuristic => "heuristic",
        }
    }

    /// Parse the stable spelling.
    #[must_use]
    pub const fn from_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"exact" => Some(Tier::Exact),
            b"sound" => Some(Tier::Sound),
            b"complete" => Some(Tier::Complete),
            b"heuristic" => Some(Tier::Heuristic),
            _ => None,
        }
    }

    /// Whether a rule of this tier may report at `severity`: every pairing
    /// except a heuristic error.
    #[must_use]
    pub const fn admits(self, severity: Severity) -> bool {
        !matches!((self, severity), (Tier::Heuristic, Severity::Error))
    }

    /// Whether the tier promises zero false positives over its domain.
    #[must_use]
    pub const fn no_false_positives(self) -> bool {
        matches!(self, Tier::Exact | Tier::Complete)
    }

    /// Whether the tier promises zero false negatives over its domain — the
    /// classes TS-37's seeded-defect recall must catch every time.
    #[must_use]
    pub const fn no_false_negatives(self) -> bool {
        matches!(self, Tier::Exact | Tier::Sound)
    }
}

/// The declared domain a tier's claim ranges over, stated in one sentence.
///
/// Part of the rule's contract: narrowing it is a breaking change, so it is
/// data a diff shows rather than prose a reviewer has to remember.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Domain(&'static str);

impl Domain {
    /// The domain stated by `statement`.
    ///
    /// # Panics
    ///
    /// Panics if `statement` is empty — an undeclared domain makes every
    /// tier claim vacuous. In a `const` context that panic is a compile
    /// error.
    #[must_use]
    pub const fn new(statement: &'static str) -> Self {
        assert!(
            !statement.is_empty(),
            "a rule's declared domain must state what the tier's claim ranges over"
        );
        Self(statement)
    }

    /// The domain statement.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// A rule's precision contract: its [`Tier`], the [`Domain`] the tier ranges
/// over, and the severity it reports at by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuleContract {
    tier: Tier,
    domain: Domain,
    severity: Severity,
}

impl RuleContract {
    /// A rule of `tier` over `domain`, reporting at `severity` by default.
    ///
    /// # Panics
    ///
    /// Panics if `tier` does not [admit](Tier::admits) `severity` — a
    /// heuristic rule declaring error severity. Contracts are declared in
    /// `const` or `static` items, where that panic is a compile error (the
    /// `compile_fail` canary in the [module docs](self)).
    #[must_use]
    pub const fn new(tier: Tier, domain: Domain, severity: Severity) -> Self {
        assert!(
            tier.admits(severity),
            "a heuristic rule cannot declare error severity: an error must be a proof"
        );
        Self {
            tier,
            domain,
            severity,
        }
    }

    /// The rule's tier.
    #[must_use]
    pub const fn tier(self) -> Tier {
        self.tier
    }

    /// The domain the tier's claim ranges over.
    #[must_use]
    pub const fn domain(self) -> Domain {
        self.domain
    }

    /// The severity the rule reports at by default.
    #[must_use]
    pub const fn severity(self) -> Severity {
        self.severity
    }

    /// The severity the rule may actually report at when a user configures
    /// `configured`: unchanged when the tier admits it, otherwise
    /// [`Severity::Warning`] — configuration cannot promote a heuristic
    /// finding into a proof.
    #[must_use]
    pub const fn clamp(self, configured: Severity) -> Severity {
        if self.tier.admits(configured) {
            configured
        } else {
            Severity::Warning
        }
    }
}

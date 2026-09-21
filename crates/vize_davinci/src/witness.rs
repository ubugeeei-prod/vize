//! TS-36 — witness verification (P4-6b).
//!
//! `assurance.md`: "The witness is machine-checkable against the fact base,
//! so a false positive is not a matter of opinion: it is a witness that fails
//! verification, caught by the same verifier infrastructure as everything
//! else." [`verify`] is that check. It re-reads every [`WitnessLink`] of a
//! diagnostic's chain against the fact base and accepts the chain only when
//! each link names
//!
//! 1. a fact group registered for witnesses ([`WitnessChecks`]),
//! 2. that the producer **declared** and the run computed — the read goes
//!    through the producer's own [`FactView`], so a witness citing a group
//!    outside its producer's demand trips the TS-35 detector,
//! 3. a key of the group's shape under which a fact is stored,
//! 4. the exact span that fact is about, and
//! 5. a fact whose [`Verdict`](crate::diagnostic::Verdict) is proven.
//!
//! The first link that fails yields the exact [`WitnessError`] — link index,
//! group and the observed values — so a forged witness is a fixture with an
//! exact oracle, never a boolean.
//!
//! A [`LegacyExempt`](crate::diagnostic::Witness::LegacyExempt) error has no
//! chain to re-check: it is counted by the exemption inventory instead, and
//! [`WitnessAudit`] reports it separately.
//!
//! # Module layout
//!
//! - [`check`] — [`WitnessGroup`], [`WitnessCheck`] and the [`WitnessChecks`]
//!   registry
//! - [`error`] — [`WitnessError`]
//! - [`audit`] — [`WitnessAudit`], the debug/CI observer (a ZST in release
//!   builds)

pub mod audit;
pub mod check;
pub mod error;

pub use audit::{AuditReport, WitnessAudit, WitnessFailure, unverifiable_witnesses};
pub use check::{WitnessCheck, WitnessChecks, WitnessGroup};
pub use error::WitnessError;

use crate::diagnostic::{Diagnostic, WitnessChain};
use crate::fact::{FactConsumer, FactManager, FactView};

/// Re-check `diagnostic`'s witness chain against `facts`, the view of the
/// consumer that produced it.
///
/// A diagnostic without a chain — an advisory, or a legacy error exempt by
/// inventory — has nothing to re-check and verifies trivially; an advisory
/// that carries a "why" chain is checked like a proof.
///
/// # Errors
///
/// The first link that does not verify, as the exact [`WitnessError`].
pub fn verify(
    diagnostic: &Diagnostic,
    facts: &FactView<'_>,
    checks: &WitnessChecks,
) -> Result<(), WitnessError> {
    match diagnostic.witness_chain() {
        Some(chain) => verify_chain(chain, facts, checks),
        None => Ok(()),
    }
}

/// [`verify`] through consumer `C`'s view of `facts` — the form a producer
/// that owns its [`FactManager`] calls.
///
/// # Errors
///
/// As [`verify`].
pub fn verify_as<C: FactConsumer, A: ?Sized + 'static>(
    diagnostic: &Diagnostic,
    facts: &FactManager<'_, '_, A>,
    checks: &WitnessChecks,
) -> Result<(), WitnessError> {
    verify(diagnostic, &facts.view::<C>(), checks)
}

/// Re-check every link of `chain`, in proof order.
///
/// # Errors
///
/// As [`verify`].
pub fn verify_chain(
    chain: &WitnessChain,
    facts: &FactView<'_>,
    checks: &WitnessChecks,
) -> Result<(), WitnessError> {
    for (index, link) in chain.links().iter().enumerate() {
        let Some(check) = checks.check(link.group) else {
            return Err(WitnessError::UnknownGroup {
                link: index,
                group: link.group,
            });
        };
        check.run(facts, index, link)?;
    }
    Ok(())
}

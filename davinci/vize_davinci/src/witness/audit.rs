//! [`WitnessAudit`] — TS-36's observer: every diagnostic a producer emits is
//! re-checked in debug and CI builds, and the audit is a zero-sized type that
//! does nothing in release builds.
//!
//! A producer hands the audit each batch together with the [`FactView`] it
//! produced the batch from. Chains are verified with [`verify_chain`];
//! failures are kept with the ordinal of the diagnostic that carried them
//! and bump a process-global counter ([`unverifiable_witnesses`]) that a
//! whole test suite can pin to zero — "unverifiable witness = CI failure".

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

#[cfg(debug_assertions)]
use super::verify_chain;
use super::{WitnessChecks, WitnessError};
use crate::diagnostic::Diagnostic;
use crate::fact::FactView;

/// How many witnesses have failed verification in this process — TS-36's
/// "zero unverifiable witnesses" reads this. Always 0 in release builds,
/// where the audit does not exist.
static UNVERIFIABLE: AtomicU64 = AtomicU64::new(0);

/// How many witnesses the audit has refused in this process.
#[must_use]
pub fn unverifiable_witnesses() -> u64 {
    UNVERIFIABLE.load(Ordering::Relaxed)
}

/// One refused witness: the ordinal of the diagnostic among everything this
/// audit observed, and the exact error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitnessFailure {
    /// Zero-based position of the diagnostic across every audited batch.
    pub diagnostic: usize,
    /// Why its witness does not verify.
    pub error: WitnessError,
}

/// What an audit saw.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuditReport {
    /// Diagnostics observed, of every severity.
    pub observed: usize,
    /// Chains that verified (errors and advisory "why" chains alike).
    pub verified: usize,
    /// Errors reported under a legacy exemption — counted by the inventory,
    /// not re-checkable.
    pub exempt: usize,
    /// Chains that did not verify, in observation order.
    pub failures: Vec<WitnessFailure>,
}

/// The TS-36 observer. Debug builds keep an [`AuditReport`]; release builds
/// keep nothing (a zero-sized type, const-asserted) and every method is a
/// no-op, so attaching the audit costs a release build nothing.
#[derive(Debug, Default)]
pub struct WitnessAudit {
    #[cfg(debug_assertions)]
    report: AuditReport,
}

#[cfg(not(debug_assertions))]
const _: () = assert!(size_of::<WitnessAudit>() == 0);

impl WitnessAudit {
    /// An audit that has observed nothing.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            #[cfg(debug_assertions)]
            report: AuditReport {
                observed: 0,
                verified: 0,
                exempt: 0,
                failures: Vec::new(),
            },
        }
    }

    /// Re-check every witness in `diagnostics` against `facts`, the view of
    /// the consumer that produced them.
    #[cfg_attr(
        not(debug_assertions),
        expect(unused_variables, reason = "the audit only runs in debug builds")
    )]
    pub fn audit(
        &mut self,
        diagnostics: &[Diagnostic],
        facts: &FactView<'_>,
        checks: &WitnessChecks,
    ) {
        #[cfg(debug_assertions)]
        for diagnostic in diagnostics {
            let ordinal = self.report.observed;
            self.report.observed += 1;
            if diagnostic.exemption().is_some() {
                self.report.exempt += 1;
            }
            let Some(chain) = diagnostic.witness_chain() else {
                continue;
            };
            match verify_chain(chain, facts, checks) {
                Ok(()) => self.report.verified += 1,
                Err(error) => {
                    UNVERIFIABLE.fetch_add(1, Ordering::Relaxed);
                    self.report.failures.push(WitnessFailure {
                        diagnostic: ordinal,
                        error,
                    });
                }
            }
        }
    }

    /// What the audit has seen so far; empty in release builds.
    #[must_use]
    pub fn report(&self) -> AuditReport {
        #[cfg(debug_assertions)]
        {
            self.report.clone()
        }
        #[cfg(not(debug_assertions))]
        {
            AuditReport::default()
        }
    }
}

//! Every Patina rule's precision contract, in one table (P4-6c).
//!
//! `assurance.md`: "Every rule declares `exact` / `sound` / `complete` /
//! `heuristic` … the declared domain is part of the rule's contract". The
//! contracts live here rather than beside each rule so lane F keeps sole
//! ownership of `rules/`; `tests.rs` asserts the table's keys are exactly
//! the registered rule names and its severities exactly their
//! `default_severity`, and the rule-parity matrix renders the `tier` column
//! from it.
//!
//! [`RuleContract::new`] rejects a heuristic error at compile time, so the
//! table cannot declare one. One rule's `RuleMeta` still defaults to error
//! while its check is a guess ([`DEMOTED`]): its contract says warning, the
//! unified channel reports it at the contract's severity
//! ([`RuleContract::clamp`]), and the drain belongs to its P4-8 wave.
//!
//! Every error-severity rule reports without a witness today, so each one's
//! [`Exemption`] is counted by `davinci-road/plan/witness-exemptions.tsv`
//! (one row per `Error` row below) — the P4-8 waves' drain queue.
//!
//! Tiers are the initial P4-6c declaration: a structural check the file's
//! own syntax decides is `Exact` over its domain; a check that fires only on
//! a fact it resolves (bindings, macros, types) and cannot see what it does
//! not resolve is `Complete`; a pattern that guesses intent is `Heuristic`.
//! Narrowing a domain or weakening a tier is a breaking change reviewed here.

use vize_davinci::diagnostic::{Exemption, RuleContract, Severity};

macro_rules! row {
    ($name:literal, $tier:ident, $domain:ident, $severity:ident) => {
        RuleEntry::new(
            $name,
            RuleContract::new(Tier::$tier, $domain, Severity::$severity),
        )
    };
}

mod table;
#[cfg(test)]
mod tests;

pub use table::RULE_CONTRACTS;

/// The crate that reports Patina's exemptions.
const PRODUCER: &str = "vize_patina";

/// Rules whose `RuleMeta` defaults to error although the check is
/// heuristic: the contract declares warning, and the drain is the rule's
/// P4-8 wave (demote the default or prove the check).
pub const DEMOTED: &[&str] = &["script/no-potential-component-option-typo"];

/// One rule's contract and the exemption its unwitnessed errors report under.
#[derive(Debug)]
pub struct RuleEntry {
    /// The rule's registered name.
    pub name: &'static str,
    /// Tier, declared domain and default severity.
    pub contract: RuleContract,
    exemption: Exemption,
}

impl RuleEntry {
    const fn new(name: &'static str, contract: RuleContract) -> Self {
        Self {
            name,
            contract,
            exemption: Exemption::new(PRODUCER, name),
        }
    }

    /// The exemption this rule's errors report under — `Some` exactly for
    /// the error-severity rows, which carry no witness yet.
    #[must_use]
    pub fn exemption(&'static self) -> Option<&'static Exemption> {
        matches!(self.contract.severity(), Severity::Error).then_some(&self.exemption)
    }
}

/// The contract registered for `rule`, when it is a Patina rule.
#[must_use]
pub fn contract_for(rule: &str) -> Option<&'static RuleEntry> {
    RULE_CONTRACTS
        .binary_search_by(|entry| entry.name.cmp(rule))
        .ok()
        .map(|index| &RULE_CONTRACTS[index])
}

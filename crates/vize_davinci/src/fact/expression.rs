//! `expression-facts` — P2-5b's expression duties, answered per expression
//! by a foreign expression dialect over the `expression-dialect` contract
//! world (Davinci P6-1b).
//!
//! The in-tree JS dialect answers these questions from the retained AST; a
//! foreign dialect (MoonBit, charter #28) answers them across the contract,
//! and its answer travels as this group's α page. Keys are the batch's
//! expression ids (identities the host assigned, never source offsets);
//! values are the three answers the capability contract names: the
//! referenced bindings in source order, whether that enumeration is exact,
//! and whether the expression is constant.

use vize_s0::{FxHashMap, String};

use super::{AlphaExport, Demand, FactGroup, FactTable, ids};
use crate::folio::Folio;
use crate::pass::AnalysisId;

/// The group: one [`ExpressionFact`] per expression id.
pub struct ExpressionFacts;

/// What a dialect answered about one expression.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExpressionFact {
    /// Referenced binding names in source order, comma-separated (binding
    /// names never contain a comma).
    pub references: String,
    /// Whether `references` is exactly the referenced set, not a lower bound.
    pub exact: bool,
    /// Whether the expression may be folded, hoisted or cached.
    pub constant: bool,
}

impl FactGroup for ExpressionFacts {
    const ID: AnalysisId = ids::EXPRESSION_FACTS;
    const NAME: &'static str = "expression-facts";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = ExpressionFact;
}

/// The α page: the three answers as three id-keyed sections.
#[derive(Debug, Clone, Default, PartialEq, Eq, Folio)]
pub struct ExpressionFactsAlpha {
    pub references: FxHashMap<u32, String>,
    pub exact: FxHashMap<u32, bool>,
    pub constant: FxHashMap<u32, bool>,
}

impl AlphaExport for ExpressionFacts {
    const ALPHA_SCHEMA: u16 = 1;
    type Alpha = ExpressionFactsAlpha;

    fn export(table: &FactTable<Self>) -> ExpressionFactsAlpha {
        let mut alpha = ExpressionFactsAlpha::default();
        for (id, fact) in table.iter() {
            alpha.references.insert(*id, fact.references.clone());
            alpha.exact.insert(*id, fact.exact);
            alpha.constant.insert(*id, fact.constant);
        }
        alpha
    }

    /// Total over any page: an id missing from a section reads as that
    /// section's pessimal answer (no references, inexact, not constant).
    /// Hosts refuse pages whose sections disagree before importing them.
    fn import(alpha: ExpressionFactsAlpha) -> FactTable<Self> {
        let mut ids: alloc::collections::BTreeSet<u32> = alpha.references.keys().copied().collect();
        ids.extend(alpha.exact.keys().copied());
        ids.extend(alpha.constant.keys().copied());
        ids.into_iter()
            .map(|id| {
                let fact = ExpressionFact {
                    references: alpha.references.get(&id).cloned().unwrap_or_default(),
                    exact: alpha.exact.get(&id).copied().unwrap_or(false),
                    constant: alpha.constant.get(&id).copied().unwrap_or(false),
                };
                (id, fact)
            })
            .collect()
    }
}

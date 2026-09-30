//! P4-1a: the `FactManager` computes exactly the demanded closure, in
//! stratum order, each group at most once per artifact, and serves borrowed
//! tables.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests assert by panicking"
)]

mod groups;

use groups::{
    Impostor, PARTIAL, Parity, REGISTRY, SquareRule, Squares, Summary, SummaryRule, Unused,
    UnusedRule, Values, serial, take_log,
};
use vize_davinci::fact::{
    Demand, FactConsumer, FactError, FactGroup, FactManager, FactTable, produced_count,
};

const ARTIFACT: &[u32] = &[1, 2, 3, 4];

fn counts() -> [u64; 5] {
    [
        produced_count(Values::ID),
        produced_count(Unused::ID),
        produced_count(Squares::ID),
        produced_count(Parity::ID),
        produced_count(Summary::ID),
    ]
}

fn delta(before: [u64; 5], after: [u64; 5]) -> [u64; 5] {
    std::array::from_fn(|i| after[i] - before[i])
}

#[test]
fn a_run_computes_exactly_the_closure_in_stratum_order() {
    let _serial = serial();
    take_log();
    let mut manager = FactManager::new(&REGISTRY);
    let produced = manager.compute(ARTIFACT, SummaryRule::DEMAND).unwrap();
    let closure = Demand::NONE
        .with(Values::ID)
        .with(Squares::ID)
        .with(Parity::ID)
        .with(Summary::ID);
    assert_eq!(produced, closure);
    assert_eq!(manager.computed(), closure);
    assert_eq!(take_log(), ["values", "squares", "parity", "summary"]);
}

#[test]
fn each_group_is_produced_once_per_artifact() {
    let _serial = serial();
    let before = counts();
    let mut manager = FactManager::new(&REGISTRY);
    manager.compute(ARTIFACT, SummaryRule::DEMAND).unwrap();
    // Overlapping and repeated demands reuse what is already computed.
    assert_eq!(
        manager.compute(ARTIFACT, SquareRule::DEMAND),
        Ok(Demand::NONE)
    );
    assert_eq!(
        manager.compute(ARTIFACT, SummaryRule::DEMAND.union(SquareRule::DEMAND)),
        Ok(Demand::NONE)
    );
    assert_eq!(delta(before, counts()), [1, 0, 1, 1, 1]);

    // A second artifact is a second manager: one more production each.
    let mut other = FactManager::new(&REGISTRY);
    other.compute(&[5, 6][..], SquareRule::DEMAND).unwrap();
    assert_eq!(delta(before, counts()), [2, 0, 2, 1, 1]);
}

#[test]
fn queries_borrow_the_exact_tables() {
    let _serial = serial();
    let mut manager = FactManager::new(&REGISTRY);
    let view = manager.prepare::<SummaryRule>(ARTIFACT).unwrap();
    let summary = view.get::<Summary>().unwrap();
    let expected: FactTable<Summary> = [(0, 4 + 16)].into_iter().collect();
    assert_eq!(summary, &expected);

    let view = manager.view::<SquareRule>();
    let squares: Vec<(u32, u64)> = view
        .get::<Squares>()
        .unwrap()
        .iter()
        .map(|(key, value)| (*key, *value))
        .collect();
    assert_eq!(squares, [(0, 1), (1, 4), (2, 9), (3, 16)]);
}

#[test]
fn an_unregistered_demand_is_refused_before_anything_runs() {
    let _serial = serial();
    take_log();
    let mut manager = FactManager::new(&PARTIAL);
    assert_eq!(
        manager.compute(ARTIFACT, UnusedRule::DEMAND.with(Values::ID)),
        Err(FactError::Unregistered { group: Unused::ID })
    );
    assert_eq!(manager.computed(), Demand::NONE);
    assert_eq!(take_log(), Vec::<&str>::new());
}

#[test]
fn a_declared_but_uncomputed_group_is_not_computed() {
    let manager = FactManager::new(&REGISTRY);
    let view = manager.view::<SquareRule>();
    assert_eq!(
        view.get::<Squares>().map(FactTable::len),
        Err(FactError::NotComputed { group: Squares::ID })
    );
}

/// The consumer of a group whose id another group type also claims.
struct ImpostorRule;
impl FactConsumer for ImpostorRule {
    const NAME: &'static str = "impostor-rule";
    const DEMAND: Demand = Demand::NONE.with(Impostor::ID);
}

#[test]
fn a_query_through_the_wrong_group_type_is_a_type_mismatch() {
    let _serial = serial();
    let mut manager = FactManager::new(&REGISTRY);
    manager.compute(ARTIFACT, SquareRule::DEMAND).unwrap();
    let view = manager.view::<ImpostorRule>();
    assert_eq!(
        view.get::<Impostor>().map(FactTable::len),
        Err(FactError::TypeMismatch {
            group: Squares::ID,
            expected: "fact_manager::groups::Impostor",
        })
    );
}

#[test]
fn an_empty_demand_computes_nothing() {
    let mut manager = FactManager::new(&REGISTRY);
    assert_eq!(manager.compute(ARTIFACT, Demand::NONE), Ok(Demand::NONE));
    assert_eq!(manager.computed(), Demand::NONE);
}

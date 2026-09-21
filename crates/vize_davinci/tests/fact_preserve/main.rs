//! P4-1b: facts across passes — post-hoc `Preserved` invalidation, the
//! named preservation groups, and the verify mode that rejects a pass
//! lying about what it preserves.
//!
//! `cargo test -p vize_davinci --test fact_preserve`

mod numbers;

use numbers::{
    BindingsLike, Count, DOUBLE, Everything, LYING_DOUBLE, REGISTRY, RENAME, Total, UsagesLike,
    Values, double, serial,
};
use vize_davinci::fact::{
    Demand, FactConsumer, FactError, FactGroup, FactManager, FactTable, FactVerifyObserver,
    NoFactVerify, PRESERVE_BINDINGS, PRESERVE_STRUCTURE, ids, produced_count,
};
use vize_davinci::pass::{Fusability, PassDesc, PassKind, Preserved};

fn total(manager: &FactManager<'_, Vec<u32>>) -> Result<u64, FactError> {
    let view = manager.view::<Everything>();
    Ok(*view.get::<Total>()?.get(&()).expect("one total"))
}

#[test]
fn a_pass_that_preserves_everything_keeps_every_table() {
    // `rename` changes nothing the facts read, and says so.
    let _serial = serial();
    let numbers = vec![1, 2, 3];
    let mut manager = FactManager::new(&REGISTRY);
    manager.compute(&numbers, Everything::DEMAND).unwrap();
    assert_eq!(
        manager.after_pass::<FactVerifyObserver>(&numbers, &RENAME),
        Ok(Demand::NONE)
    );
    assert_eq!(manager.computed(), Everything::DEMAND);
    assert_eq!(total(&manager), Ok(6));
}

#[test]
fn a_pass_drops_exactly_what_it_does_not_preserve() {
    let _serial = serial();
    let mut numbers = vec![1, 2, 3];
    let mut manager = FactManager::new(&REGISTRY);
    manager.compute(&numbers, Everything::DEMAND).unwrap();

    double(&mut numbers);
    let dropped = manager.after_pass::<FactVerifyObserver>(&numbers, &DOUBLE);
    assert_eq!(dropped, Ok(Demand::NONE.with(Values::ID).with(Total::ID)));
    assert_eq!(
        manager.computed(),
        Demand::NONE
            .with(Count::ID)
            .with(BindingsLike::ID)
            .with(UsagesLike::ID)
    );
    assert_eq!(
        total(&manager),
        Err(FactError::NotComputed { group: Total::ID })
    );

    // The next demand recomputes only what the pass dropped, on the new
    // artifact state; verification recomputes are not productions.
    let before = [produced_count(Values::ID), produced_count(Count::ID)];
    assert_eq!(
        manager.compute(&numbers, Everything::DEMAND),
        Ok(Demand::NONE.with(Values::ID).with(Total::ID))
    );
    let after = [produced_count(Values::ID), produced_count(Count::ID)];
    assert_eq!([after[0] - before[0], after[1] - before[1]], [1, 0]);
    assert_eq!(total(&manager), Ok(12));
}

#[cfg(debug_assertions)]
#[test]
fn a_pass_lying_about_what_it_preserves_is_rejected_exactly() {
    let _serial = serial();
    let mut numbers = vec![1, 2, 3];
    let mut manager = FactManager::new(&REGISTRY);
    manager.compute(&numbers, Everything::DEMAND).unwrap();

    double(&mut numbers);
    assert_eq!(
        manager.after_pass::<FactVerifyObserver>(&numbers, &LYING_DOUBLE),
        Err(FactError::StalePreserved {
            pass: "lying-double",
            group: Values::ID,
        })
    );
    // Every stale table is gone (values and the total read off it); the
    // groups the lie happened not to break are kept.
    assert_eq!(
        manager.computed(),
        Demand::NONE
            .with(Count::ID)
            .with(BindingsLike::ID)
            .with(UsagesLike::ID)
    );
    manager.compute(&numbers, Everything::DEMAND).unwrap();
    assert_eq!(total(&manager), Ok(12));
}

#[test]
fn trusting_mode_keeps_what_a_pass_claims() {
    let _serial = serial();
    let mut numbers = vec![1, 2, 3];
    let mut manager = FactManager::new(&REGISTRY);
    manager.compute(&numbers, Everything::DEMAND).unwrap();
    double(&mut numbers);
    assert_eq!(
        manager.after_pass::<NoFactVerify>(&numbers, &LYING_DOUBLE),
        Ok(Demand::NONE)
    );
    // Stale by construction: this is what verify mode exists to catch.
    assert_eq!(total(&manager), Ok(6));
}

#[cfg(not(debug_assertions))]
#[test]
fn release_verify_mode_is_the_trusting_mode() {
    let _serial = serial();
    let mut numbers = vec![1, 2, 3];
    let mut manager = FactManager::new(&REGISTRY);
    manager.compute(&numbers, Everything::DEMAND).unwrap();
    double(&mut numbers);
    assert_eq!(
        manager.after_pass::<FactVerifyObserver>(&numbers, &LYING_DOUBLE),
        Ok(Demand::NONE)
    );
}

#[test]
fn the_named_preservation_groups_are_exact_id_sets() {
    assert_eq!(
        PRESERVE_STRUCTURE,
        Preserved::NONE
            .with(ids::COMPONENT_USAGES)
            .with(ids::RENDER_TREE)
    );
    assert_eq!(
        PRESERVE_BINDINGS,
        Preserved::NONE
            .with(ids::BINDINGS)
            .with(ids::UNDEFINED_REFS)
            .with(ids::UNUSED_BINDINGS)
            .with(ids::REACTIVITY)
    );
    assert_eq!(
        PRESERVE_STRUCTURE.intersect(PRESERVE_BINDINGS),
        Preserved::NONE
    );
}

/// A structural pass (a hoist) that keeps every binding fact.
const HOIST: PassDesc = PassDesc::new(
    "hoist",
    PassKind::Optional,
    Fusability::Fusable,
    PRESERVE_BINDINGS,
);

#[test]
fn a_named_group_keeps_its_members_and_drops_the_rest() {
    let _serial = serial();
    let numbers = vec![4, 5];
    let mut manager = FactManager::new(&REGISTRY);
    manager.compute(&numbers, Everything::DEMAND).unwrap();
    assert_eq!(
        manager.after_pass::<FactVerifyObserver>(&numbers, &HOIST),
        Ok(Everything::DEMAND.minus(Demand::NONE.with(BindingsLike::ID)))
    );
    assert_eq!(manager.computed(), Demand::NONE.with(BindingsLike::ID));
    let view = manager.view::<Everything>();
    assert_eq!(
        view.get::<UsagesLike>().map(FactTable::len),
        Err(FactError::NotComputed {
            group: UsagesLike::ID
        })
    );
    assert_eq!(view.get::<BindingsLike>().map(FactTable::len), Ok(1));
}

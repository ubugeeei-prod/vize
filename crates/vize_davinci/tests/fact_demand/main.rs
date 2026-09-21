//! TS-35 — fact demand enforcement: zero undeclared accesses over the
//! declared consumers, the debug detector's exact refusal (and counter) on
//! an undeclared read, and a demand graph acyclic by strata.
//!
//! `cargo test -p vize_davinci --test fact_demand`

mod words;

use vize_davinci::fact::{
    Demand, FactConsumer, FactError, FactGroup, FactManager, FactTable, GroupDesc, StrataError,
    check_strata, undeclared_accesses,
};
use vize_davinci::pass::AnalysisId;
use words::{
    Lengths, Longest, LongestRule, REGISTRY, SneakyRule, VowelRule, Vowels, Words, serial,
};

const WORDS: &Words = &["fact", "demand", "view", "queue"];

#[test]
fn declared_consumers_make_zero_undeclared_accesses() {
    let _serial = serial();
    let before = undeclared_accesses();
    let mut manager = FactManager::new(&REGISTRY, WORDS);
    manager
        .compute(LongestRule::DEMAND.union(VowelRule::DEMAND))
        .unwrap();

    let longest = manager.view::<LongestRule>();
    assert_eq!(longest.get::<Longest>().unwrap().get(&()), Some(&1));
    let vowels = manager.view::<VowelRule>();
    let counts: Vec<usize> = vowels
        .get::<Vowels>()
        .unwrap()
        .iter()
        .map(|(_, count)| *count)
        .collect();
    assert_eq!(counts, [1, 2, 2, 4]);

    assert_eq!(undeclared_accesses() - before, 0);
}

#[cfg(debug_assertions)]
#[test]
fn an_undeclared_read_is_refused_exactly_even_when_computed() {
    let _serial = serial();
    let before = undeclared_accesses();
    let mut manager = FactManager::new(&REGISTRY, WORDS);
    // `Vowels` is computed — the vowel rule demands it …
    manager
        .compute(VowelRule::DEMAND.union(SneakyRule::DEMAND))
        .unwrap();
    assert_eq!(
        manager.computed(),
        Demand::NONE
            .with(Lengths::ID)
            .with(Vowels::ID)
            .with(Longest::ID)
    );
    // … but the sneaky rule never declared it.
    let view = manager.view::<SneakyRule>();
    assert_eq!(
        view.get::<Vowels>().map(FactTable::len),
        Err(FactError::Undeclared {
            consumer: "sneaky-rule",
            group: Vowels::ID,
        })
    );
    assert_eq!(view.get::<Longest>().map(FactTable::len), Ok(1));
    assert_eq!(undeclared_accesses() - before, 1);
}

#[cfg(debug_assertions)]
#[test]
fn a_producer_reading_outside_its_depends_is_refused_too() {
    let _serial = serial();
    let before = undeclared_accesses();
    let mut manager = FactManager::new(&REGISTRY, WORDS);
    // `Vowels` (stratum 0) is computed before `Leaky` (stratum 1) runs, so
    // only the detector stands between the producer and the table.
    manager
        .compute(VowelRule::DEMAND.union(words::LeakyRule::DEMAND))
        .unwrap();
    let view = manager.view::<words::LeakyRule>();
    let seen = view.get::<words::Leaky>().unwrap().get(&()).unwrap();
    assert_eq!(
        seen.as_str(),
        "Err(Undeclared { consumer: \"leaky\", group: AnalysisId(21) })"
    );
    assert_eq!(undeclared_accesses() - before, 1);
}

/// Release builds carry no detector: the declared demand is a zero-sized
/// type and a read succeeds whenever the table exists.
#[cfg(not(debug_assertions))]
#[test]
fn release_builds_carry_no_detector() {
    let mut manager = FactManager::new(&REGISTRY, WORDS);
    manager
        .compute(VowelRule::DEMAND.union(SneakyRule::DEMAND))
        .unwrap();
    let view = manager.view::<SneakyRule>();
    assert_eq!(view.get::<Vowels>().map(FactTable::len), Ok(4));
    assert_eq!(undeclared_accesses(), 0);
}

fn desc(id: u8, stratum: u8, depends: Demand) -> GroupDesc {
    GroupDesc {
        id: AnalysisId::new(id),
        name: "g",
        stratum,
        depends,
    }
}

#[test]
fn the_fixture_graph_is_acyclic_by_strata() {
    let descs: Vec<GroupDesc> = REGISTRY
        .producers()
        .iter()
        .map(|entry| entry.desc)
        .collect();
    assert_eq!(check_strata(&descs), Ok(()));
    let strata: Vec<(&str, u8)> = descs.iter().map(|d| (d.name, d.stratum)).collect();
    assert_eq!(
        strata,
        [("lengths", 0), ("vowels", 0), ("longest", 1), ("leaky", 1)]
    );
    assert_eq!(
        REGISTRY.closure(LongestRule::DEMAND),
        Demand::NONE.with(Lengths::ID).with(Longest::ID)
    );
}

#[test]
fn the_stratification_check_names_each_violation_exactly() {
    let a = AnalysisId::new(0);
    let b = AnalysisId::new(1);
    let same_stratum = [desc(0, 1, Demand::NONE), desc(1, 1, Demand::NONE.with(a))];
    assert_eq!(
        check_strata(&same_stratum),
        Err(StrataError::NotStrictlyLower {
            group: b,
            dependency: a,
        })
    );
    let upward = [desc(0, 0, Demand::NONE.with(b)), desc(1, 1, Demand::NONE)];
    assert_eq!(
        check_strata(&upward),
        Err(StrataError::NotStrictlyLower {
            group: a,
            dependency: b,
        })
    );
    let itself = [desc(0, 3, Demand::NONE.with(a))];
    assert_eq!(
        check_strata(&itself),
        Err(StrataError::NotStrictlyLower {
            group: a,
            dependency: a,
        })
    );
    let dangling = [desc(0, 1, Demand::NONE.with(b))];
    assert_eq!(
        check_strata(&dangling),
        Err(StrataError::UnregisteredDependency {
            group: a,
            dependency: b,
        })
    );
    let twice = [desc(0, 0, Demand::NONE), desc(0, 1, Demand::NONE)];
    assert_eq!(
        check_strata(&twice),
        Err(StrataError::DuplicateId { id: a })
    );
}

#[test]
fn every_fixture_consumer_declares_a_registered_closure() {
    for (name, demand) in [
        (LongestRule::NAME, LongestRule::DEMAND),
        (VowelRule::NAME, VowelRule::DEMAND),
        (SneakyRule::NAME, SneakyRule::DEMAND),
        (words::LeakyRule::NAME, words::LeakyRule::DEMAND),
    ] {
        assert_eq!(
            REGISTRY.closure(demand).minus(REGISTRY.registered()),
            Demand::NONE,
            "{name} demands an unregistered group"
        );
    }
}

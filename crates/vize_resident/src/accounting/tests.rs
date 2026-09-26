//! Long sessions keep exact counts without retaining every event.
use super::{QueryCounts, Recorder, tally};
use salsa::{Database as _, Setter as _};
use vize_l0::String;

#[salsa::db]
#[derive(Clone)]
struct TestDatabase {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for TestDatabase {}

#[salsa::input]
struct Tick {
    #[returns(copy)]
    value: u32,
}

#[salsa::tracked(returns(copy))]
fn measured(db: &dyn salsa::Database, tick: Tick) -> u32 {
    tick.value(db)
}

#[test]
fn ten_thousand_undrained_revisions_use_one_accounting_row() {
    let recorder = Recorder::default();
    let mut db = TestDatabase {
        storage: salsa::Storage::new(Some(recorder.callback())),
    };
    let tick = Tick::new(&db, 0);
    assert_eq!(measured(&db, tick), 0);
    for value in 1..=10_000 {
        tick.set_value(&mut db).to(value);
        assert_eq!(measured(&db, tick), value);
    }
    let records = recorder.drain();
    assert_eq!(records.len(), 1);
    let rows = tally(records, |index| {
        String::from(db.ingredient_debug_name(index).as_ref())
    });
    assert_eq!(
        rows.get("measured"),
        Some(&QueryCounts {
            executed: 10_001,
            reused: 0
        })
    );
    assert!(recorder.drain().is_empty());
    tick.set_value(&mut db).to(10_001);
    assert_eq!(measured(&db, tick), 10_001);
    let rows = tally(recorder.drain(), |index| {
        String::from(db.ingredient_debug_name(index).as_ref())
    });
    assert_eq!(
        rows.get("measured"),
        Some(&QueryCounts {
            executed: 1,
            reused: 0
        })
    );
}

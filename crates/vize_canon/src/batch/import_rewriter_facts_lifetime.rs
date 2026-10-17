//! Test-only ownership receipts for the actual preparation/store boundary.

use std::cell::{Cell, RefCell};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Stats {
    pub(crate) created: usize,
    pub(crate) dropped: usize,
    pub(crate) live: usize,
    pub(crate) checkpoints: usize,
}

struct Active {
    id: u64,
    stats: Stats,
}

thread_local! {
    static OBSERVER: RefCell<Option<Active>> = const { RefCell::new(None) };
    static NEXT_ID: Cell<u64> = const { Cell::new(0) };
}

pub(crate) struct Observation(u64);

pub(super) struct ObservedLifetime(u64);

pub(super) fn observe() -> Observation {
    let id = NEXT_ID.with(|next| {
        let id = next.get() + 1;
        next.set(id);
        id
    });
    OBSERVER.with_borrow_mut(|observer| {
        assert!(
            observer.is_none(),
            "preparation lifetime observations cannot nest"
        );
        *observer = Some(Active {
            id,
            stats: Stats::default(),
        });
    });
    Observation(id)
}

pub(super) fn created() -> Option<ObservedLifetime> {
    OBSERVER.with_borrow_mut(|observer| {
        let state = observer.as_mut()?;
        state.stats.created += 1;
        state.stats.live += 1;
        Some(ObservedLifetime(state.id))
    })
}

pub(super) fn checkpoint() {
    OBSERVER.with_borrow_mut(|observer| {
        if let Some(state) = observer {
            assert_eq!(
                state.stats.live, 0,
                "syntax facts escaped the build/stamp scope"
            );
            state.stats.checkpoints += 1;
        }
    });
}

impl Observation {
    pub(crate) fn stats(&self) -> Stats {
        OBSERVER.with_borrow(|observer| {
            let state = observer.as_ref().expect("active lifetime observation");
            assert_eq!(state.id, self.0);
            state.stats
        })
    }
}

impl Drop for Observation {
    fn drop(&mut self) {
        OBSERVER.with_borrow_mut(|observer| {
            let state = observer.take().expect("active lifetime observation");
            assert_eq!(state.id, self.0);
        });
    }
}

impl Drop for ObservedLifetime {
    fn drop(&mut self) {
        OBSERVER.with_borrow_mut(|observer| {
            if let Some(state) = observer.as_mut().filter(|state| state.id == self.0) {
                state.stats.live -= 1;
                state.stats.dropped += 1;
            }
        });
    }
}

#[test]
fn ownership_checkpoint_rejects_live_facts_and_accepts_their_drop() {
    let observation = observe();
    let facts = super::ModuleSpecifierFactsCache::default();
    assert!(std::panic::catch_unwind(checkpoint).is_err());
    assert_eq!(observation.stats().live, 1);
    drop(facts);
    checkpoint();
    assert_eq!(
        observation.stats(),
        Stats {
            created: 1,
            dropped: 1,
            live: 0,
            checkpoints: 1,
        }
    );
}

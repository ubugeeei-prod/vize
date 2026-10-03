//! Cache-hit accounting: which queries salsa executed and which memos it
//! reused, read from salsa's own event stream rather than from counters
//! hand-placed in query bodies — so the accounting cannot disagree with what
//! the engine did.
//!
//! A **reuse** is `DidValidateMemoizedValue`: a memo from an earlier
//! revision proven still valid without running its body — the cache hit
//! TS-46 accounts. A repeat read inside one revision is not an event (salsa
//! answers it from the already-verified memo) and is not counted.

use std::collections::BTreeMap;
use std::sync::{Mutex, PoisonError};

use salsa::{Event, EventKind, IngredientIndex};
use vize_l0::String;

/// Executions and reuses of one query since the last
/// [`take_accounting`](crate::ResidentDatabase::take_accounting).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct QueryCounts {
    /// Bodies run (memo absent or invalidated).
    pub executed: u32,
    /// Memos from an earlier revision reused after validation.
    pub reused: u32,
}

/// Per-query counts, keyed by the query's name.
pub type Accounting = BTreeMap<String, QueryCounts>;

/// One bounded row per registered query ingredient, independent of event volume.
pub(crate) type Records = BTreeMap<IngredientIndex, QueryCounts>;

/// The event sink the database installs in its storage. The records are
/// shared between the storage's callback and the database handle, which
/// salsa clones per snapshot — hence the shared owner.
#[derive(Clone, Default)]
pub(crate) struct Recorder {
    #[expect(
        clippy::disallowed_types,
        reason = "shared between the storage's event callback and every database clone \
                  (salsa clones the handle per snapshot); a scoped reference cannot outlive \
                  the storage that owns the callback"
    )]
    records: std::sync::Arc<Mutex<Records>>,
}

impl Recorder {
    /// The callback salsa's storage invokes on every event.
    pub(crate) fn callback(&self) -> Box<dyn Fn(Event) + Send + Sync + 'static> {
        let records = self.records.clone();
        Box::new(move |event| {
            let record = match event.kind {
                EventKind::WillExecute { database_key } => (database_key.ingredient_index(), true),
                EventKind::DidValidateMemoizedValue { database_key } => {
                    (database_key.ingredient_index(), false)
                }
                _ => return,
            };
            let mut records = records.lock().unwrap_or_else(PoisonError::into_inner);
            let counts = records.entry(record.0).or_default();
            if record.1 {
                counts.executed = counts.executed.saturating_add(1);
            } else {
                counts.reused = counts.reused.saturating_add(1);
            }
        })
    }

    /// Take every record since the last drain.
    pub(crate) fn drain(&self) -> Records {
        core::mem::take(&mut *self.records.lock().unwrap_or_else(PoisonError::into_inner))
    }
}

/// Fold records into per-query counts, naming each ingredient with `name`.
pub(crate) fn tally(records: Records, name: impl Fn(IngredientIndex) -> String) -> Accounting {
    let mut accounting = Accounting::new();
    for (ingredient, recorded) in records {
        let counts = accounting.entry(name(ingredient)).or_default();
        counts.executed = counts.executed.saturating_add(recorded.executed);
        counts.reused = counts.reused.saturating_add(recorded.reused);
    }
    accounting
}

#[cfg(test)]
mod tests;

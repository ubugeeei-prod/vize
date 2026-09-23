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
use vize_s0::String;

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

/// Execution (`true`) or reuse (`false`) of one ingredient.
pub(crate) type Record = (IngredientIndex, bool);

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
    records: std::sync::Arc<Mutex<Vec<Record>>>,
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
            records
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(record);
        })
    }

    /// Take every record since the last drain.
    pub(crate) fn drain(&self) -> Vec<Record> {
        core::mem::take(&mut *self.records.lock().unwrap_or_else(PoisonError::into_inner))
    }
}

/// Fold records into per-query counts, naming each ingredient with `name`.
pub(crate) fn tally(records: Vec<Record>, name: impl Fn(IngredientIndex) -> String) -> Accounting {
    let mut accounting = Accounting::new();
    for (ingredient, executed) in records {
        let counts = accounting.entry(name(ingredient)).or_default();
        if executed {
            counts.executed += 1;
        } else {
            counts.reused += 1;
        }
    }
    accounting
}

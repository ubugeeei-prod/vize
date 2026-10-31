//! Bounded weak publication order; never owns lexical facts or reader leases.

use std::sync::{Arc, Weak};

use super::{Cell, Entry, MAX_ENTRIES};

#[derive(Clone)]
pub(super) struct Publication {
    cell: Weak<Cell>,
    entry: Weak<Entry>,
}

#[derive(Default)]
pub(super) struct Registry {
    rows: [Option<Publication>; MAX_ENTRIES],
}

impl Registry {
    pub(super) fn register(&mut self, cell: &Arc<Cell>, entry: &Arc<Entry>) -> bool {
        // At most MAX_ENTRIES live entries hold leases process-wide. Remove
        // expired identities while preserving the order of live publications.
        let mut live: [Option<Publication>; MAX_ENTRIES] = std::array::from_fn(|_| None);
        let mut count = 0;
        for row in self.rows.iter().flatten() {
            if row.entry.strong_count() != 0 {
                let Some(slot) = live.get_mut(count) else {
                    return false;
                };
                *slot = Some(row.clone());
                count += 1;
            }
        }
        self.rows = live;
        let identity = Arc::downgrade(entry);
        if self
            .rows
            .iter()
            .flatten()
            .any(|row| Weak::ptr_eq(&row.entry, &identity))
        {
            return true;
        }
        let Some(slot) = self.rows.get_mut(count) else {
            return false;
        };
        *slot = Some(Publication {
            cell: Arc::downgrade(cell),
            entry: identity,
        });
        true
    }

    pub(super) fn snapshot(&self) -> [Option<Publication>; MAX_ENTRIES] {
        self.rows.clone()
    }

    pub(super) fn evict_oldest_idle(rows: [Option<Publication>; MAX_ENTRIES]) -> bool {
        for row in rows.into_iter().flatten() {
            let Some(cell) = row.cell.upgrade() else {
                continue;
            };
            let mut cached = cell.entry.lock();
            let Some(entry) = cached.as_ref() else {
                continue;
            };
            // The cell lock excludes new readers. Never upgrade entry's weak
            // identity: that would itself add a reader to the strong count.
            if !Weak::ptr_eq(&row.entry, &Arc::downgrade(entry)) || Arc::strong_count(entry) != 1 {
                continue;
            }
            let removed = cached.take();
            drop(cached);
            // Entry fields (including facts) drop before its final lease.
            // No registry/cell lock is held while either allocation drops.
            drop(removed);
            return true;
        }
        false
    }
}

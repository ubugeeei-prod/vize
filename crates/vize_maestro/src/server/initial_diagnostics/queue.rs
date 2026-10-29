//! Canonical URI ownership plus bounded, reusable scheduler slot indexes.

use super::pending::InitialDiagnosticsJob;
use std::time::Instant;
use tower_lsp::lsp_types::Url;
use vize_l0::FxHashMap;

mod heap;
use heap::IndexedHeap;

// The lookup table and its reusable slot share this single URI allocation.
// No Arc is added to the diagnostic/continuation payload.
#[expect(
    clippy::disallowed_types,
    reason = "one canonical URI is owned jointly by its lookup key and reusable queue slot"
)]
type SharedUri = std::sync::Arc<Url>;

struct Slot {
    uri: SharedUri,
    job: InitialDiagnosticsJob,
}

#[derive(Default)]
pub(super) struct JobQueue {
    by_uri: FxHashMap<SharedUri, usize>,
    slots: Vec<Option<Slot>>,
    free: Vec<usize>,
    sync: IndexedHeap<u64>,
    preparation: IndexedHeap<u64>,
    // Equal deadlines refine the old unspecified hash iteration to FIFO.
    ready: IndexedHeap<(Instant, u64)>,
}

impl JobQueue {
    pub(super) fn get(&self, uri: &Url) -> Option<&InitialDiagnosticsJob> {
        let slot = *self.by_uri.get(uri)?;
        self.slots.get(slot)?.as_ref().map(|slot| &slot.job)
    }

    pub(super) fn contains_key(&self, uri: &Url) -> bool {
        self.by_uri.contains_key(uri)
    }

    pub(super) fn insert(&mut self, uri: Url, job: InitialDiagnosticsJob) {
        if let Some(slot) = self.by_uri.get(&uri).copied() {
            self.unindex(slot);
            if let Some(Some(entry)) = self.slots.get_mut(slot) {
                entry.job = job;
                self.index(slot);
                return;
            }
            self.by_uri.remove(&uri);
        }
        let slot = self.free.pop().unwrap_or(self.slots.len());
        let uri = SharedUri::new(uri);
        self.by_uri.insert(SharedUri::clone(&uri), slot);
        let entry = Some(Slot { uri, job });
        if let Some(value) = self.slots.get_mut(slot) {
            *value = entry;
        } else {
            self.slots.push(entry);
        }
        self.index(slot);
    }

    pub(super) fn remove(&mut self, uri: &Url) -> Option<InitialDiagnosticsJob> {
        let slot = self.by_uri.remove(uri)?;
        self.unindex(slot);
        let entry = self.slots.get_mut(slot)?.take()?;
        self.free.push(slot);
        Some(entry.job)
    }

    pub(super) fn clear(&mut self) {
        self.by_uri.clear();
        self.slots.clear();
        self.free.clear();
        self.sync.clear();
        self.preparation.clear();
        self.ready.clear();
    }

    pub(super) fn skip_sync(&mut self, uri: &Url) {
        let Some(slot) = self.by_uri.get(uri).copied() else {
            return;
        };
        self.unindex(slot);
        if let Some(Some(entry)) = self.slots.get_mut(slot) {
            entry.job.sync_pending = false;
        }
        self.index(slot);
    }

    pub(super) fn take_sync(&mut self) -> Option<(Url, InitialDiagnosticsJob)> {
        let (slot, _) = self.sync.peek()?;
        self.unindex(slot);
        let entry = self.slots.get_mut(slot)?.as_mut()?;
        entry.job.sync_pending = false;
        entry.job.sync_running = true;
        let selected = (entry.uri.as_ref().clone(), entry.job.clone());
        self.index(slot);
        Some(selected)
    }

    pub(super) fn complete_sync(&mut self, uri: &Url, sequence: u64) {
        let Some(slot) = self.by_uri.get(uri).copied() else {
            return;
        };
        let Some(Some(entry)) = self.slots.get_mut(slot) else {
            return;
        };
        if entry.job.sequence != sequence {
            return;
        }
        entry.job.sync_running = false;
        self.index(slot);
    }

    pub(super) fn take_preparation(&mut self) -> Option<(Url, InitialDiagnosticsJob)> {
        let (slot, _) = self.preparation.peek()?;
        self.preparation.remove(slot);
        let entry = self.slots.get_mut(slot)?.as_mut()?;
        entry.job.preparation_pending = false;
        Some((entry.uri.as_ref().clone(), entry.job.clone()))
    }

    pub(super) fn next_not_before(&self) -> Option<Instant> {
        self.ready.peek().map(|(_, key)| key.0)
    }

    pub(super) fn take_ready(&mut self, now: Instant) -> Option<(Url, InitialDiagnosticsJob)> {
        let (slot, (not_before, _)) = self.ready.peek()?;
        if not_before > now {
            return None;
        }
        let uri = self.slots.get(slot)?.as_ref()?.uri.as_ref().clone();
        self.remove(&uri).map(|job| (uri, job))
    }

    fn unindex(&mut self, slot: usize) {
        self.sync.remove(slot);
        self.preparation.remove(slot);
        self.ready.remove(slot);
    }

    fn index(&mut self, slot: usize) {
        let Some(Some(entry)) = self.slots.get(slot) else {
            return;
        };
        let job = &entry.job;
        if job.sync_pending {
            self.sync.insert(slot, job.sequence);
        }
        if !job.sync_pending && !job.sync_running {
            if job.preparation_pending {
                self.preparation.insert(slot, job.sequence);
            }
            self.ready.insert(slot, (job.not_before, job.sequence));
        }
    }
}

#[cfg(test)]
impl JobQueue {
    pub(super) fn len(&self) -> usize {
        self.by_uri.len()
    }
    pub(super) fn is_empty(&self) -> bool {
        self.by_uri.is_empty()
    }
    #[cfg(feature = "micro-profiling")]
    pub(super) fn capacity(&self) -> usize {
        self.by_uri.capacity()
    }
    #[cfg(feature = "micro-profiling")]
    pub(super) fn keys(&self) -> impl Iterator<Item = &Url> {
        self.by_uri.keys().map(AsRef::as_ref)
    }

    pub(super) fn assert_slots(&self, expected: usize) {
        assert_eq!(self.slots.len(), expected);
    }

    pub(super) fn verify(&self) {
        self.sync.verify();
        self.preparation.verify();
        self.ready.verify();
        assert_eq!(self.by_uri.len() + self.free.len(), self.slots.len());
        for (uri, &slot) in &self.by_uri {
            let entry = self.slots[slot].as_ref().unwrap();
            assert_eq!(uri.as_ref(), entry.uri.as_ref());
            let job = &entry.job;
            assert_eq!(
                self.sync.key(slot),
                job.sync_pending.then_some(job.sequence)
            );
            let eligible = !job.sync_pending && !job.sync_running;
            assert_eq!(
                self.preparation.key(slot),
                (eligible && job.preparation_pending).then_some(job.sequence)
            );
            assert_eq!(
                self.ready.key(slot),
                eligible.then_some((job.not_before, job.sequence))
            );
        }
        for &slot in &self.free {
            assert!(self.slots[slot].is_none());
        }
        let mut free = self.free.clone();
        free.sort_unstable();
        free.dedup();
        assert_eq!(free.len(), self.free.len());
        let live = self.sync.storage().0 + self.preparation.storage().0 + self.ready.storage().0;
        assert!(live <= 2 * self.by_uri.len());
    }

    #[cfg(feature = "micro-profiling")]
    pub(super) fn storage(&self) -> serde_json::Value {
        serde_json::json!({
            "map_capacity_api": self.by_uri.capacity(),
            "slot_layout_bytes": std::mem::size_of::<Option<Slot>>(),
            "map_entry_layout_bytes": std::mem::size_of::<(SharedUri, usize)>(),
            "heap_storage_tuple_fields": ["live_nodes", "node_capacity", "position_slots", "position_capacity"],
            "slots_len": self.slots.len(), "slots_capacity": self.slots.capacity(),
            "free_len": self.free.len(), "free_capacity": self.free.capacity(),
            "sync": self.sync.storage(), "preparation": self.preparation.storage(), "ready": self.ready.storage(),
        })
    }
}

#[cfg(test)]
impl std::ops::Index<&Url> for JobQueue {
    type Output = InitialDiagnosticsJob;
    fn index(&self, uri: &Url) -> &Self::Output {
        self.get(uri).expect("a test asserted this URI was pending")
    }
}

#[cfg(all(test, feature = "micro-profiling"))]
impl JobQueue {
    pub(super) fn census(&self) -> [heap::Census; 3] {
        [
            self.sync.census(),
            self.preparation.census(),
            self.ready.census(),
        ]
    }
    pub(super) fn reset_census(&self) {
        self.sync.reset_census();
        self.preparation.reset_census();
        self.ready.reset_census();
    }
}

#[cfg(test)]
mod tests;

//! Indexed binary min-heap: one live node per reusable scheduler slot.

use std::num::NonZeroUsize;

struct Entry<K> {
    slot: usize,
    key: K,
}

pub(super) struct IndexedHeap<K> {
    entries: Vec<Entry<K>>,
    positions: Vec<Option<NonZeroUsize>>,
    #[cfg(all(test, feature = "micro-profiling"))]
    census: std::cell::Cell<Census>,
}

impl<K> Default for IndexedHeap<K> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            positions: Vec::new(),
            #[cfg(all(test, feature = "micro-profiling"))]
            census: std::cell::Cell::new(Census::default()),
        }
    }
}

impl<K: Copy + Ord> IndexedHeap<K> {
    pub(super) fn peek(&self) -> Option<(usize, K)> {
        #[cfg(all(test, feature = "micro-profiling"))]
        self.count(|c| c.peeks += 1);
        self.entries.first().map(|entry| (entry.slot, entry.key))
    }

    pub(super) fn insert(&mut self, slot: usize, key: K) {
        #[cfg(all(test, feature = "micro-profiling"))]
        self.count(|c| c.insertions += 1);
        self.remove(slot);
        if self.positions.len() <= slot {
            self.positions.resize(slot + 1, None);
        }
        let index = self.entries.len();
        self.entries.push(Entry { slot, key });
        self.set_position(index);
        self.sift_up(index);
    }

    pub(super) fn remove(&mut self, slot: usize) -> Option<K> {
        #[cfg(all(test, feature = "micro-profiling"))]
        self.count(|c| c.remove_lookups += 1);
        let index = self.positions.get_mut(slot)?.take()?.get() - 1;
        // Every stored position was assigned from a live entry. The check
        // also keeps this private representation free of unchecked indexing.
        self.entries.get(index)?;
        #[cfg(all(test, feature = "micro-profiling"))]
        self.count(|c| c.removed_entries += 1);
        let removed = self.entries.swap_remove(index);
        if index < self.entries.len() {
            self.set_position(index);
            if index > 0 && self.less(index, (index - 1) / 2) {
                self.sift_up(index);
            } else {
                self.sift_down(index);
            }
        }
        Some(removed.key)
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
        self.positions.clear();
    }

    fn less(&self, left: usize, right: usize) -> bool {
        #[cfg(all(test, feature = "micro-profiling"))]
        self.count(|c| c.key_comparisons += 1);
        self.entries
            .get(left)
            .zip(self.entries.get(right))
            .is_some_and(|(left, right)| left.key < right.key)
    }

    fn set_position(&mut self, index: usize) {
        if let Some(entry) = self.entries.get(index)
            && let Some(position) = self.positions.get_mut(entry.slot)
        {
            *position = NonZeroUsize::new(index + 1);
        }
    }

    fn exchange(&mut self, left: usize, right: usize) {
        // Both positions were checked against this heap's current length by
        // sift_up/down; a swap cannot change its length or slot identities.
        #[cfg(all(test, feature = "micro-profiling"))]
        self.count(|c| c.swaps += 1);
        self.entries.swap(left, right);
        self.set_position(left);
        self.set_position(right);
    }

    fn sift_up(&mut self, mut index: usize) {
        while index > 0 {
            let parent = (index - 1) / 2;
            if !self.less(index, parent) {
                break;
            }
            self.exchange(index, parent);
            index = parent;
        }
    }

    fn sift_down(&mut self, mut index: usize) {
        loop {
            let left = index.saturating_mul(2).saturating_add(1);
            if left >= self.entries.len() {
                break;
            }
            let right = left + 1;
            let smallest = if right < self.entries.len() && self.less(right, left) {
                right
            } else {
                left
            };
            if !self.less(smallest, index) {
                break;
            }
            self.exchange(index, smallest);
            index = smallest;
        }
    }
}

#[cfg(test)]
impl<K: Copy + Ord> IndexedHeap<K> {
    pub(super) fn verify(&self) {
        for (index, entry) in self.entries.iter().enumerate() {
            assert_eq!(self.positions[entry.slot].unwrap().get(), index + 1);
            if index > 0 {
                assert!(self.entries[(index - 1) / 2].key <= entry.key);
            }
        }
        for (slot, position) in self.positions.iter().enumerate() {
            if let Some(position) = position {
                assert_eq!(self.entries[position.get() - 1].slot, slot);
            }
        }
    }

    pub(super) fn key(&self, slot: usize) -> Option<K> {
        let index = self.positions.get(slot)?.as_ref()?.get() - 1;
        self.entries.get(index).map(|entry| entry.key)
    }

    pub(super) fn storage(&self) -> (usize, usize, usize, usize) {
        (
            self.entries.len(),
            self.entries.capacity(),
            self.positions.len(),
            self.positions.capacity(),
        )
    }
}

#[cfg(all(test, feature = "micro-profiling"))]
#[derive(Clone, Copy, Default, serde::Serialize)]
pub(in crate::server::initial_diagnostics) struct Census {
    pub(super) peeks: u64,
    pub(super) insertions: u64,
    pub(super) remove_lookups: u64,
    pub(super) removed_entries: u64,
    pub(super) key_comparisons: u64,
    pub(super) swaps: u64,
}

#[cfg(all(test, feature = "micro-profiling"))]
impl<K> IndexedHeap<K> {
    fn count(&self, update: impl FnOnce(&mut Census)) {
        let mut census = self.census.get();
        update(&mut census);
        self.census.set(census);
    }

    pub(super) fn census(&self) -> Census {
        self.census.get()
    }
    pub(super) fn reset_census(&self) {
        self.census.set(Census::default());
    }
}

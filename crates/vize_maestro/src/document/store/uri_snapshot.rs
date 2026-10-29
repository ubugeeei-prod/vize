//! Immutable open-document URI snapshots.
#![expect(
    clippy::disallowed_types,
    reason = "request snapshots share immutable protocol URLs"
)]

use std::collections::BTreeSet;
use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard, RwLock};
use tower_lsp::lsp_types::Url;

const MUTATION_STRIPES: usize = 64;
const SMALL_DELTA_BATCH: usize = 8;
const MAX_PENDING_DELTAS: usize = 4_096;

pub(super) struct UriSnapshotCache {
    mutation_stripes: [Mutex<()>; MUTATION_STRIPES],
    overflow_stripe: Mutex<()>,
    state: RwLock<State>,
}

struct State {
    generation: u128,
    snapshot_generation: u128,
    snapshot: Arc<[Url]>,
    pending: Vec<Mutation>,
}

enum Mutation {
    Opened(Url),
    Closed(Url),
    Renamed { old: Url, new: Url },
}

impl Default for UriSnapshotCache {
    fn default() -> Self {
        Self {
            mutation_stripes: std::array::from_fn(|_| Mutex::new(())),
            overflow_stripe: Mutex::new(()),
            state: RwLock::new(State {
                generation: 0,
                snapshot_generation: 0,
                snapshot: Arc::<[Url]>::from([]),
                pending: Vec::new(),
            }),
        }
    }
}

impl UriSnapshotCache {
    pub(super) fn snapshot(&self) -> Arc<[Url]> {
        {
            let state = self.state.read();
            if state.snapshot_generation == state.generation {
                return Arc::clone(&state.snapshot);
            }
        }

        let mut state = self.state.write();
        if state.snapshot_generation != state.generation {
            rebuild(&mut state);
        }
        Arc::clone(&state.snapshot)
    }

    /// Serialize only mutations that touch the same URI stripe. Independent
    /// documents keep mutating concurrently, while conflicting map operations
    /// and their committed deltas cannot be observed in different orders.
    pub(super) fn mutation(&self, first: &Url, second: Option<&Url>) -> UriSetMutation<'_> {
        let first = self.stripe_index(first);
        let second = second.map(|uri| self.stripe_index(uri));
        let (low, high) = match second {
            Some(second) if second < first => (second, Some(first)),
            Some(second) if second > first => (first, Some(second)),
            _ => (first, None),
        };
        let low_guard = self.stripe(low).lock();
        let high_guard = high.map(|index| self.stripe(index).lock());
        UriSetMutation {
            cache: self,
            _low_guard: low_guard,
            _high_guard: high_guard,
        }
    }

    pub(super) fn stripe_index(&self, uri: &Url) -> usize {
        (vize_l0::hash::hash_str(uri.as_str()) as usize) & (MUTATION_STRIPES - 1)
    }

    fn stripe(&self, index: usize) -> &Mutex<()> {
        self.mutation_stripes
            .get(index)
            .unwrap_or(&self.overflow_stripe)
    }

    #[cfg(test)]
    pub(super) fn mutation_is_locked(&self, uri: &Url) -> bool {
        self.stripe(self.stripe_index(uri)).is_locked()
    }

    fn publish(&self, mutation: Mutation) {
        let mut state = self.state.write();
        state.pending.push(mutation);
        let generation = state.generation.wrapping_add(1);
        if state.snapshot_generation == generation {
            // A full u128 wrap must still force a rebuild.
            state.snapshot_generation = generation.wrapping_sub(1);
        }
        state.generation = generation;
        // Bound retained URI ownership when no workspace request consumes a
        // snapshot for a long-running open/close stream.
        if state.pending.len() >= MAX_PENDING_DELTAS {
            rebuild(&mut state);
        }
    }
}

pub(super) struct UriSetMutation<'a> {
    cache: &'a UriSnapshotCache,
    _low_guard: MutexGuard<'a, ()>,
    _high_guard: Option<MutexGuard<'a, ()>>,
}

impl UriSetMutation<'_> {
    pub(super) fn opened(self, uri: Url) {
        self.cache.publish(Mutation::Opened(uri));
    }

    pub(super) fn closed(self, uri: &Url) {
        self.cache.publish(Mutation::Closed(uri.clone()));
    }

    pub(super) fn renamed(self, old: &Url, new: Url) {
        self.cache.publish(Mutation::Renamed {
            old: old.clone(),
            new,
        });
    }
}

fn rebuild(state: &mut State) {
    let pending = std::mem::take(&mut state.pending);
    let mut uris = state.snapshot.to_vec();

    if pending.len() <= SMALL_DELTA_BATCH {
        for mutation in pending {
            apply_to_sorted_vec(&mut uris, mutation);
        }
    } else if pending
        .iter()
        .all(|mutation| matches!(mutation, Mutation::Opened(_)))
    {
        uris.reserve(pending.len());
        uris.extend(pending.into_iter().filter_map(|mutation| match mutation {
            Mutation::Opened(uri) => Some(uri),
            Mutation::Closed(_) | Mutation::Renamed { .. } => None,
        }));
        uris.sort_unstable();
        uris.dedup();
    } else {
        let mut members = uris.into_iter().collect::<BTreeSet<_>>();
        for mutation in pending {
            apply_to_set(&mut members, mutation);
        }
        uris = members.into_iter().collect();
    }

    state.snapshot = Arc::from(uris);
    state.snapshot_generation = state.generation;
}

fn apply_to_sorted_vec(uris: &mut Vec<Url>, mutation: Mutation) {
    match mutation {
        Mutation::Opened(uri) => insert_sorted(uris, uri),
        Mutation::Closed(uri) => remove_sorted(uris, &uri),
        Mutation::Renamed { old, new } => {
            remove_sorted(uris, &old);
            insert_sorted(uris, new);
        }
    }
}

fn insert_sorted(uris: &mut Vec<Url>, uri: Url) {
    if let Err(index) = uris.binary_search(&uri) {
        uris.insert(index, uri);
    }
}

fn remove_sorted(uris: &mut Vec<Url>, uri: &Url) {
    if let Ok(index) = uris.binary_search(uri) {
        uris.remove(index);
    }
}

fn apply_to_set(uris: &mut BTreeSet<Url>, mutation: Mutation) {
    match mutation {
        Mutation::Opened(uri) => {
            uris.insert(uri);
        }
        Mutation::Closed(uri) => {
            uris.remove(&uri);
        }
        Mutation::Renamed { old, new } => {
            uris.remove(&old);
            uris.insert(new);
        }
    }
}

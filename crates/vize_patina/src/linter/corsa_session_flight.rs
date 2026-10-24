//! One in-flight spawn per session key.
//!
//! The map lock covers claim and publish only. `spawn` runs after it is
//! released. A failed spawn is not inserted, so a later caller may retry.

#![expect(
    clippy::disallowed_types,
    reason = "waiters share one in-flight spawn handle for a session key"
)]

use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use super::corsa_session_pool::{CorsaSessionKey, lock_failure};
use vize_l0::{FxHashMap, String, ToCompactString};

pub(super) struct PoolInner<V> {
    pub(super) ready: FxHashMap<CorsaSessionKey, V>,
    inflight: FxHashMap<CorsaSessionKey, Arc<Flight<V>>>,
}

impl<V> Default for PoolInner<V> {
    fn default() -> Self {
        Self {
            ready: FxHashMap::default(),
            inflight: FxHashMap::default(),
        }
    }
}

enum Claim<V> {
    Ready(V),
    Wait(Arc<Flight<V>>),
    Lead(Arc<Flight<V>>),
}

struct Flight<V> {
    state: Mutex<Option<Result<V, String>>>,
    ready: Condvar,
}

impl<V> Flight<V> {
    fn pending() -> Self {
        Self {
            state: Mutex::new(None),
            ready: Condvar::new(),
        }
    }

    fn complete(&self, result: Result<V, String>) {
        let Ok(mut guard) = self.state.lock() else {
            return;
        };
        if guard.is_none() {
            *guard = Some(result);
            self.ready.notify_all();
        }
    }

    fn wait(&self) -> Result<V, String>
    where
        V: Clone,
    {
        let mut guard = self.state.lock().map_err(|_| lock_failure())?;
        loop {
            if let Some(result) = guard.as_ref() {
                return result.clone();
            }
            guard = self.ready.wait(guard).map_err(|_| lock_failure())?;
        }
    }
}

struct LeadGuard<'a, V> {
    inner: &'a Mutex<PoolInner<V>>,
    key: &'a CorsaSessionKey,
    flight: &'a Arc<Flight<V>>,
    armed: bool,
}

impl<V> LeadGuard<'_, V> {
    fn publish(&mut self, value: V) -> Result<V, String>
    where
        V: Clone,
    {
        let mut guard = match lock_map(self.inner) {
            Ok(guard) => guard,
            Err(error) => {
                self.fail(error.clone());
                return Err(error);
            }
        };
        let stored = guard.ready.entry(self.key.clone()).or_insert(value).clone();
        let current = guard
            .inflight
            .get(self.key)
            .is_some_and(|flight| Arc::ptr_eq(flight, self.flight));
        if current {
            guard.inflight.remove(self.key);
        }
        drop(guard);
        self.armed = false;
        self.flight.complete(Ok(stored.clone()));
        Ok(stored)
    }

    fn fail(&mut self, error: String) {
        if !self.armed {
            return;
        }
        self.armed = false;
        if let Ok(mut guard) = lock_map(self.inner) {
            let current = guard
                .inflight
                .get(self.key)
                .is_some_and(|flight| Arc::ptr_eq(flight, self.flight));
            if current {
                guard.inflight.remove(self.key);
            }
        }
        self.flight.complete(Err(error));
    }
}

impl<V> Drop for LeadGuard<'_, V> {
    fn drop(&mut self) {
        self.fail("corsa session spawn failed".to_compact_string());
    }
}

pub(super) fn get_or_spawn<V: Clone>(
    inner: &Mutex<PoolInner<V>>,
    key: &CorsaSessionKey,
    spawn: impl FnOnce() -> Result<V, String>,
) -> Result<V, String> {
    let claim = {
        let mut guard = lock_map(inner)?;
        claim(&mut guard, key)
    };
    match claim {
        Claim::Ready(value) => Ok(value),
        Claim::Wait(flight) => flight.wait(),
        Claim::Lead(flight) => {
            let mut lead = LeadGuard {
                inner,
                key,
                flight: &flight,
                armed: true,
            };
            match spawn() {
                Ok(value) => lead.publish(value),
                Err(error) => {
                    lead.fail(error.clone());
                    Err(error)
                }
            }
        }
    }
}

fn claim<V: Clone>(guard: &mut PoolInner<V>, key: &CorsaSessionKey) -> Claim<V> {
    if let Some(existing) = guard.ready.get(key).cloned() {
        return Claim::Ready(existing);
    }
    if let Some(existing) = guard.inflight.get(key).cloned() {
        return Claim::Wait(existing);
    }
    let flight = Arc::new(Flight::pending());
    guard.inflight.insert(key.clone(), Arc::clone(&flight));
    Claim::Lead(flight)
}

fn lock_map<V>(inner: &Mutex<PoolInner<V>>) -> Result<MutexGuard<'_, PoolInner<V>>, String> {
    vize_l0::profile!("patina.type_aware.corsa.lock", inner.lock()).map_err(|_| lock_failure())
}

#[cfg(test)]
mod tests;

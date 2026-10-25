use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};

use super::{Claim, PoolInner, claim, get_or_spawn};
use crate::linter::corsa_session_pool::CorsaSessionKey;
use vize_l0::{String, ToCompactString};

#[test]
fn concurrent_misses_spawn_once() {
    let inner = Mutex::new(PoolInner::<Arc<u8>>::default());
    let calls = AtomicUsize::new(0);
    let key = key("/app");
    let barrier = Barrier::new(8);
    let slots = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..8 {
            handles.push(scope.spawn(|| {
                barrier.wait();
                get_or_spawn(&inner, &key, || {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok(Arc::new(1))
                })
            }));
        }
        handles
            .into_iter()
            .map(|handle| handle.join().expect("spawn thread").expect("session"))
            .collect::<Vec<_>>()
    });
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let shared = slots.iter().all(|slot| Arc::ptr_eq(slot, &slots[0]));
    assert!(shared);
}

#[test]
fn failed_spawn_is_not_cached() {
    let inner = Mutex::new(PoolInner::<Arc<u8>>::default());
    let calls = AtomicUsize::new(0);
    let key = key("/app");
    let spawn_started = Barrier::new(2);
    let followers_claimed = Barrier::new(4);
    let failures = std::thread::scope(|scope| {
        let mut handles = vec![scope.spawn(|| {
            get_or_spawn(&inner, &key, || {
                calls.fetch_add(1, Ordering::SeqCst);
                spawn_started.wait();
                followers_claimed.wait();
                Err(failed_spawn())
            })
        })];
        spawn_started.wait();
        for _ in 0..3 {
            handles.push(scope.spawn(|| {
                let follower = {
                    let mut guard = inner.lock().expect("pool");
                    claim(&mut guard, &key)
                };
                // A caller released from a start barrier can still reach claim
                // after failure removes the flight. Retain every actual Wait
                // claim before allowing this flight's leader to fail.
                followers_claimed.wait();
                let Claim::Wait(flight) = follower else {
                    panic!("follower must join the pending flight");
                };
                flight.wait()
            }));
        }
        handles
            .into_iter()
            .map(|handle| handle.join().expect("spawn thread"))
            .collect::<Vec<_>>()
    });
    let all_failed = failures.iter().all(|result| result == &Err(failed_spawn()));
    assert!(all_failed);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let retried = get_or_spawn(&inner, &key, || {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(Arc::new(3))
    })
    .expect("a later caller retries after a failed spawn");
    assert_eq!(*retried, 3);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    let cached = get_or_spawn(&inner, &key, || {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(Arc::new(4))
    })
    .expect("the successful retry is cached");
    assert!(Arc::ptr_eq(&retried, &cached));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn different_keys_spawn_independently() {
    let inner = Mutex::new(PoolInner::<Arc<u8>>::default());
    let calls = AtomicUsize::new(0);
    let barrier = Barrier::new(2);
    let alpha = key("/alpha");
    let beta = key("/beta");
    std::thread::scope(|scope| {
        let left = scope.spawn(|| {
            get_or_spawn(&inner, &alpha, || {
                barrier.wait();
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(Arc::new(1))
            })
        });
        let right = scope.spawn(|| {
            get_or_spawn(&inner, &beta, || {
                barrier.wait();
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(Arc::new(2))
            })
        });
        left.join().expect("alpha thread").expect("alpha session");
        right.join().expect("beta thread").expect("beta session");
    });
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(inner.lock().expect("pool").ready.len(), 2);
}

#[test]
fn spawn_does_not_hold_the_map_lock() {
    let inner = Mutex::new(PoolInner::<Arc<u8>>::default());
    let unlocked = AtomicBool::new(false);
    let key = key("/app");
    get_or_spawn(&inner, &key, || {
        let acquired = inner.try_lock().is_ok();
        unlocked.store(acquired, Ordering::SeqCst);
        Ok(Arc::new(1))
    })
    .expect("spawn");
    let map_was_free = unlocked.load(Ordering::SeqCst);
    assert!(map_was_free);
}

fn key(root: &str) -> CorsaSessionKey {
    CorsaSessionKey {
        project_root: PathBuf::from(root),
        tsconfig: None,
    }
}

fn failed_spawn() -> String {
    "spawn failed".to_compact_string()
}

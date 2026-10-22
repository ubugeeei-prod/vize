use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::time::Duration;

use super::{PoolInner, get_or_spawn};
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
                    std::thread::sleep(Duration::from_millis(40));
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
    let barrier = Barrier::new(4);
    let failures = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..4 {
            handles.push(scope.spawn(|| {
                barrier.wait();
                get_or_spawn(&inner, &key, || {
                    calls.fetch_add(1, Ordering::SeqCst);
                    std::thread::sleep(Duration::from_millis(30));
                    Err(failed_spawn())
                })
            }));
        }
        handles
            .into_iter()
            .map(|handle| handle.join().expect("spawn thread").is_err())
            .collect::<Vec<_>>()
    });
    let all_failed = failures.iter().all(|failed| *failed);
    assert!(all_failed);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let retried = get_or_spawn(&inner, &key, || {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(Arc::new(3))
    })
    .expect("a later caller retries after a failed spawn");
    assert_eq!(*retried, 3);
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

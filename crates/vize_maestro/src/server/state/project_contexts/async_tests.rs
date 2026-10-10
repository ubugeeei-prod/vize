//! Shared initialization, bounded work, and canceled-owner lifetime controls.
use super::{Arc, pool, tests::fixture};
use futures::Future;
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    task::{Context, Poll},
    time::{Duration, Instant},
};

#[test]
fn concurrent_async_targets_reuse_one_config_generation_without_starting_corsa() {
    let (root, owner, a, _) = fixture();
    let package = root.path().join("packages/a");
    std::fs::write(package.join("vite.config.mjs"), "import {appendFileSync} from 'node:fs'; appendFileSync(new URL('./evaluations',import.meta.url),'1'); export default {vize:{lsp:{hover:false}}};").unwrap();
    let states = futures::executor::block_on(futures::future::join_all(
        (0..8).map(|_| owner.document_project_state_async(&a)),
    ));
    let first = states.first().unwrap().as_ref().unwrap();
    assert!(
        states
            .iter()
            .all(|state| Arc::ptr_eq(state.as_ref().unwrap(), first))
    );
    assert!(!first.lsp_features().hover);
    assert!(!first.has_corsa_bridge());
    assert_eq!(
        std::fs::read_to_string(package.join("evaluations")).unwrap(),
        "1"
    );
    assert_eq!(owner.cached_project_states().len(), 1);
    owner.retire_project_contexts();
    assert!(first.project_context_retired());
    assert!(futures::executor::block_on(owner.current_project_states_async()).is_empty());
}

#[test]
fn canceled_pending_initializer_has_no_registry_or_owner_cycle_after_shutdown() {
    let (root, owner, a, _) = fixture();
    let package = root.path().join("packages/a");
    std::fs::write(package.join("vite.config.mjs"), "import {existsSync,writeFileSync} from 'node:fs'; writeFileSync(new URL('./started',import.meta.url),'1'); while(!existsSync(new URL('./release',import.meta.url))) await new Promise(r=>setTimeout(r,10)); export default {vize:{lsp:{hover:false}}};").unwrap();
    {
        let request = owner.document_project_state_async(&a);
        futures::pin_mut!(request);
        let mut context = Context::from_waker(futures::task::noop_waker_ref());
        let deadline = Instant::now() + Duration::from_secs(5);
        while !package.join("started").exists() {
            if Instant::now() >= deadline {
                std::fs::write(package.join("release"), "1").unwrap();
                panic!("Node never entered pending initialization");
            }
            assert!(matches!(request.as_mut().poll(&mut context), Poll::Pending));
            std::thread::sleep(Duration::from_millis(10));
        }
        // Cancel only after the actual Node import is running on a worker.
    }
    let weak = Arc::downgrade(&owner);
    owner.retire_project_contexts();
    drop(owner);
    std::fs::write(package.join("release"), "1").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while weak.upgrade().is_some() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        weak.upgrade().is_none(),
        "canceled initialization retained an owner cycle"
    );
}

#[test]
fn queued_config_work_stays_bounded_and_all_jobs_finish() {
    let active = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let results = futures::executor::block_on(futures::future::join_all((0..48).map(|index| {
        let active = active.clone();
        let peak = peak.clone();
        pool::run(move || {
            let count = active.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(count, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(2));
            active.fetch_sub(1, Ordering::SeqCst);
            index
        })
    })));
    assert_eq!(results, (0..48).map(Some).collect::<Vec<_>>());
    assert!(peak.load(Ordering::SeqCst) <= 2);
    assert_eq!(active.load(Ordering::SeqCst), 0);
}

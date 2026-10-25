//! Shutdown admission and all-owner completion without real compiler claims.
use super::{Arc, ServerState, finish_all};
use futures::{FutureExt, channel::oneshot, executor::block_on};
use std::sync::atomic::Ordering;
use vize_canon::{CorsaBridge, CorsaBridgeError};

#[test]
fn no_owner_shutdown_is_safe_and_stops_root_admission() {
    let state = Arc::new(ServerState::new());
    assert!(block_on(state.shutdown_project_contexts()).is_ok());
    assert!(state.project_context_retired());
    assert!(state.project_routing_shutting_down());
    assert!(block_on(state.get_corsa_bridge()).is_none());
    assert!(block_on(state.shutdown_project_contexts()).is_ok());
}

#[test]
fn captures_root_and_every_cached_package_without_initializing_idle_contexts() {
    let (_directory, owner, a, b) = super::super::tests::fixture();
    let first = owner.document_project_state(&a).unwrap();
    let second = owner.document_project_state(&b).unwrap();
    let idle = Arc::new(super::super::registry::ProjectContext::new(
        owner.get_workspace_root().unwrap().join("idle"),
    ));
    owner.project_contexts.contexts.lock().insert(
        owner.get_workspace_root().unwrap().join("idle").into(),
        idle.clone(),
    );
    let root_bridge = Arc::new(CorsaBridge::new());
    let first_bridge = Arc::new(CorsaBridge::new());
    let second_bridge = Arc::new(CorsaBridge::new());
    *owner.corsa_bridge.write() = Some(root_bridge.clone());
    *first.corsa_bridge.write() = Some(first_bridge.clone());
    *second.corsa_bridge.write() = Some(second_bridge.clone());
    let states = owner.take_shutdown_states();
    assert_eq!(states.len(), 3);
    assert!(states.iter().any(|state| Arc::ptr_eq(state, &owner)));
    assert!(states.iter().any(|state| Arc::ptr_eq(state, &first)));
    assert!(states.iter().any(|state| Arc::ptr_eq(state, &second)));
    assert!(owner.cached_project_states().is_empty());
    assert!(idle.initialize(&owner).is_none());
    for (state, expected) in [
        (&owner, &root_bridge),
        (&first, &first_bridge),
        (&second, &second_bridge),
    ] {
        assert!(state.project_context_retired());
        assert!(state.project_routing_shutting_down());
        assert!(state.owns_corsa_bridge(expected));
        assert!(block_on(state.get_corsa_bridge()).is_none());
    }
}

#[test]
fn a_retained_unretired_package_cannot_admit_startup_after_global_shutdown() {
    let (_directory, owner, a, _) = super::super::tests::fixture();
    let package = owner.document_project_state(&a).unwrap();
    package.lsp_typecheck_enabled.store(true, Ordering::Release);
    owner
        .project_contexts
        .shutting_down
        .store(true, Ordering::Release);
    assert!(!package.project_context_retired());
    assert!(package.project_routing_shutting_down());
    assert!(block_on(package.get_corsa_bridge()).is_none());
    assert!(package.corsa_bridge.read().is_none());
}

#[test]
fn pending_admission_keeps_its_bridge_and_releases_registry_guards_before_waiting() {
    let state = Arc::new(ServerState::new());
    let bridge = Arc::new(CorsaBridge::new());
    let admission = block_on(state.corsa_init_lock.lock());
    *state.corsa_bridge.write() = Some(bridge.clone());
    let mut shutdown = Box::pin(state.shutdown_project_contexts());
    assert!(shutdown.as_mut().now_or_never().is_none());
    assert!(state.owns_corsa_bridge(&bridge));
    assert!(state.project_contexts.contexts.try_lock().is_some());
    assert!(
        state
            .project_contexts
            .pending_config_reloads
            .try_lock()
            .is_some()
    );
    assert!(
        state
            .project_contexts
            .configuration_attempts
            .try_lock()
            .is_some()
    );
    assert!(state.corsa_bridge.try_write().is_some());
    assert!(block_on(state.get_corsa_bridge()).is_none());
    // A nonblocking emergency retirement can clear the cache while startup
    // owns the gate. Shutdown retains the previously accepted Arc itself.
    *state.corsa_bridge.write() = None;
    assert_eq!(Arc::strong_count(&bridge), 2);
    drop(admission);
    assert!(block_on(shutdown).is_ok());
    assert!(state.corsa_bridge.read().is_none());
    assert!(state.corsa_init_lock.try_lock().is_some());
}

#[test]
fn every_owner_is_attempted_and_an_error_waits_for_the_other_owner() {
    let (release, held) = oneshot::channel();
    let mut all = Box::pin(finish_all([
        futures::future::ready(Err(CorsaBridgeError::NotInitialized)).boxed(),
        async move {
            held.await.unwrap();
            Ok(())
        }
        .boxed(),
    ]));
    assert!(all.as_mut().now_or_never().is_none());
    release.send(()).unwrap();
    assert!(matches!(
        block_on(all),
        Err(CorsaBridgeError::NotInitialized)
    ));
}

#[test]
fn a_panicking_owner_still_waits_for_every_other_owner() {
    let (release, held) = oneshot::channel();
    let mut all = Box::pin(finish_all([
        async { panic!("controlled owner panic") }.boxed(),
        async move {
            held.await.unwrap();
            Ok(())
        }
        .boxed(),
    ]));
    assert!(all.as_mut().now_or_never().is_none());
    release.send(()).unwrap();
    assert!(matches!(
        block_on(all),
        Err(CorsaBridgeError::CommunicationError(_))
    ));
}

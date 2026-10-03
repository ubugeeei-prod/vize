use super::{Arc, NavigationRefusal, Position, block_on, original_worker, project, uri, worker};
use crate::source_project::SnapshotRefusal;
use std::{future::Future, sync::atomic::Ordering, task::Context};

const SOURCE: &str = "<template><button/></template>";

fn both(project: &super::NativeNavigationProject<'_>) {
    block_on(project.definition(&uri(), Position::new(0, 1))).unwrap();
    block_on(project.template_definition(&uri(), Position::new(0, 1))).unwrap();
}

#[test]
fn close_retires_both_original_families_and_releases_actual_snapshot_after_owner_exit() {
    let (_, project) = project(SOURCE);
    both(&project);
    let selected = worker(&project);
    let original = original_worker(&project);
    let weak = Arc::downgrade(
        &project
            .selected_workers
            .lock()
            .get(&uri())
            .unwrap()
            .snapshot,
    );
    project.source.close(&uri());
    project.notify_host_change(&uri());
    selected.wait_exit();
    original.wait_exit();
    assert!(weak.upgrade().is_none());
    assert_eq!(project.live.load(Ordering::Acquire), 0);
    assert!(project.workers.lock().is_empty());
    assert!(project.selected_workers.lock().is_empty());
    assert_eq!(
        block_on(selected.template_definition(Position::new(0, 1))),
        Err(NavigationRefusal::WorkerUnavailable)
    );
}

#[test]
fn actual_in_flight_configuration_change_refuses_publication_and_retires_both_caches() {
    let (state, project) = project(SOURCE);
    both(&project);
    let selected = worker(&project);
    let original = original_worker(&project);
    let resume = selected.pause();
    let target = uri();
    let mut pending = Box::pin(project.template_definition(&target, Position::new(0, 1)));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    state.set_dialect_config(Some(vize_l0::config::VueDialect::PetiteVue));
    resume.send(()).unwrap();
    assert_eq!(
        block_on(pending),
        Err(NavigationRefusal::ConfigurationChanged)
    );
    selected.wait_exit();
    original.wait_exit();
    assert!(project.workers.lock().is_empty());
    assert!(project.selected_workers.lock().is_empty());
    assert_eq!(project.live.load(Ordering::Acquire), 0);
}

#[test]
fn an_actual_source_change_invalidates_selected_response_and_preserves_fresh_family_owner() {
    let (state, project) = project(SOURCE);
    both(&project);
    let selected = worker(&project);
    let original = original_worker(&project);
    let resume = selected.pause();
    let target = uri();
    let mut pending = Box::pin(project.template_definition(&target, Position::new(0, 1)));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    state.documents.open(
        uri(),
        "<template><span/></template>".into(),
        2,
        "vue".into(),
    );
    project.notify_host_change(&uri());
    assert_eq!(
        block_on(pending),
        Err(NavigationRefusal::Host(SnapshotRefusal::Cancelled))
    );
    assert_eq!(
        block_on(project.template_definition(&uri(), Position::new(0, 1))),
        Ok(vec![])
    );
    let fresh = worker(&project);
    project.notify_host_change(&uri());
    assert!(Arc::ptr_eq(&fresh, &worker(&project)));
    resume.send(()).unwrap();
    selected.wait_exit();
    original.wait_exit();
    assert_eq!(fresh.sfc_productions(), 1);
}

#[test]
fn a_dropped_selected_request_does_not_query_or_replace_the_retained_original_body() {
    let source = "<template><button @click='let value=$event;return value'/></template>";
    let (_, project) = project(source);
    let (position, _) = super::occurrence(source, "value", 1);
    block_on(project.template_definition(&uri(), position)).unwrap();
    let selected = worker(&project);
    let before = block_on(selected.selected_inspect(position)).unwrap();
    let resume = selected.pause();
    let mut pending = Box::pin(selected.template_definition(position));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    drop(pending);
    resume.send(()).unwrap();
    let after = block_on(selected.selected_inspect(position)).unwrap();
    assert_eq!(before, after);
    assert_eq!(selected.counts(), (0, 1));
}

#[test]
fn original_selected_worker_unwind_releases_owners_and_never_reparses_the_same_snapshot() {
    let (_, project) = project(SOURCE);
    block_on(project.template_definition(&uri(), Position::new(0, 1))).unwrap();
    let selected = worker(&project);
    selected.panic_worker();
    selected.wait_exit();
    assert_eq!(project.live.load(Ordering::Acquire), 0);
    assert_eq!(
        block_on(project.template_definition(&uri(), Position::new(0, 1))),
        Err(NavigationRefusal::WorkerUnavailable)
    );
    assert!(Arc::ptr_eq(&selected, &worker(&project)));
    assert_eq!(selected.sfc_productions(), 1);
}

#[test]
fn project_drop_retires_both_live_families_without_joining_under_cache_locks() {
    let (_, project) = project(SOURCE);
    both(&project);
    let selected = worker(&project);
    let original = original_worker(&project);
    let weak = Arc::downgrade(
        &project
            .selected_workers
            .lock()
            .get(&uri())
            .unwrap()
            .snapshot,
    );
    drop(project);
    selected.wait_exit();
    original.wait_exit();
    assert!(weak.upgrade().is_none());
    assert_eq!(selected.sfc_productions(), 1);
    assert_eq!(original.sfc_productions(), 1);
}

#[test]
fn actual_close_cancels_queued_selected_publication_without_waiting_for_receiver() {
    let (_, project) = project(SOURCE);
    both(&project);
    let selected = worker(&project);
    let original = original_worker(&project);
    let resume = selected.pause();
    let target = uri();
    let mut pending = Box::pin(project.template_definition(&target, Position::new(0, 1)));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    project.source.close(&target);
    project.notify_host_change(&target);
    assert_eq!(
        block_on(pending),
        Err(NavigationRefusal::Host(SnapshotRefusal::Cancelled))
    );
    assert!(project.workers.lock().is_empty());
    assert!(project.selected_workers.lock().is_empty());
    resume.send(()).unwrap();
    selected.wait_exit();
    original.wait_exit();
    assert_eq!(project.live.load(Ordering::Acquire), 0);
}

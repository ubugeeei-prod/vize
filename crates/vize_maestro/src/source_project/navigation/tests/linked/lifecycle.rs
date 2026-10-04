use super::{Arc, NavigationRefusal, Position, block_on, project, ranges, uri, worker};
use crate::source_project::SnapshotRefusal;
use std::{future::Future, sync::atomic::Ordering, task::Context};
const SOURCE: &str = "<template><p></p></template>";
fn both(project: &super::NativeNavigationProject<'_>) {
    block_on(project.definition(&uri(), Position::new(0, 1))).unwrap();
    block_on(project.template_definition(&uri(), Position::new(0, 1))).unwrap();
    block_on(project.linked_editing(&uri(), Position::new(0, 11))).unwrap();
}
fn owners(
    project: &super::NativeNavigationProject<'_>,
) -> Vec<Arc<super::super::super::worker::NavigationWorker>> {
    [
        &project.workers,
        &project.selected_workers,
        &project.names_workers,
    ]
    .into_iter()
    .map(|cache| Arc::clone(cache.lock().get(&uri()).unwrap().result.as_ref().unwrap()))
    .collect()
}
#[test]
fn close_retires_all_three_families_and_drops_actual_snapshot_after_owner_exit() {
    let (_, project) = project(SOURCE);
    both(&project);
    let owners = owners(&project);
    let weak = Arc::downgrade(&project.names_workers.lock().get(&uri()).unwrap().snapshot);
    project.source.close(&uri());
    project.notify_host_change(&uri());
    for owner in owners {
        owner.wait_exit();
    }
    assert!(weak.upgrade().is_none());
    assert_eq!(project.live.load(Ordering::Acquire), 0);
    assert!(project.workers.lock().is_empty());
    assert!(project.selected_workers.lock().is_empty());
    assert!(project.names_workers.lock().is_empty());
}
#[test]
fn configuration_change_during_original_names_query_refuses_and_retires_all_families() {
    let (state, project) = project(SOURCE);
    both(&project);
    let owners = owners(&project);
    let names = worker(&project);
    let resume = names.pause();
    let target = uri();
    let mut pending = Box::pin(project.linked_editing(&target, Position::new(0, 11)));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    state.set_dialect_config(Some(vize_l0::config::VueDialect::PetiteVue));
    resume.send(()).unwrap();
    assert_eq!(
        block_on(pending),
        Err(NavigationRefusal::ConfigurationChanged)
    );
    for owner in owners {
        owner.wait_exit();
    }
    assert!(project.names_workers.lock().is_empty());
    assert_eq!(project.live.load(Ordering::Acquire), 0);
}
#[test]
fn actual_change_cancels_old_names_publication_and_preserves_fresh_original_owner() {
    let (state, project) = project(SOURCE);
    both(&project);
    let owners = owners(&project);
    let names = worker(&project);
    let resume = names.pause();
    let target = uri();
    let mut pending = Box::pin(project.linked_editing(&target, Position::new(0, 11)));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    state.documents.open(
        uri(),
        "<template><span></span></template>".into(),
        2,
        "vue".into(),
    );
    project.notify_host_change(&uri());
    assert_eq!(
        block_on(pending),
        Err(NavigationRefusal::Host(SnapshotRefusal::Cancelled))
    );
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 12))),
        Ok(ranges((0, 11, 15), (0, 18, 22)))
    );
    let fresh = worker(&project);
    project.notify_host_change(&uri());
    assert!(Arc::ptr_eq(&fresh, &worker(&project)));
    resume.send(()).unwrap();
    for owner in owners {
        owner.wait_exit();
    }
    assert_eq!(fresh.sfc_productions(), 1);
}
#[test]
fn dropped_request_does_not_traverse_or_replace_the_retained_names_owner() {
    let (_, project) = project(SOURCE);
    block_on(project.linked_editing(&uri(), Position::new(0, 11))).unwrap();
    let names = worker(&project);
    let before = block_on(names.names_inspect()).unwrap();
    let resume = names.pause();
    let mut pending = Box::pin(names.linked_editing(Position::new(0, 11)));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    drop(pending);
    resume.send(()).unwrap();
    assert_eq!(before, block_on(names.names_inspect()).unwrap());
    assert_eq!(names.counts(), (0, 1));
}
#[test]
fn original_names_worker_unwind_releases_owner_and_never_reparses_same_snapshot() {
    let (_, project) = project(SOURCE);
    block_on(project.linked_editing(&uri(), Position::new(0, 11))).unwrap();
    let names = worker(&project);
    names.panic_worker();
    names.wait_exit();
    assert_eq!(project.live.load(Ordering::Acquire), 0);
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 11))),
        Err(NavigationRefusal::WorkerUnavailable)
    );
    assert!(Arc::ptr_eq(&names, &worker(&project)));
    assert_eq!(names.sfc_productions(), 1);
}
#[test]
fn project_drop_retires_all_original_families_without_joining_under_locks() {
    let (_, project) = project(SOURCE);
    both(&project);
    let owners = owners(&project);
    let weak = Arc::downgrade(&project.names_workers.lock().get(&uri()).unwrap().snapshot);
    drop(project);
    for owner in owners {
        owner.wait_exit();
        assert_eq!(owner.sfc_productions(), 1);
    }
    assert!(weak.upgrade().is_none());
}
#[test]
fn actual_close_cancels_names_publication_before_receiver_resumes() {
    let (_, project) = project(SOURCE);
    both(&project);
    let owners = owners(&project);
    let names = worker(&project);
    let resume = names.pause();
    let target = uri();
    let mut pending = Box::pin(project.linked_editing(&target, Position::new(0, 11)));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    project.source.close(&target);
    project.notify_host_change(&target);
    assert_eq!(
        block_on(pending),
        Err(NavigationRefusal::Host(SnapshotRefusal::Cancelled))
    );
    assert!(project.names_workers.lock().is_empty());
    resume.send(()).unwrap();
    for owner in owners {
        owner.wait_exit();
    }
    assert_eq!(project.live.load(Ordering::Acquire), 0);
}

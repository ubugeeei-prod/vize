use super::{NavigationRefusal, Position, block_on, project, ranges, uri, worker};
use crate::source_project::SnapshotRefusal;
use std::{future::Future, sync::atomic::Ordering, task::Context};
const SOURCE: &str = "<template><p></p></template>";

#[test]
fn original_frame_change_cancels_old_publication_and_requeries_the_new_owner() {
    let (state, project) = project(SOURCE);
    block_on(project.linked_editing(&uri(), Position::new(0, 1))).unwrap();
    let original = worker(&project);
    let resume = original.pause();
    let target = uri();
    let mut pending = Box::pin(project.linked_editing(&target, Position::new(0, 1)));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    state
        .documents
        .open(uri(), "<template>new</template>".into(), 2, "vue".into());
    project.notify_host_change(&target);
    assert_eq!(
        block_on(pending),
        Err(NavigationRefusal::Host(SnapshotRefusal::Cancelled))
    );
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 1))),
        Ok(ranges((0, 1, 9), (0, 15, 23)))
    );
    let fresh = worker(&project);
    assert!(!std::sync::Arc::ptr_eq(&original, &fresh));
    assert_eq!(fresh.sfc_productions(), 1);
    resume.send(()).unwrap();
    original.wait_exit();
}

#[test]
fn original_frame_configuration_change_refuses_and_retires_the_live_owner() {
    let (state, project) = project(SOURCE);
    block_on(project.linked_editing(&uri(), Position::new(0, 1))).unwrap();
    let original = worker(&project);
    let resume = original.pause();
    let target = uri();
    let mut pending = Box::pin(project.linked_editing(&target, Position::new(0, 1)));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    state.set_dialect_config(Some(vize_l0::config::VueDialect::PetiteVue));
    resume.send(()).unwrap();
    assert_eq!(
        block_on(pending),
        Err(NavigationRefusal::ConfigurationChanged)
    );
    original.wait_exit();
    assert!(project.names_workers.lock().is_empty());
    assert_eq!(project.live.load(Ordering::Acquire), 0);
}

#[test]
fn original_frame_close_retires_the_snapshot_before_the_pending_receiver_resumes() {
    let (_, project) = project(SOURCE);
    block_on(project.linked_editing(&uri(), Position::new(0, 1))).unwrap();
    let original = worker(&project);
    let weak =
        std::sync::Arc::downgrade(&project.names_workers.lock().get(&uri()).unwrap().snapshot);
    let resume = original.pause();
    let target = uri();
    let mut pending = Box::pin(project.linked_editing(&target, Position::new(0, 1)));
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
    original.wait_exit();
    assert!(weak.upgrade().is_none());
    assert_eq!(project.live.load(Ordering::Acquire), 0);
}

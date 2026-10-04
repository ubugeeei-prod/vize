use super::{Arc, NavigationRefusal, Position, Url, block_on, project, uri, worker};
use std::{future::Future, sync::atomic::Ordering, task::Context};
const SOURCE: &str = "<template><p></p></template>";
#[test]
fn names_selected_and_program_families_share_one_actual_live_limit_with_retryable_slots() {
    let (state, project) = project(SOURCE);
    let mut owners = Vec::new();
    let mut uris = Vec::new();
    for index in 0..5 {
        let target = Url::parse(&vize_l0::cstr!("file:///names-capacity-{index}.vue")).unwrap();
        state
            .documents
            .open(target.clone(), SOURCE.into(), 1, "vue".into());
        block_on(project.definition(&target, Position::new(0, 1))).unwrap();
        block_on(project.template_definition(&target, Position::new(0, 1))).unwrap();
        block_on(project.linked_editing(&target, Position::new(0, 11))).unwrap();
        for cache in [
            &project.workers,
            &project.selected_workers,
            &project.names_workers,
        ] {
            owners.push(Arc::clone(
                cache.lock().get(&target).unwrap().result.as_ref().unwrap(),
            ));
        }
        uris.push(target);
    }
    block_on(project.definition(&uri(), Position::new(0, 1))).unwrap();
    assert_eq!(project.live.load(Ordering::Acquire), 16);
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 11))),
        Err(NavigationRefusal::Capacity)
    );
    assert_eq!(
        block_on(project.template_definition(&uri(), Position::new(0, 1))),
        Err(NavigationRefusal::Capacity)
    );
    assert!(!project.names_workers.lock().contains_key(&uri()));
    project.source.close(&uris[0]);
    project.notify_host_change(&uris[0]);
    for owner in &owners[..3] {
        owner.wait_exit();
    }
    assert_eq!(project.live.load(Ordering::Acquire), 13);
    assert!(
        block_on(project.linked_editing(&uri(), Position::new(0, 11)))
            .unwrap()
            .is_some()
    );
    block_on(project.template_definition(&uri(), Position::new(0, 1))).unwrap();
    assert_eq!(project.live.load(Ordering::Acquire), 15);
}
#[test]
fn original_names_mailbox_refuses_busy_and_skips_all_cancelled_requests() {
    let (_, project) = project(SOURCE);
    block_on(project.linked_editing(&uri(), Position::new(0, 11))).unwrap();
    let names = worker(&project);
    let resume = names.pause();
    let mut pending = Vec::new();
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    for _ in 0..16 {
        let mut query = Box::pin(names.linked_editing(Position::new(0, 11)));
        assert!(query.as_mut().poll(&mut context).is_pending());
        pending.push(query);
    }
    assert_eq!(
        block_on(names.linked_editing(Position::new(0, 11))),
        Err(NavigationRefusal::Busy)
    );
    drop(pending);
    resume.send(()).unwrap();
    let original = block_on(names.names_drain_barrier()).unwrap();
    assert_eq!(original.productions, 1);
    assert_eq!(names.counts(), (0, 1));
}

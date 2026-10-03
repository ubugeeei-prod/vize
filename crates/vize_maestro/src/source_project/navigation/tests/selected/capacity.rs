use super::{Arc, NavigationRefusal, Position, Url, block_on, project, uri, worker};
use std::{future::Future, sync::atomic::Ordering, task::Context};

const SOURCE: &str = "<template><button/></template>";

#[test]
fn selected_and_original_families_share_one_real_live_owner_limit_and_retryable_slots() {
    let (state, project) = project(SOURCE);
    let mut owners = Vec::new();
    let mut uris = Vec::new();
    for index in 0..8 {
        let uri = Url::parse(&vize_l0::cstr!("file:///selected-capacity-{index}.vue")).unwrap();
        state
            .documents
            .open(uri.clone(), SOURCE.into(), 1, "vue".into());
        block_on(project.definition(&uri, Position::new(0, 1))).unwrap();
        block_on(project.template_definition(&uri, Position::new(0, 1))).unwrap();
        for cache in [&project.workers, &project.selected_workers] {
            owners.push(Arc::clone(
                cache.lock().get(&uri).unwrap().result.as_ref().unwrap(),
            ));
        }
        uris.push(uri);
    }
    assert_eq!(project.live.load(Ordering::Acquire), 16);
    let extra = uri();
    assert_eq!(
        block_on(project.template_definition(&extra, Position::new(0, 1))),
        Err(NavigationRefusal::Capacity)
    );
    assert_eq!(
        block_on(project.definition(&extra, Position::new(0, 1))),
        Err(NavigationRefusal::Capacity)
    );
    assert!(!project.workers.lock().contains_key(&extra));
    assert!(!project.selected_workers.lock().contains_key(&extra));
    project.source.close(&uris[0]);
    project.notify_host_change(&uris[0]);
    owners[0].wait_exit();
    owners[1].wait_exit();
    assert_eq!(project.live.load(Ordering::Acquire), 14);
    assert_eq!(
        block_on(project.definition(&extra, Position::new(0, 1))),
        Ok(None)
    );
    assert_eq!(
        block_on(project.template_definition(&extra, Position::new(0, 1))),
        Ok(vec![])
    );
    assert_eq!(project.live.load(Ordering::Acquire), 16);
}

#[test]
fn a_full_selected_mailbox_refuses_busy_and_cancelled_commands_do_not_query() {
    let (_, project) = project(SOURCE);
    block_on(project.template_definition(&uri(), Position::new(0, 1))).unwrap();
    let selected = worker(&project);
    let resume = selected.pause();
    let mut pending = Vec::new();
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    for _ in 0..16 {
        let mut query = Box::pin(selected.template_definition(Position::new(0, 1)));
        assert!(query.as_mut().poll(&mut context).is_pending());
        pending.push(query);
    }
    assert_eq!(
        block_on(selected.template_definition(Position::new(0, 1))),
        Err(NavigationRefusal::Busy)
    );
    drop(pending);
    resume.send(()).unwrap();
    block_on(selected.selected_drain_barrier(Position::new(0, 1))).unwrap();
    assert_eq!(selected.counts(), (0, 1));
    assert_eq!(
        block_on(project.template_definition(&uri(), Position::new(0, 1))),
        Ok(vec![])
    );
    assert_eq!(selected.sfc_productions(), 1);
}

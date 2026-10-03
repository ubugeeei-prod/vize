use super::{
    Arc, Kind, NavigationRefusal, Position, block_on, both, expected, occurrence, project, uri,
    worker,
};
use crate::source_project::SnapshotRefusal;
use std::{future::Future, sync::atomic::Ordering, task::Context};

const SOURCE: &str = "<template><button/></template>";

#[test]
fn highlight_close_cancels_pending_publication_and_drops_both_original_owners() {
    let (_, project) = project(SOURCE, "vue");
    both(&project);
    let selected = worker(&project, true);
    let original = worker(&project, false);
    let weak = Arc::downgrade(
        &project
            .selected_workers
            .lock()
            .get(&uri())
            .unwrap()
            .snapshot,
    );
    let resume = selected.pause();
    let target = uri();
    let mut pending = Box::pin(project.template_highlights(&target, Position::new(0, 1)));
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
    assert!(weak.upgrade().is_none());
    assert_eq!(project.live.load(Ordering::Acquire), 0);
}

#[test]
fn either_highlight_family_refuses_changed_real_configuration_and_retires_both_maps() {
    for selected_family in [false, true] {
        let (state, project) = project(SOURCE, "vue");
        both(&project);
        let selected = worker(&project, true);
        let original = worker(&project, false);
        let paused = if selected_family {
            &selected
        } else {
            &original
        };
        let resume = paused.pause();
        let target = uri();
        let mut pending = Box::pin(async {
            if selected_family {
                project
                    .template_highlights(&target, Position::new(0, 1))
                    .await
            } else {
                project.highlights(&target, Position::new(0, 1)).await
            }
        });
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
}

#[test]
fn genuine_source_change_refuses_old_highlight_and_preserves_new_owner_from_late_hook() {
    let source = "<template><button @click='let value=$event;return value'/></template>";
    let (state, project) = project(source, "vue");
    let target = uri();
    let at = occurrence(source, "value", 0).0;
    block_on(project.template_highlights(&target, at)).unwrap();
    let old = worker(&project, true);
    let resume = old.pause();
    let mut pending = Box::pin(project.template_highlights(&target, at));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    let changed = "<template><button @click='let fresh=$event;return fresh'/></template>";
    state
        .documents
        .open(target.clone(), changed.into(), 2, "vue".into());
    project.notify_host_change(&target);
    assert_eq!(
        block_on(pending),
        Err(NavigationRefusal::Host(SnapshotRefusal::Cancelled))
    );
    let wanted = expected(changed, "fresh", &[(0, Kind::TEXT), (1, Kind::READ)]);
    assert_eq!(
        block_on(project.template_highlights(&target, at)),
        Ok(wanted)
    );
    let fresh = worker(&project, true);
    assert!(!Arc::ptr_eq(&old, &fresh));
    project.notify_host_change(&target);
    assert!(Arc::ptr_eq(&fresh, &worker(&project, true)));
    resume.send(()).unwrap();
    old.wait_exit();
    assert_eq!(fresh.sfc_productions(), 1);
}

#[test]
fn dropped_highlight_receivers_skip_queued_work_in_both_real_worker_families() {
    for (source, language, selected_family, name) in [
        ("let value=1;value++;", "javascript", false, "value"),
        (
            "<template><button @click='let value=$event;return value'/></template>",
            "vue",
            true,
            "value",
        ),
    ] {
        let (_, project) = project(source, language);
        let position = occurrence(source, name, 0).0;
        block_on(async {
            if selected_family {
                project.template_highlights(&uri(), position).await
            } else {
                project.highlights(&uri(), position).await
            }
        })
        .unwrap();
        let original = worker(&project, selected_family);
        let before = original.counts();
        let resume = original.pause();
        let mut pending = Box::pin(original.highlights(position));
        let mut context = Context::from_waker(futures::task::noop_waker_ref());
        assert!(pending.as_mut().poll(&mut context).is_pending());
        drop(pending);
        resume.send(()).unwrap();
        if selected_family {
            block_on(original.selected_inspect(position)).unwrap();
        } else {
            block_on(original.inspect(position)).unwrap();
        }
        assert_eq!(before, original.counts());
    }
}

#[test]
fn highlight_commands_share_actual_mailbox_busy_limit_with_existing_selected_queries() {
    let (_, project) = project(SOURCE, "vue");
    block_on(project.template_highlights(&uri(), Position::new(0, 1))).unwrap();
    let original = worker(&project, true);
    let resume = original.pause();
    let mut pending = Vec::new();
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    for _ in 0..16 {
        let mut request = Box::pin(original.highlights(Position::new(0, 1)));
        assert!(request.as_mut().poll(&mut context).is_pending());
        pending.push(request);
    }
    assert_eq!(
        block_on(original.highlights(Position::new(0, 1))),
        Err(NavigationRefusal::Busy)
    );
    assert_eq!(
        block_on(original.template_definition(Position::new(0, 1))),
        Err(NavigationRefusal::Busy)
    );
    drop(pending);
    resume.send(()).unwrap();
    block_on(original.selected_drain_barrier(Position::new(0, 1))).unwrap();
    assert_eq!(original.counts(), (0, 1));
    assert_eq!(
        block_on(project.template_highlights(&uri(), Position::new(0, 1))),
        Ok(vec![])
    );
    assert_eq!(original.sfc_productions(), 1);
}

#[test]
fn original_worker_unwind_and_project_drop_keep_highlight_owner_exit_sound() {
    let (_, project) = project(SOURCE, "vue");
    both(&project);
    let selected = worker(&project, true);
    let original = worker(&project, false);
    selected.panic_worker();
    selected.wait_exit();
    assert_eq!(
        block_on(project.template_highlights(&uri(), Position::new(0, 1))),
        Err(NavigationRefusal::WorkerUnavailable)
    );
    assert_eq!(selected.sfc_productions(), 1);
    let weak = Arc::downgrade(&project.workers.lock().get(&uri()).unwrap().snapshot);
    drop(project);
    original.wait_exit();
    assert!(weak.upgrade().is_none());
}

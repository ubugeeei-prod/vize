use super::{
    Arc, NavigationRefusal, Position, Url, block_on, new_project, occurrence, uri, worker,
};
use crate::source_project::SnapshotRefusal;
use std::{future::Future, sync::atomic::Ordering, task::Context};

#[test]
fn actual_project_vue_version_and_template_role_refuse_and_refresh_the_original_owner() {
    let source = "<script setup>const value=1;</script><template>{{value}}</template>";
    let (state, project) = new_project(source);
    let (reference, declaration) = occurrence(source, "value", 0);
    block_on(project.definition(&uri(), reference)).unwrap();
    let original = worker(&project);
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("vize.config.json");
    std::fs::write(&config, r#"{"vue":{"version":"2"}}"#).unwrap();
    state.load_lsp_config(directory.path());
    assert!(matches!(
        block_on(project.definition(&uri(), reference)),
        Err(NavigationRefusal::SfcProducer(_))
    ));
    original.wait_exit();
    let refused = worker(&project);
    assert_eq!(refused.sfc_productions(), 1);
    std::fs::write(&config, r#"{"vue":{"version":"3"}}"#).unwrap();
    state.load_lsp_config(directory.path());
    assert_eq!(
        block_on(project.definition(&uri(), reference)),
        Ok(Some(declaration))
    );
    refused.wait_exit();
    let current = worker(&project);
    assert!(!Arc::ptr_eq(&original, &current));
    assert_eq!(current.sfc_productions(), 1);
    state.documents.open(
        Url::parse("file:///standalone.html").unwrap(),
        source.into(),
        1,
        "vue".into(),
    );
    assert_eq!(
        block_on(project.definition(&Url::parse("file:///standalone.html").unwrap(), reference)),
        Err(NavigationRefusal::Language)
    );
}

#[test]
fn configuration_change_and_source_edit_refuse_ready_vue_publication() {
    let source = "<script setup>const value=1;</script><template>{{value}}</template>";
    let (state, project) = new_project(source);
    let (position, _) = occurrence(source, "value", 1);
    let (query, _) = project.source.begin_query(&uri()).unwrap();
    let owner = &project;
    let ready = block_on(query.run(|snapshot| async move {
        let worker = owner.worker(snapshot)?;
        Ok((worker.profile(), worker.definition(position).await))
    }))
    .unwrap();
    state.set_dialect_config(Some(vize_l0::config::VueDialect::PetiteVue));
    assert_eq!(
        ready.publish(|result| project.checked_response(result)),
        Ok(Err(NavigationRefusal::ConfigurationChanged))
    );
    project.retire_changed_configuration(&uri());
    assert!(project.workers.lock().is_empty());
    let (query, _) = project.source.begin_query(&uri()).unwrap();
    let owner = &project;
    let ready = block_on(query.run(|snapshot| async move {
        let worker = owner.worker(snapshot)?;
        worker.definition(position).await
    }))
    .unwrap();
    state
        .documents
        .open(uri(), source.replace("value", "fresh"), 2, "vue".into());
    assert_eq!(
        ready.publish(|result| result),
        Err(SnapshotRefusal::Superseded)
    );
    project.notify_host_change(&uri());
}

#[test]
fn actual_patterned_template_configuration_is_a_sticky_explicit_refusal() {
    let source = "<script setup>const value=1;</script><template>{{value}}</template>";
    let (state, project) = new_project(source);
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("vize.config.json"),
        r#"{"experimentals":{"patternedTemplate":true}}"#,
    )
    .unwrap();
    state.load_lsp_config(directory.path());
    for _ in 0..2 {
        assert_eq!(
            block_on(project.definition(&uri(), Position::new(0, 21))),
            Err(NavigationRefusal::Configuration)
        );
    }
    assert_eq!(worker(&project).sfc_productions(), 0);
}

#[test]
fn cancelled_and_closed_vue_requests_release_original_owners_and_live_capacity() {
    let source = "<script setup>const value=1;</script><template>{{value}}</template>";
    let (state, project) = new_project(source);
    let (position, declaration) = occurrence(source, "value", 0);
    block_on(project.definition(&uri(), position)).unwrap();
    let worker = worker(&project);
    let (query, cancel) = project.source.begin_query(&uri()).unwrap();
    let resume = worker.pause();
    let mut pending = Box::pin(query.run(|_| async { worker.definition(position).await }));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    cancel.abort();
    assert!(matches!(
        pending.as_mut().poll(&mut context),
        std::task::Poll::Ready(Err(SnapshotRefusal::Cancelled))
    ));
    drop(pending);
    resume.send(()).unwrap();
    assert_eq!(
        block_on(project.definition(&uri(), position)),
        Ok(Some(declaration))
    );
    assert_eq!(worker.sfc_productions(), 1);
    assert_eq!(worker.counts().1, 2);
    let weak = Arc::downgrade(&project.workers.lock().get(&uri()).unwrap().snapshot);
    state.documents.close(&uri());
    project.notify_host_change(&uri());
    worker.wait_exit();
    assert!(weak.upgrade().is_none());
    assert_eq!(project.live.load(Ordering::Acquire), 0);
}

#[test]
fn vue_workers_share_the_bounded_mailbox_and_real_live_owner_limit() {
    let source = "<script setup>const value=1;</script><template>{{value}}</template>";
    let (state, project) = new_project(source);
    let (position, _) = occurrence(source, "value", 1);
    block_on(project.definition(&uri(), position)).unwrap();
    let original = worker(&project);
    let resume = original.pause();
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    let mut pending = Vec::new();
    for _ in 0..16 {
        let mut request = Box::pin(original.definition(position));
        assert!(request.as_mut().poll(&mut context).is_pending());
        pending.push(request);
    }
    assert_eq!(
        block_on(original.definition(position)),
        Err(NavigationRefusal::Busy)
    );
    drop(pending);
    resume.send(()).unwrap();
    block_on(original.drain_barrier(position)).unwrap();
    for index in 1..16 {
        let next = Url::parse(&vize_l0::cstr!("file:///vue-capacity-{index}.vue")).unwrap();
        state
            .documents
            .open(next.clone(), source.into(), 1, "vue".into());
        assert!(
            block_on(project.definition(&next, position))
                .unwrap()
                .is_some()
        );
    }
    let extra = Url::parse("file:///vue-capacity-extra.vue").unwrap();
    state
        .documents
        .open(extra.clone(), source.into(), 1, "vue".into());
    assert_eq!(
        block_on(project.definition(&extra, position)),
        Err(NavigationRefusal::Capacity)
    );
    assert!(!project.workers.lock().contains_key(&extra));
    let resume = original.pause();
    state.documents.close(&uri());
    project.notify_host_change(&uri());
    assert_eq!(project.live.load(Ordering::Acquire), 16);
    assert_eq!(
        block_on(project.definition(&extra, position)),
        Err(NavigationRefusal::Capacity)
    );
    resume.send(()).unwrap();
    original.wait_exit();
    assert!(
        block_on(project.definition(&extra, position))
            .unwrap()
            .is_some()
    );
    assert_eq!(project.live.load(Ordering::Acquire), 16);
}

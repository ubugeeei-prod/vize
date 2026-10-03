use super::{
    Arc, NativeNavigationProject, NavigationRefusal, Position, SourceQueryProject, block_on,
};
use crate::{server::ServerState, source_project::SnapshotRefusal};
use std::{future::Future, sync::atomic::Ordering, task::Context};
use tower_lsp::lsp_types::{Location, Range, Url};

fn uri() -> Url {
    Url::parse("file:///original.vue").unwrap()
}

fn new_project(source: &str) -> (Arc<ServerState>, NativeNavigationProject<'static>) {
    let state = Arc::new(ServerState::new());
    state.documents.open(uri(), source.into(), 1, "vue".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new_server(Arc::clone(&state)));
    (state, project)
}

fn worker(project: &NativeNavigationProject<'_>) -> Arc<super::super::worker::NavigationWorker> {
    Arc::clone(
        project
            .workers
            .lock()
            .get(&uri())
            .unwrap()
            .result
            .as_ref()
            .unwrap(),
    )
}

// The expected locations select authored spellings, independently of native
// resolution/decoder tables. In particular &fjlig; must retain all seven bytes.
fn position(source: &str, byte: usize) -> Position {
    let prefix = &source[..byte];
    Position::new(
        prefix.bytes().filter(|byte| *byte == b'\n').count() as u32,
        prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    )
}
fn occurrence(source: &str, needle: &str, index: usize) -> (Position, Location) {
    let byte = source.match_indices(needle).nth(index).unwrap().0;
    let start = position(source, byte);
    (
        start,
        Location::new(
            uri(),
            Range::new(start, position(source, byte + needle.len())),
        ),
    )
}

#[test]
fn genuine_sfc_file_descriptor_programs_and_embeds_survive_repeated_requests() {
    let source = "\r\n<template><div :title=\"café\">{{café}}</div></template>\r\n<script setup lang=ts>/*😀*/ const café=1; café;</script>";
    let (_, project) = new_project(source);
    let (attribute, attribute_location) = occurrence(source, "café", 0);
    let (template, template_location) = occurrence(source, "café", 1);
    let (declaration, declaration_location) = occurrence(source, "café", 2);
    let (_, script_location) = occurrence(source, "café", 3);
    assert_eq!(
        block_on(project.definition(&uri(), attribute)),
        Ok(Some(declaration_location.clone()))
    );
    let owner = worker(&project);
    let before = block_on(owner.inspect(attribute)).unwrap();
    let sfc = before.sfc.as_ref().unwrap();
    assert_eq!(sfc.productions, 1);
    assert_eq!(sfc.programs.len(), 1);
    assert_eq!(sfc.programs[0].1, 2);
    assert_eq!(sfc.expressions.len(), 2);
    assert_eq!(before.parses, 0); // Not a fabricated one-Program parser count.
    assert_eq!(
        block_on(project.references(&uri(), declaration, true)),
        Ok(vec![
            attribute_location,
            template_location.clone(),
            declaration_location.clone(),
            script_location
        ])
    );
    assert_eq!(
        block_on(project.definition(&uri(), template)),
        Ok(Some(declaration_location))
    );
    let after = block_on(owner.inspect(template)).unwrap();
    assert_eq!(before, after);
    assert!(Arc::ptr_eq(&owner, &worker(&project)));
}

#[test]
fn exact_entity_and_escaped_identifier_projection_keeps_authored_ranges() {
    let source = "<script setup>const fj=1; const café=2;</script>\n<template><div :title=\"&fjlig;\">{{caf\\u00e9}}</div></template>";
    let (_, project) = new_project(source);
    let (entity, entity_location) = occurrence(source, "&fjlig;", 0);
    let (_, fj_declaration) = occurrence(source, "fj", 0);
    let (escaped, escaped_location) = occurrence(source, "caf\\u00e9", 0);
    let (_, cafe_declaration) = occurrence(source, "café", 0);
    assert_eq!(
        block_on(project.definition(&uri(), entity)),
        Ok(Some(fj_declaration.clone()))
    );
    // Selecting an authored byte inside the entity still selects the genuine
    // identifier occurrence; returning its location retains the complete entity.
    assert_eq!(
        block_on(project.references(
            &uri(),
            Position::new(entity.line, entity.character + 2),
            true
        )),
        Ok(vec![fj_declaration, entity_location])
    );
    assert_eq!(
        block_on(project.definition(&uri(), escaped)),
        Ok(Some(cafe_declaration))
    );
    assert_eq!(
        block_on(project.references(&uri(), escaped, false)),
        Ok(vec![escaped_location])
    );
}

#[test]
fn reversed_script_order_setup_shadow_and_local_import_aliases_use_real_binding_ids() {
    let source = "<script setup>import { run as local } from 'dep'; const value=2;</script>\n<script>const value=1;</script>\n<template>{{local(value)}}</template>";
    let (_, project) = new_project(source);
    let (setup, setup_location) = occurrence(source, "value", 0);
    let (ordinary, ordinary_location) = occurrence(source, "value", 1);
    let (template, template_location) = occurrence(source, "value", 2);
    let (_, alias_location) = occurrence(source, "local", 0);
    let (call, call_location) = occurrence(source, "local", 1);
    assert_eq!(
        block_on(project.definition(&uri(), template)),
        Ok(Some(setup_location.clone()))
    );
    assert_eq!(
        block_on(project.references(&uri(), setup, true)),
        Ok(vec![setup_location, template_location])
    );
    assert_eq!(
        block_on(project.references(&uri(), ordinary, true)),
        Ok(vec![ordinary_location])
    );
    assert_eq!(
        block_on(project.definition(&uri(), call)),
        Ok(Some(alias_location.clone()))
    );
    assert_eq!(
        block_on(project.references(&uri(), call, true)),
        Ok(vec![alias_location, call_location])
    );
    let retained = block_on(worker(&project).inspect(call)).unwrap();
    assert_eq!(retained.sfc.unwrap().programs.len(), 2);
}

#[test]
fn half_open_names_static_source_holes_and_opaque_styles_have_no_fabricated_references() {
    let source = "<script setup>const value=1;</script>\n<template>{{value}}</template>\n<style scoped>.value{color:red}</style>";
    let (_, project) = new_project(source);
    let (template, _) = occurrence(source, "value", 1);
    let (style, _) = occurrence(source, "value", 2);
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(template.line, template.character + 5))),
        Ok(None)
    );
    assert_eq!(block_on(project.definition(&uri(), style)), Ok(None));
    assert_eq!(
        block_on(project.references(&uri(), style, true)),
        Ok(Vec::new())
    );
    let (_, static_project) = new_project("<template><div>日本語</div></template>");
    assert_eq!(
        block_on(static_project.definition(&uri(), Position::new(0, 17))),
        Ok(None)
    );
}

#[test]
fn original_sfc_refusals_are_sticky_without_script_only_or_legacy_fallback() {
    for source in [
        "<script>const value=1;</script><template>{{value}}</template>",
        "<script setup lang=ts>const value:number=1;</script><template>{{value}}</template>",
        "<script setup>const value=/x/uv;</script><template>{{value}}</template>",
        "<script setup>const value=1;</script><template><div v-if=\"value\"/></template>",
        "<script setup src='./external.ts'></script><template></template>",
    ] {
        let (_, project) = new_project(source);
        let result = block_on(project.definition(&uri(), Position::new(0, 1)));
        assert!(
            matches!(&result, Err(NavigationRefusal::SfcProducer(issues)) if !issues.is_empty()),
            "{source}: {result:?}"
        );
        let original = worker(&project);
        assert_eq!(
            block_on(project.definition(&uri(), Position::new(0, 1))),
            result
        );
        assert!(Arc::ptr_eq(&original, &worker(&project)));
        assert_eq!(original.sfc_productions(), 1);
    }
}

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

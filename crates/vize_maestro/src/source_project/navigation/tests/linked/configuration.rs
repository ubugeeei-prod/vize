//! Real settings transactions revoke old requests, never original syntax owners.
use super::{
    Arc, NativeNavigationProject, NavigationRefusal, Position, block_on, project, ranges, uri,
    worker,
};
use crate::server::{NativeNamesConfigurationError, ServerState};
use std::{future::Future, task::Context};
use vize_l0::config::VueDialect;
mod loaders;
mod publication;
const SOURCE: &str = "<template><p></p></template>";

fn enable(state: &ServerState) {
    state.apply_lsp_initialization_options(Some(
        &serde_json::json!({"nativeLinkedEditing":true,"rename":true}),
    ));
}
fn linked_ticket(state: &ServerState) -> crate::server::NativeLinkedNamesTicket {
    let crate::server::NativeLinkedNamesRoute::Native(ticket) = state.capture_native_linked_route()
    else {
        panic!("actual native linked route disabled");
    };
    ticket
}
fn assert_original(project: &NativeNavigationProject<'_>) {
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 11))),
        Ok(ranges((0, 11, 12), (0, 15, 16)))
    );
}
fn all_owners(
    project: &NativeNavigationProject<'_>,
) -> Vec<Arc<super::super::super::worker::NavigationWorker>> {
    block_on(project.definition(&uri(), Position::new(0, 1))).unwrap();
    block_on(project.template_definition(&uri(), Position::new(0, 1))).unwrap();
    assert_original(project);
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
fn equal_foreign_server_settings_cannot_admit_or_publish_a_names_ticket() {
    let (local, _) = project(SOURCE);
    let (foreign, project) = project(SOURCE);
    enable(&local);
    enable(&foreign);
    assert_eq!(
        local.native_names_settings(),
        foreign.native_names_settings()
    );
    let parser = local.capture_native_names_parser();
    assert_eq!(
        foreign.with_current_native_names_parser(&parser, |_| ()),
        Err(NativeNamesConfigurationError::ParserChanged)
    );
    let linked = linked_ticket(&local);
    assert_eq!(
        foreign.with_current_native_linked_names(&linked, |_| ()),
        Err(NativeNamesConfigurationError::ParserChanged)
    );
    let (query, _) = project.source.begin_query(&uri()).unwrap();
    let ticket = super::super::super::names::NamesConfiguration::Parser(parser);
    assert!(matches!(
        project.names_worker(Arc::clone(query.snapshot()), &ticket),
        Err(NavigationRefusal::ConfigurationChanged)
    ));
    assert!(project.names_workers.lock().is_empty());
    assert_original(&project);
    assert_eq!(worker(&project).sfc_productions(), 1);
}

#[test]
fn parser_only_provider_keeps_pending_response_and_all_owners_across_route_aba() {
    let (state, project) = project(SOURCE);
    let original = all_owners(&project);
    let names = worker(&project);
    let resume = names.pause();
    let target = uri();
    let mut pending = Box::pin(project.linked_editing(&target, Position::new(0, 11)));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    // The public API was admitted with the default transport opt-in false.
    for options in [
        serde_json::json!({"nativeLinkedEditing":true,"rename":false}),
        serde_json::json!({"nativeLinkedEditing":false,"rename":true}),
    ] {
        state.apply_lsp_initialization_options(Some(&options));
    }
    resume.send(()).unwrap();
    assert_eq!(block_on(pending), Ok(ranges((0, 11, 12), (0, 15, 16))));
    for (before, current) in original.iter().zip(all_owners(&project)) {
        assert!(Arc::ptr_eq(before, &current));
    }
    assert_eq!(names.sfc_productions(), 1);
}

#[test]
fn native_opt_in_aba_refuses_old_publication_without_retiring_fresh_owners() {
    route_aba("nativeLinkedEditing");
}
#[test]
fn rename_aba_refuses_old_publication_without_retiring_fresh_owners() {
    route_aba("rename");
}
fn route_aba(key: &str) {
    let (state, project) = project(SOURCE);
    enable(&state);
    let original = all_owners(&project);
    let names = worker(&project);
    let resume = names.pause();
    let target = uri();
    let ticket = linked_ticket(&state);
    let mut old =
        Box::pin(project.linked_editing_configured(&target, Position::new(0, 11), ticket));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(old.as_mut().poll(&mut context).is_pending());
    assert!(old.as_mut().poll(&mut context).is_pending());
    state.apply_lsp_initialization_options(Some(&serde_json::json!({(key):false})));
    state.apply_lsp_initialization_options(Some(&serde_json::json!({(key):true})));
    let ticket = linked_ticket(&state);
    let mut fresh =
        Box::pin(project.linked_editing_configured(&target, Position::new(0, 11), ticket));
    assert!(fresh.as_mut().poll(&mut context).is_pending());
    assert!(fresh.as_mut().poll(&mut context).is_pending());
    resume.send(()).unwrap();
    assert_eq!(block_on(old), Err(NavigationRefusal::NamesRouteChanged));
    assert_eq!(block_on(fresh), Ok(ranges((0, 11, 12), (0, 15, 16))));
    for (before, current) in original.iter().zip(all_owners(&project)) {
        assert!(Arc::ptr_eq(before, &current));
    }
    assert_eq!(names.sfc_productions(), 1);
}

#[test]
fn actual_dialect_aba_refuses_old_parser_ticket_and_keeps_compatible_original_owner() {
    parser_aba(|state| {
        state.set_dialect_config(Some(VueDialect::PetiteVue));
        state.set_dialect_config(None);
    });
}
#[test]
fn actual_effective_legacy_aba_refuses_old_parser_ticket_and_keeps_original_owner() {
    parser_aba(|state| {
        state.apply_lsp_initialization_options(Some(&serde_json::json!({"legacyVue2":true})));
        state.apply_lsp_initialization_options(Some(&serde_json::json!({"legacyVue2":false})));
    });
}
fn parser_aba(change: impl FnOnce(&ServerState)) {
    let (state, project) = project(SOURCE);
    let original = all_owners(&project);
    let names = worker(&project);
    let resume = names.pause();
    let target = uri();
    let mut pending = Box::pin(project.linked_editing(&target, Position::new(0, 11)));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    change(&state);
    resume.send(()).unwrap();
    assert_eq!(
        block_on(pending),
        Err(NavigationRefusal::ConfigurationChanged)
    );
    for (before, current) in original.iter().zip(all_owners(&project)) {
        assert!(Arc::ptr_eq(before, &current));
    }
    assert_eq!(names.sfc_productions(), 1);
}

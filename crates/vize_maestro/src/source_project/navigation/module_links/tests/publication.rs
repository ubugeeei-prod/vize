//! Real retained operands keep their original query through target publication.
use super::{Arc, block_on, link, project};
use crate::{
    server::{ModuleLinkContextError, ModuleLinkRetirement, ModuleTargetError},
    source_project::{
        ModuleLinkPublicationError, ProjectQueryResult, SnapshotRefusal,
        navigation::{
            NativeNavigationProject,
            retained::module_links::{ModuleOperandRefusal, ModuleOperands},
        },
    },
};
use tower_lsp::lsp_types::Url;

fn ready(
    project: &NativeNavigationProject<'static>,
    uri: &Url,
) -> ProjectQueryResult<'static, Result<ModuleOperands, ModuleOperandRefusal>> {
    let (query, _) = project.source.begin_query(uri).unwrap();
    block_on(query.run(|snapshot| async move {
        project
            .worker(snapshot)
            .map_err(ModuleOperandRefusal::Navigation)?
            .module_operands()
            .await
    }))
    .unwrap()
}

#[test]
fn module_link_consumer_original_query_cancel_cannot_borrow_success_from_equal_cached_query() {
    let (_dir, root, uri, _state, project) = project("import './child.ts';");
    let (a, cancel) = project.source.begin_query(&uri).unwrap();
    let (b, _) = project.source.begin_query(&uri).unwrap();
    assert!(Arc::ptr_eq(a.snapshot(), b.snapshot()));
    let a = block_on(
        a.run(|snapshot| async { project.worker(snapshot).unwrap().module_operands().await }),
    )
    .unwrap();
    let b = block_on(
        b.run(|snapshot| async { project.worker(snapshot).unwrap().module_operands().await }),
    )
    .unwrap();
    assert_eq!(
        a.prepared()
            .as_ref()
            .unwrap()
            .decoded_requests()
            .collect::<Vec<_>>(),
        ["./child.ts"]
    );
    let context = a.capture_current_module_link_context().unwrap();
    let checked = a
        .observe_module_targets(&context, &["./child.ts"])
        .unwrap()
        .recheck()
        .unwrap();
    cancel.abort();
    let mut published = false;
    assert!(matches!(
        a.publish_with_module_targets(checked, |_, _| published = true),
        Err(ModuleTargetError::Source(SnapshotRefusal::Cancelled))
    ));
    assert!(!published);
    let context = b.capture_current_module_link_context().unwrap();
    let checked = b
        .observe_module_targets(&context, &["./child.ts"])
        .unwrap()
        .recheck()
        .unwrap();
    assert_eq!(
        b.publish_with_module_targets(checked, |response, targets| {
            response.unwrap().into_links(targets).unwrap()
        })
        .unwrap(),
        vec![link(
            0,
            7,
            19,
            Url::from_file_path(root.join("child.ts")).unwrap()
        )]
    );
}

#[test]
fn module_link_consumer_held_zero_operands_and_read_refusal_both_require_original_context() {
    for source in ["const local=1;local;", ""] {
        let (_dir, root, uri, state, project) = project(source);
        let ready = ready(&project, &uri);
        if source.is_empty() {
            assert!(
                matches!(ready.prepared(),Err(ModuleOperandRefusal::Sources(error))
                if error.kind == vize_l2::lang::js::ModuleSourceErrorKind::EmptyProgram)
            );
        } else {
            assert!(ready.prepared().as_ref().unwrap().is_empty());
        }
        let context = ready.capture_current_module_link_context().unwrap();
        state.set_workspace_root(root.clone());
        let mut published = false;
        assert!(matches!(
            ready.publish_with_module_link_context(&context, |_| published = true),
            Err(ModuleLinkPublicationError::Context(
                ModuleLinkContextError::Superseded
            ))
        ));
        assert!(!published);
    }
}

#[test]
fn module_link_consumer_full_original_operand_batch_refuses_late_physical_replacement() {
    let (_dir, root, uri, _state, project) =
        project("import './child.ts';\nimport './second.vue';");
    let ready = ready(&project, &uri);
    let context = ready.capture_current_module_link_context().unwrap();
    let requests = ready
        .prepared()
        .as_ref()
        .unwrap()
        .decoded_requests()
        .collect::<Vec<_>>();
    assert_eq!(requests, ["./child.ts", "./second.vue"]);
    let observed = ready.observe_module_targets(&context, &requests).unwrap();
    std::fs::rename(root.join("second.vue"), root.join("held.vue")).unwrap();
    std::fs::write(root.join("second.vue"), "<template/>").unwrap();
    assert!(
        matches!(observed.recheck(),Err(ModuleTargetError::Changed(path))
        if path == root.join("second.vue"))
    );
    assert_eq!(
        ready
            .prepared()
            .as_ref()
            .unwrap()
            .decoded_requests()
            .collect::<Vec<_>>(),
        requests
    );
    // Fresh point evidence remains explicit and does not reuse the refused batch.
    let checked = ready
        .observe_module_targets(&context, &requests)
        .unwrap()
        .recheck()
        .unwrap();
    assert_eq!(
        ready
            .publish_with_module_targets(checked, |response, targets| {
                response.unwrap().into_links(targets).unwrap()
            })
            .unwrap(),
        vec![
            link(
                0,
                7,
                19,
                Url::from_file_path(root.join("child.ts")).unwrap()
            ),
            link(
                1,
                7,
                21,
                Url::from_file_path(root.join("second.vue")).unwrap()
            ),
        ]
    );
}

#[test]
fn module_link_consumer_received_event_epoch_discards_held_complete_link_response() {
    let (_dir, _root, uri, state, project) = project("import './child.ts';");
    let ready = ready(&project, &uri);
    let context = ready.capture_current_module_link_context().unwrap();
    let checked = ready
        .observe_module_targets(&context, &["./child.ts"])
        .unwrap()
        .recheck()
        .unwrap();
    state.observe_module_target_file_events(true);
    let mut published = false;
    assert!(matches!(
        ready.publish_with_module_targets(checked, |_, _| published = true),
        Err(ModuleTargetError::EventSuperseded)
    ));
    assert!(!published);
    assert_eq!(
        block_on(project.module_document_links(&uri)).unwrap().len(),
        1
    );
}

#[test]
fn module_link_consumer_source_then_context_then_event_priority_guards_entire_owned_response() {
    for stale_source in [false, true] {
        let (_dir, root, uri, state, project) = project("import './child.ts';");
        let ready = ready(&project, &uri);
        let context = ready.capture_current_module_link_context().unwrap();
        let checked = ready
            .observe_module_targets(&context, &["./child.ts"])
            .unwrap()
            .recheck()
            .unwrap();
        if stale_source {
            // Bypass the advisory notification deliberately: final source identity still refuses.
            state.documents.close(&uri);
        }
        state.set_workspace_root(root);
        state.observe_module_target_file_events(true);
        let mut published = false;
        let result = ready.publish_with_module_targets(checked, |_, _| published = true);
        if stale_source {
            assert!(matches!(
                result,
                Err(ModuleTargetError::Source(SnapshotRefusal::MissingDocument))
            ));
        } else {
            assert!(matches!(
                result,
                Err(ModuleTargetError::Context(
                    ModuleLinkContextError::Superseded
                ))
            ));
        }
        assert!(!published);
    }
}

#[test]
fn module_link_consumer_input_retirement_with_retained_server_state_refuses_held_real_operands() {
    for reason in [
        ModuleLinkRetirement::InputEof,
        ModuleLinkRetirement::InputError,
        ModuleLinkRetirement::TransportEnded,
        ModuleLinkRetirement::Shutdown,
    ] {
        let (_dir, _root, uri, state, project) = project("import './child.ts';");
        let held_state = Arc::clone(&state);
        let ready = ready(&project, &uri);
        let context = ready.capture_current_module_link_context().unwrap();
        let checked = ready
            .observe_module_targets(&context, &["./child.ts"])
            .unwrap()
            .recheck()
            .unwrap();
        state.retire_module_links(reason);
        let mut published = false;
        assert!(
            matches!(ready.publish_with_module_targets(checked,|_,_|published=true),
            Err(ModuleTargetError::Context(ModuleLinkContextError::Retired(actual))) if actual==reason)
        );
        assert!(!published);
        assert!(
            matches!(held_state.capture_module_link_context(),Err(ModuleLinkContextError::Retired(actual))
            if actual==reason)
        );
        assert!(block_on(project.module_document_links(&uri)).is_err());
    }
}

#[test]
fn module_link_consumer_actual_checker_load_origin_and_config_aba_guard_original_operands() {
    let (_dir, root, uri, state, project) = project("import './child.ts';");
    let path = root.join("vize.config.json");
    let first = r#"{"typeChecker":{"strict":true,"lspRequestTimeoutMs":91000}}"#;
    std::fs::write(&path, first).unwrap();
    state.load_workspace_config(&root);
    let ready = ready(&project, &uri);
    let context = ready.capture_current_module_link_context().unwrap();
    assert_eq!(context.root(), root);
    assert_eq!(context.load_origin(), Some(path.as_path()));
    assert_eq!(context.timeout_ms(), 91000);
    assert_eq!(
        context.checker(),
        &vize_l0::config::TypeCheckerConfig {
            strict: true,
            ..Default::default()
        }
    );
    assert_eq!(
        ready
            .prepared()
            .as_ref()
            .unwrap()
            .decoded_requests()
            .collect::<Vec<_>>(),
        ["./child.ts"]
    );
    let checked = ready
        .observe_module_targets(&context, &["./child.ts"])
        .unwrap()
        .recheck()
        .unwrap();
    std::fs::write(
        &path,
        r#"{"typeChecker":{"strict":false,"lspRequestTimeoutMs":72000}}"#,
    )
    .unwrap();
    state.load_workspace_config(&root);
    std::fs::write(&path, first).unwrap();
    state.load_workspace_config(&root);
    let mut published = false;
    assert!(matches!(
        ready.publish_with_module_targets(checked, |_, _| published = true),
        Err(ModuleTargetError::Context(
            ModuleLinkContextError::Superseded
        ))
    ));
    assert!(!published);
    let current = state.capture_module_link_context().unwrap();
    assert_eq!(current.root(), context.root());
    assert_eq!(current.checker(), context.checker());
    assert_eq!(current.timeout_ms(), context.timeout_ms());
    assert_eq!(current.load_origin(), context.load_origin());
    assert_eq!(
        block_on(project.module_document_links(&uri)).unwrap(),
        vec![link(
            0,
            7,
            19,
            Url::from_file_path(root.join("child.ts")).unwrap()
        )]
    );
}

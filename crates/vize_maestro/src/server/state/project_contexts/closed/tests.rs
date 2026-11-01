//! Closed-world routing ownership, default reuse and cold reopen controls.
use super::super::{registry::ProjectContext, tests::fixture};
use super::*;
use tower_lsp::lsp_types::Url;

fn open(state: &ServerState, uri: &Url, source: &str) {
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
}

#[test]
fn absent_false_and_invalid_options_keep_the_original_warm_context() {
    for option in [
        serde_json::Value::Null,
        serde_json::json!({"releaseClosedProjectContexts":false}),
        serde_json::json!({"releaseClosedProjectContexts":"invalid"}),
    ] {
        let (_files, root, a, _) = fixture();
        root.apply_lsp_initialization_options((!option.is_null()).then_some(&option));
        let child = root.document_project_state(&a).unwrap();
        assert_eq!(root.release_closed_child_contexts(), 0);
        assert!(Arc::ptr_eq(
            &child,
            &root.document_project_state(&a).unwrap()
        ));
        assert!(!child.project_context_retired());
    }
}

#[test]
fn every_open_document_protects_all_children_and_primary_remains_owned() {
    let (files, root, a, b) = fixture();
    root.apply_lsp_initialization_options(Some(
        &serde_json::json!({"releaseClosedProjectContexts":true}),
    ));
    let child_a = root.document_project_state(&a).unwrap();
    let child_b = root.document_project_state(&b).unwrap();
    let primary = Arc::new(ProjectContext::new(files.path().to_path_buf()));
    root.project_contexts
        .contexts
        .lock()
        .insert(files.path().to_path_buf().into(), Arc::clone(&primary));
    let primary_bridge = Arc::new(vize_canon::CorsaBridge::new());
    *root.corsa_bridge.write() = Some(Arc::clone(&primary_bridge));
    open(
        &root,
        &a,
        "<script setup>import Child from '../../b/src/App.vue'</script>",
    );
    open(
        &root,
        &b,
        "<script setup>const unsaved: string = 1</script>",
    );
    assert_eq!(root.release_closed_child_contexts(), 0);
    root.documents.close(&a);
    assert_eq!(
        root.release_closed_child_contexts(),
        0,
        "an unsaved cross-config dependency remains open"
    );
    assert!(!child_a.project_context_retired());
    assert!(!child_b.project_context_retired());
    root.documents.close(&b);
    assert_eq!(root.release_closed_child_contexts(), 2);
    assert!(child_a.project_context_retired());
    assert!(child_b.project_context_retired());
    assert!(root.cached_project_states().is_empty());
    assert_eq!(root.project_contexts.contexts.lock().len(), 1);
    assert!(root.owns_corsa_bridge(&primary_bridge));
    assert!(!root.project_context_retired());
    assert_eq!(root.release_closed_child_contexts(), 0);
}

#[test]
fn detached_state_keeps_its_bridge_and_reopen_version_one_cannot_revive_old_stamps() {
    let (_files, root, a, _) = fixture();
    root.apply_lsp_initialization_options(Some(
        &serde_json::json!({"releaseClosedProjectContexts":true}),
    ));
    open(&root, &a, "<template>before</template>");
    let old = root.document_project_state(&a).unwrap();
    let old_stamp = old.corsa_request_stamp();
    let bridge = Arc::new(vize_canon::CorsaBridge::new());
    let weak_bridge = Arc::downgrade(&bridge);
    *old.corsa_bridge.write() = Some(bridge);
    root.documents.close(&a);
    assert_eq!(root.release_closed_child_contexts(), 1);
    assert!(old.project_context_retired());
    assert!(!old_stamp.is_current(&old));
    assert!(
        weak_bridge.upgrade().is_some(),
        "held old state owns its exact connection"
    );
    assert_eq!(root.project_contexts.detached_bridges.lock().len(), 1);
    let discovered = root.take_detached_corsa_bridges();
    assert_eq!(discovered.len(), 1);
    assert!(Arc::ptr_eq(&discovered[0], &weak_bridge.upgrade().unwrap()));
    assert!(root.project_contexts.detached_bridges.lock().is_empty());
    drop(discovered);
    open(&root, &a, "<template>after</template>");
    let fresh = root.document_project_state(&a).unwrap();
    assert!(!Arc::ptr_eq(&old, &fresh));
    assert!(!fresh.project_context_retired());
    assert!(!old_stamp.is_current(&fresh));
    assert_eq!(fresh.documents.version(&a), Some(1));
    assert_eq!(
        fresh.documents.text(&a).as_deref(),
        Some("<template>after</template>")
    );
    assert_eq!(fresh.lsp_features().hover, old.lsp_features().hover);
    drop(old);
    assert!(weak_bridge.upgrade().is_none());
}

#[test]
fn busy_registry_and_pending_reload_preserve_entries_without_waiting() {
    let (_files, root, a, _) = fixture();
    root.apply_lsp_initialization_options(Some(
        &serde_json::json!({"releaseClosedProjectContexts":true}),
    ));
    let child = root.document_project_state(&a).unwrap();
    let before = root.project_routing_generation();
    let contexts = root.project_contexts.contexts.lock();
    assert_eq!(root.release_closed_child_contexts(), 0);
    drop(contexts);
    assert_eq!(root.project_routing_generation(), before);
    let pending = root.project_contexts.pending_config_reloads.lock();
    assert!(pending.is_empty());
    assert_eq!(
        root.release_closed_child_contexts(),
        0,
        "busy empty reload authority is unknown"
    );
    assert!(!child.project_context_retired());
    drop(pending);
    assert_eq!(root.project_routing_generation(), before);
    let key = child.project_contexts.key.read().clone().unwrap();
    root.project_contexts
        .pending_config_reloads
        .lock()
        .insert(key.clone());
    assert_eq!(root.release_closed_child_contexts(), 0);
    assert!(!child.project_context_retired());
    root.project_contexts
        .pending_config_reloads
        .lock()
        .remove(&key);
    assert_eq!(root.release_closed_child_contexts(), 1);
}

#[cfg(feature = "retained-program")]
#[test]
fn the_retained_navigation_source_retirement_aborts_and_drops_its_waiter() {
    use futures::FutureExt;
    let (_files, root, a, _) = fixture();
    open(&root, &a, "<template />");
    let child = root.document_project_state(&a).unwrap();
    let project = crate::source_project::SourceQueryProject::new_server(child);
    let (query, _) = project.begin_query(&a).unwrap();
    let (dropped, observed) = std::sync::mpsc::channel();
    let (entered, observed_entry) = std::sync::mpsc::channel();
    struct OnDrop(std::sync::mpsc::Sender<()>);
    impl Drop for OnDrop {
        fn drop(&mut self) {
            let _ = self.0.send(());
        }
    }
    let mut work = Box::pin(query.run(move |_| async move {
        let _owner = OnDrop(dropped);
        entered.send(()).unwrap();
        std::future::pending::<()>().await;
    }));
    // SourceQuery::run deliberately yields to cancellation before polling compute.
    assert!(work.as_mut().now_or_never().is_none());
    assert!(observed_entry.try_recv().is_err());
    assert!(work.as_mut().now_or_never().is_none());
    observed_entry
        .try_recv()
        .expect("the held computation actually entered");
    // This is the same source retirement called by NativeNavigationProject::retire.
    project.retire();
    assert!(crate::runtime::block_on(work).is_err());
    observed
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
}

#[test]
fn cancelled_configuration_initialization_drains_into_a_detached_old_owner() {
    use futures::Future;
    use std::{
        task::Context,
        time::{Duration, Instant},
    };
    for stop in [false, true] {
        let (files, root, a, _) = fixture();
        let package = files.path().join("packages/a");
        let release = package.join("release");
        struct Release(std::path::PathBuf);
        impl Drop for Release {
            fn drop(&mut self) {
                let _ = std::fs::write(&self.0, "1");
            }
        }
        let _release_on_failure = Release(release.clone());
        std::fs::write(package.join("vite.config.mjs"), "import {existsSync,writeFileSync} from 'node:fs'; writeFileSync(new URL('./started',import.meta.url),'1'); while(!existsSync(new URL('./release',import.meta.url))) await new Promise(r=>setTimeout(r,10)); export default {vue:{languageServer:{hover:false}}};").unwrap();
        root.apply_lsp_initialization_options(Some(
            &serde_json::json!({"releaseClosedProjectContexts":true}),
        ));
        let old = root.document_project_context(&a).unwrap();
        let mut request = Box::pin(old.initialize_async(&root));
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        while !package.join("started").exists() {
            assert!(
                Instant::now() < deadline,
                "actual configuration worker must enter"
            );
            assert!(request.as_mut().poll(&mut cx).is_pending());
            std::thread::yield_now();
        }
        assert_eq!(root.release_closed_child_contexts(), 1);
        assert!(root.project_contexts.detached_bridges.lock().is_empty());
        drop(request);
        if stop {
            crate::runtime::block_on(root.shutdown_project_contexts()).unwrap();
        }
        std::fs::write(&release, "1").unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !old
            .published_state()
            .is_some_and(|state| state.project_context_retired())
        {
            assert!(
                Instant::now() < deadline,
                "entered initializer must finish and mark its old owner"
            );
            std::thread::yield_now();
        }
        assert!(old.initialized_state().is_none());
        let previous = Arc::clone(old.published_state().unwrap());
        if stop {
            assert!(previous.project_routing_shutting_down());
            assert!(crate::runtime::block_on(previous.get_corsa_bridge()).is_none());
            assert!(
                previous.corsa_init_failure().is_none(),
                "aborted config publication starts no native owner"
            );
            assert!(root.project_contexts.detached_bridges.lock().is_empty());
            continue;
        }
        let fresh = root.document_project_state(&a).unwrap();
        assert!(!Arc::ptr_eq(&previous, &fresh));
        assert!(!fresh.project_context_retired());
        assert!(!fresh.lsp_features().hover);
        assert!(!fresh.has_corsa_bridge());
    }
}

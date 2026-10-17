//! Ordinary document operations detect configuration edits without watch events.
use super::{Arc, ServerState, Url, tests};
use futures::executor::block_on;
use std::path::Path;

fn current(state: &Arc<ServerState>, uri: &Url) -> Arc<ServerState> {
    state.document_project_state(uri).unwrap()
}

fn refresh(state: &Arc<ServerState>, uri: &Url) -> super::config_reload::ProjectConfigChanges {
    block_on(state.refresh_document_configuration_async(uri))
}

fn write_base(path: &Path, target: f64) {
    std::fs::write(
        path,
        serde_json::json!({"vueCompilerOptions":{"target":target}}).to_string(),
    )
    .unwrap();
}

#[test]
fn ordinary_boundary_reloads_inherited_options_and_preserves_sibling_and_unsaved_source() {
    let (fixture, state, a, b) = tests::fixture();
    let root = fixture.path().join("packages/a");
    let base = root.join("base.json");
    write_base(&base, 2.7);
    std::fs::write(root.join("tsconfig.json"), r#"{"extends":"./base.json"}"#).unwrap();
    state.documents.open(
        a.clone(),
        "<template>unsaved</template>".into(),
        7,
        "vue".into(),
    );
    let sibling = current(&state, &b);
    assert_eq!(
        current(&state, &a).type_checker_vue_version().as_str(),
        "2.7"
    );
    assert!(refresh(&state, &a).errors.is_empty());
    let previous = current(&state, &a);
    let generation = state.project_routing_generation();
    assert!(refresh(&state, &a).errors.is_empty());
    assert_eq!(state.project_routing_generation(), generation);
    assert!(Arc::ptr_eq(&previous, &current(&state, &a)));

    write_base(&base, 3.0);
    assert!(refresh(&state, &a).errors.is_empty());
    let next = current(&state, &a);
    assert!(!Arc::ptr_eq(&previous, &next));
    assert_eq!(next.type_checker_vue_version().as_str(), "3");
    assert_eq!(
        next.documents.text(&a).as_deref(),
        Some("<template>unsaved</template>")
    );
    assert_eq!(next.documents.version(&a), Some(7));
    assert!(Arc::ptr_eq(&sibling, &current(&state, &b)));
}

#[test]
fn failed_boundary_retains_owner_without_repeated_errors_and_repairs_new_external_dependency() {
    let (fixture, state, a, _) = tests::fixture();
    let root = fixture.path().join("packages/a");
    let config = root.join("tsconfig.json");
    std::fs::write(&config, r#"{"vueCompilerOptions":{"target":2.7}}"#).unwrap();
    current(&state, &a);
    assert!(refresh(&state, &a).errors.is_empty());
    let previous = current(&state, &a);
    let stamp = previous.corsa_request_stamp();
    let outside = fixture.path().join("shared");
    std::fs::create_dir_all(&outside).unwrap();
    let base = outside.join("new-base.json");
    std::fs::write(&config, r#"{"extends":"../../shared/new-base.json"}"#).unwrap();
    let rejected = refresh(&state, &a);
    assert_eq!(rejected.errors.len(), 1);
    assert!(Arc::ptr_eq(&previous, &current(&state, &a)));
    assert!(stamp.is_current(&previous));
    let watched = state.configuration_watch_paths();
    assert!(watched.contains(&super::paths::physical_path(&base)));
    assert!(refresh(&state, &a).errors.is_empty());
    assert!(Arc::ptr_eq(&previous, &current(&state, &a)));
    write_base(&base, 3.0);
    assert!(refresh(&state, &a).errors.is_empty());
    assert_eq!(current(&state, &a).type_checker_vue_version().as_str(), "3");
    assert!(!Arc::ptr_eq(&previous, &current(&state, &a)));
}

#[test]
fn vite_factory_runs_once_per_attempt_and_stable_or_failed_boundaries_do_not_evaluate_again() {
    let (fixture, state, a, _) = tests::fixture();
    let root = fixture.path().join("packages/a");
    let counter = root.join("factory-count.txt");
    std::fs::write(&counter, "").unwrap();
    let config = root.join("vite.config.mjs");
    let source = "import { appendFileSync } from 'node:fs'; export default () => { appendFileSync(new URL('./factory-count.txt', import.meta.url), 'x'); return {vue:{languageServer:{hover:false}}}; };";
    std::fs::write(&config, source).unwrap();
    current(&state, &a);
    assert_eq!(std::fs::read_to_string(&counter).unwrap(), "x");
    assert!(refresh(&state, &a).errors.is_empty());
    assert_eq!(std::fs::read_to_string(&counter).unwrap(), "x");
    let stable = current(&state, &a);
    assert!(refresh(&state, &a).errors.is_empty());
    assert_eq!(std::fs::read_to_string(&counter).unwrap(), "x");
    assert!(Arc::ptr_eq(&stable, &current(&state, &a)));

    let tsconfig = root.join("tsconfig.json");
    std::fs::write(&tsconfig, r#"{"extends":"./missing-base.json"}"#).unwrap();
    assert_eq!(refresh(&state, &a).errors.len(), 1);
    assert_eq!(std::fs::read_to_string(&counter).unwrap(), "xx");
    assert!(refresh(&state, &a).errors.is_empty());
    assert_eq!(std::fs::read_to_string(&counter).unwrap(), "xx");
    assert!(Arc::ptr_eq(&stable, &current(&state, &a)));
    std::fs::write(&tsconfig, "{}").unwrap();
    assert!(refresh(&state, &a).errors.is_empty());
    assert_eq!(std::fs::read_to_string(&counter).unwrap(), "xxx");
    assert!(!Arc::ptr_eq(&stable, &current(&state, &a)));
}

#[test]
fn incomplete_failed_package_resolution_retries_next_operation_without_repeating_notification() {
    let (fixture, state, a, _) = tests::fixture();
    let root = fixture.path().join("packages/a");
    let config = root.join("vite.config.mjs");
    let counter = root.join("factory-count.txt");
    std::fs::write(&counter, "").unwrap();
    current(&state, &a);
    assert!(refresh(&state, &a).errors.is_empty());
    let retained = current(&state, &a);
    std::fs::write(&config, "import { appendFileSync } from 'node:fs'; export default async () => { appendFileSync(new URL('./factory-count.txt', import.meta.url), 'x'); const {vue} = await import('missing-editor-plugin'); return {vue}; };").unwrap();
    assert_eq!(refresh(&state, &a).errors.len(), 1);
    assert_eq!(std::fs::read_to_string(&counter).unwrap(), "x");
    assert!(refresh(&state, &a).errors.is_empty());
    assert_eq!(std::fs::read_to_string(&counter).unwrap(), "xx");
    assert!(Arc::ptr_eq(&retained, &current(&state, &a)));
    let installed = root.join("node_modules/missing-editor-plugin");
    std::fs::create_dir_all(&installed).unwrap();
    std::fs::write(
        installed.join("package.json"),
        r#"{"type":"module","exports":"./index.mjs"}"#,
    )
    .unwrap();
    std::fs::write(
        installed.join("index.mjs"),
        "export const vue = {languageServer:{hover:true}};",
    )
    .unwrap();
    assert!(refresh(&state, &a).errors.is_empty());
    assert_eq!(std::fs::read_to_string(&counter).unwrap(), "xxx");
    assert!(current(&state, &a).lsp_features().hover);
    assert!(!Arc::ptr_eq(&retained, &current(&state, &a)));
    // The newly reported installed graph is confirmed without retiring an
    // unchanged semantic owner or invalidating its in-flight native requests.
    let confirmed = current(&state, &a);
    let pending = confirmed.corsa_request_stamp();
    assert!(refresh(&state, &a).errors.is_empty());
    assert_eq!(std::fs::read_to_string(&counter).unwrap(), "xxxx");
    assert!(Arc::ptr_eq(&confirmed, &current(&state, &a)));
    assert!(pending.is_current(&confirmed));
    assert!(refresh(&state, &a).errors.is_empty());
    assert_eq!(std::fs::read_to_string(&counter).unwrap(), "xxxx");
}

#[test]
fn a_known_input_edit_after_confirmation_preparation_still_replaces_the_owner() {
    let (fixture, state, a, _) = tests::fixture();
    let root = fixture.path().join("packages/a");
    let config = root.join("vite.config.mjs");
    std::fs::write(
        root.join("policy.mjs"),
        "export default {languageServer:{hover:false}};",
    )
    .unwrap();
    std::fs::write(
        &config,
        "import vue from './policy.mjs'; export default {vue};",
    )
    .unwrap();
    let previous = current(&state, &a);
    let stamp = previous.corsa_request_stamp();
    let key = super::ProjectContextKey::from(root.clone());
    let record = state
        .configuration_records()
        .into_iter()
        .find(|(candidate, _)| candidate == &key)
        .unwrap()
        .1;
    assert!(record.stable_confirmation());
    let mut scope = super::watched::ConfigEventScope::for_owners(vec![key.clone()]);
    scope
        .confirmations
        .push((key, record.input_observations().clone()));
    let edited = config.clone();
    super::lock_order_tests::at_config_install(move || {
        let mut bytes = std::fs::read(&edited).unwrap();
        bytes.extend_from_slice(b"\n// a real edit after checked evaluation\n");
        std::fs::write(edited, bytes).unwrap();
    });
    let result = state.apply_project_config_paths(&[Url::from_file_path(config).unwrap()], scope);
    assert!(result.errors.is_empty());
    let next = current(&state, &a);
    assert!(!Arc::ptr_eq(&previous, &next));
    assert!(!stamp.is_current(&previous));
    assert!(!next.lsp_features().hover);
}

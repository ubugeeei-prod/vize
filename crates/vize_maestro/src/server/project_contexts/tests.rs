//! Production document dispatch retains original per-owner navigation workers.
use super::MaestroServer;
use crate::{
    runtime::block_on,
    source_project::{SnapshotRefusal, navigation::NavigationRefusal},
};
use serde_json::{Value, json};
use std::{path::Path, sync::Arc};
use tower_lsp::{
    LspService,
    lsp_types::{FileChangeType, FileEvent, Location, Position, Url},
};
use vize_l0::cstr;

const INPUT: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/project-navigation-cache-8371.json"
);

struct Fixture {
    root: tempfile::TempDir,
    service: LspService<MaestroServer>,
    input: Value,
    uris: Vec<Url>,
}

type FixtureResult<T> = Result<T, Box<dyn std::error::Error>>;

fn member<'a>(input: &'a Value, name: &str) -> FixtureResult<&'a Value> {
    input
        .get(name)
        .ok_or_else(|| "authored fixture member".into())
}

fn file(root: &Path, name: &str, source: &str) -> FixtureResult<Url> {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().ok_or("fixture value")?)?;
    std::fs::write(&path, source)?;
    Ok(Url::from_file_path(path).map_err(|()| "fixture value")?)
}

fn fixture() -> FixtureResult<Fixture> {
    let root = tempfile::tempdir()?;
    file(
        root.path(),
        "vite.config.mjs",
        "export default { vue: {} };",
    )?;
    for package in ["a", "b"] {
        file(root.path(), &cstr!("packages/{package}/package.json"), "{}")?;
        file(
            root.path(),
            &cstr!("packages/{package}/vite.config.mjs"),
            "export default { vue: {} };",
        )?;
    }
    let (service, socket) = LspService::new(|client| {
        let mut server = MaestroServer::new(client);
        server.initial_diagnostics = None;
        server
    });
    drop(socket);
    let server = service.inner();
    server.state.set_workspace_root(root.path().to_path_buf());
    server
        .state
        .set_workspace_folders(vec![root.path().to_path_buf()]);
    server.state.load_workspace_config(root.path());
    server.state.apply_lsp_initialization_options(Some(
        &json!({"lint":false,"typecheck":false,"ecosystem":false}),
    ));
    let input: Value = serde_json::from_str(INPUT)?;
    let source = member(&input, "source")?.as_str().ok_or("fixture value")?;
    let uris = ["Root.ts", "packages/a/src/One.ts", "packages/b/src/Two.ts"]
        .map(|name| -> FixtureResult<Url> {
            let uri = file(root.path(), name, source)?;
            server.state.documents.open(
                uri.clone(),
                source.into(),
                member(&input, "version")?.as_i64().ok_or("fixture value")? as i32,
                member(&input, "languageId")?
                    .as_str()
                    .ok_or("fixture value")?
                    .into(),
            );
            Ok(uri)
        })
        .into_iter()
        .collect::<FixtureResult<Vec<_>>>()?;
    Ok(Fixture {
        root,
        service,
        input,
        uris,
    })
}

fn position(fixture: &Fixture) -> FixtureResult<Position> {
    Ok(serde_json::from_value(
        member(&fixture.input, "definitionPosition")?.clone(),
    )?)
}

fn definition(fixture: &Fixture, uri: &Url) -> FixtureResult<Location> {
    Ok(Location::new(
        uri.clone(),
        serde_json::from_value(member(&fixture.input, "definitionRange")?.clone())?,
    ))
}

fn references(fixture: &Fixture, uri: &Url) -> FixtureResult<Vec<Location>> {
    member(&fixture.input, "referenceRanges")?
        .as_array()
        .ok_or("fixture value")?
        .iter()
        .map(|range| -> FixtureResult<Location> {
            Ok(Location::new(
                uri.clone(),
                serde_json::from_value(range.clone())?,
            ))
        })
        .collect()
}

#[test]
fn primary_and_nested_document_dispatch_reuse_actual_owner_and_program_workers() {
    let fixture = fixture().unwrap();
    let server = fixture.service.inner();
    let primary = server.with_project_state(server.state.clone());
    assert!(Arc::ptr_eq(
        server.navigation.as_ref().unwrap(),
        primary.navigation.as_ref().unwrap()
    ));
    let mut worker_ids = Vec::new();
    let mut ordinary_missing = server.with_project_state(server.state.clone());
    ordinary_missing.navigation = None;
    assert_eq!(
        ordinary_missing
            .native_navigation_project()
            .err()
            .unwrap()
            .code,
        tower_lsp::jsonrpc::ErrorCode::InternalError
    );
    for uri in &fixture.uris {
        let first = block_on(server.for_document(uri));
        let navigation = first.navigation.as_ref().unwrap();
        assert_eq!(
            block_on(navigation.definition(uri, position(&fixture).unwrap())),
            Ok(Some(definition(&fixture, uri).unwrap()))
        );
        let worker = navigation.cached_program_worker_for_test(uri).unwrap();
        assert_eq!(worker.1, (1, 1));
        let second = block_on(server.for_document(uri));
        assert!(Arc::ptr_eq(&first.state, &second.state));
        assert!(Arc::ptr_eq(navigation, second.navigation.as_ref().unwrap()));
        assert_eq!(
            block_on(second.navigation.as_ref().unwrap().references(
                uri,
                position(&fixture).unwrap(),
                true
            )),
            Ok(references(&fixture, uri).unwrap())
        );
        assert_eq!(
            navigation.cached_program_worker_for_test(uri),
            Some((worker.0, (1, 2)))
        );
        worker_ids.push(worker.0);
    }
    assert_eq!(server.state.cached_project_states().len(), 2);
    let a = block_on(server.for_document(&fixture.uris[1]));
    let b = block_on(server.for_document(&fixture.uris[2]));
    assert!(!Arc::ptr_eq(&a.state, &b.state));
    assert!(!Arc::ptr_eq(
        a.navigation.as_ref().unwrap(),
        b.navigation.as_ref().unwrap()
    ));
    assert!(
        worker_ids
            .iter()
            .enumerate()
            .all(|(index, worker)| !worker_ids[index + 1..].contains(worker))
    );
}

#[test]
fn watcher_retires_old_navigation_and_preserves_the_unaffected_owner_cache() {
    let fixture = fixture().unwrap();
    let server = fixture.service.inner();
    let a = &fixture.uris[1];
    let b = &fixture.uris[2];
    let old_a = block_on(server.for_document(a));
    let sibling = block_on(server.for_document(b));
    let old_navigation = old_a.navigation.as_ref().unwrap();
    old_a
        .state
        .apply_lsp_initialization_options(Some(&json!({"nativeLinkedEditing":true,"rename":true})));
    let crate::server::NativeLinkedNamesRoute::Native(ticket) =
        old_a.state.capture_native_linked_route()
    else {
        panic!("actual native linked route required")
    };
    let sibling_navigation = sibling.navigation.as_ref().unwrap();
    assert_eq!(
        block_on(old_navigation.definition(a, position(&fixture).unwrap())),
        Ok(Some(definition(&fixture, a).unwrap()))
    );
    assert_eq!(
        block_on(sibling_navigation.definition(b, position(&fixture).unwrap())),
        Ok(Some(definition(&fixture, b).unwrap()))
    );
    let sibling_worker = sibling_navigation
        .cached_program_worker_for_test(b)
        .unwrap();
    let config = file(
        fixture.root.path(),
        "packages/a/vite.config.mjs",
        r#"export default { vue: {"languageServer":{"hover":false}} };"#,
    )
    .unwrap();
    server.state.observe_project_config_events(&[FileEvent {
        uri: config,
        typ: FileChangeType::CHANGED,
    }]);
    assert!(old_a.state.project_context_retired());
    assert!(old_a.state.cached_project_navigation().is_none());
    let stale_view = server.with_project_state(old_a.state.clone());
    assert!(stale_view.navigation.is_none());
    assert_eq!(
        stale_view.native_navigation_project().err().unwrap().code,
        tower_lsp::jsonrpc::ErrorCode::ContentModified
    );
    let linked = serde_json::from_value(
        json!({"textDocument":{"uri":a},"position":fixture.input["definitionPosition"]}),
    )
    .unwrap();
    assert_eq!(
        block_on(stale_view.native_linked_editing(linked, ticket))
            .unwrap_err()
            .code,
        tower_lsp::jsonrpc::ErrorCode::ContentModified
    );
    assert_eq!(
        block_on(old_navigation.definition(a, position(&fixture).unwrap())),
        Err(NavigationRefusal::Host(SnapshotRefusal::Superseded))
    );
    assert!(old_navigation.cached_program_worker_for_test(a).is_none());
    let new_a = block_on(server.for_document(a));
    let params = serde_json::from_value(
        json!({"textDocument":{"uri":a},"position":fixture.input["definitionPosition"]}),
    )
    .unwrap();
    let rerouted = block_on(stale_view.for_pos(&params));
    assert!(Arc::ptr_eq(&new_a.state, &rerouted.state));
    assert!(Arc::ptr_eq(
        new_a.navigation.as_ref().unwrap(),
        rerouted.navigation.as_ref().unwrap()
    ));
    let new_b = block_on(server.for_document(b));
    assert!(!Arc::ptr_eq(
        old_navigation,
        new_a.navigation.as_ref().unwrap()
    ));
    assert!(Arc::ptr_eq(
        sibling_navigation,
        new_b.navigation.as_ref().unwrap()
    ));
    assert_eq!(
        block_on(
            new_a
                .navigation
                .as_ref()
                .unwrap()
                .definition(a, position(&fixture).unwrap())
        ),
        Ok(Some(definition(&fixture, a).unwrap()))
    );
    assert_eq!(
        block_on(new_b.navigation.as_ref().unwrap().references(
            b,
            position(&fixture).unwrap(),
            true
        )),
        Ok(references(&fixture, b).unwrap())
    );
    assert_eq!(
        sibling_navigation.cached_program_worker_for_test(b),
        Some((sibling_worker.0, (1, 2)))
    );
    assert_eq!(server.state.documents.version(a), Some(1));
}

#[test]
fn primary_configuration_replacement_refuses_retained_owner_and_drains_its_worker() {
    let fixture = fixture().unwrap();
    let server = fixture.service.inner();
    let uri = &fixture.uris[0];
    let navigation = server.navigation.as_ref().unwrap();
    assert_eq!(
        block_on(navigation.definition(uri, position(&fixture).unwrap())),
        Ok(Some(definition(&fixture, uri).unwrap()))
    );
    let config = file(
        fixture.root.path(),
        "vite.config.mjs",
        r#"export default { vue: {"languageServer":{"hover":false}} };"#,
    )
    .unwrap();
    // Marking occurs in the registry; publication/admission refuse immediately,
    // before the ordinary server watcher drains the old primary owner.
    server.state.observe_project_config_events(&[FileEvent {
        uri: config.clone(),
        typ: FileChangeType::CHANGED,
    }]);
    assert_eq!(
        block_on(navigation.definition(uri, position(&fixture).unwrap())),
        Err(NavigationRefusal::Host(SnapshotRefusal::Superseded))
    );
    block_on(tower_lsp::LanguageServer::did_change_watched_files(
        server,
        tower_lsp::lsp_types::DidChangeWatchedFilesParams {
            changes: vec![FileEvent {
                uri: config,
                typ: FileChangeType::CHANGED,
            }],
        },
    ));
    assert!(navigation.cached_program_worker_for_test(uri).is_none());
    let replacement = block_on(server.for_document(uri));
    assert!(!Arc::ptr_eq(
        navigation,
        replacement.navigation.as_ref().unwrap()
    ));
    assert_eq!(
        block_on(
            replacement
                .navigation
                .as_ref()
                .unwrap()
                .definition(uri, position(&fixture).unwrap())
        ),
        Ok(Some(definition(&fixture, uri).unwrap()))
    );
}

async fn protocol_send(
    service: &mut LspService<MaestroServer>,
    value: Value,
) -> FixtureResult<Option<Value>> {
    use tower::Service;
    futures::future::poll_fn(|cx| service.poll_ready(cx)).await?;
    Ok(service
        .call(serde_json::from_value::<tower_lsp::jsonrpc::Request>(
            value,
        )?)
        .await?
        .map(serde_json::to_value)
        .transpose()?)
}

#[test]
fn registered_client_watcher_reloads_imported_vite_policy_through_the_protocol() {
    use futures::{SinkExt, StreamExt};
    block_on(crate::runtime::timeout(std::time::Duration::from_secs(30), async {
        let root = tempfile::tempdir().unwrap();
        file(
            root.path(),
            "vite.config.mjs",
            "export default { vue: {} };",
        )
        .unwrap();
        let helper = file(
            root.path(),
            "shared-policy.mjs",
            "export default { languageServer: { hover: false } };",
        )
        .unwrap();
        for package in ["a", "b"] {
            file(root.path(), &cstr!("packages/{package}/package.json"), "{}").unwrap();
            let config = if package == "a" {
                "import vue from '../../shared-policy.mjs'; export default { vue };"
            } else {
                "export default { vue: {} };"
            };
            file(
                root.path(),
                &cstr!("packages/{package}/vite.config.mjs"),
                config,
            )
            .unwrap();
        }
        let (mut service, mut socket) = LspService::new(|client| {
            let mut server = MaestroServer::new(client);
            server.initial_diagnostics = None;
            server
        });
        let root_uri = Url::from_directory_path(root.path()).unwrap();
        let initialized = protocol_send(&mut service, json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"rootUri":root_uri,"capabilities":{"workspace":{"didChangeWatchedFiles":{"dynamicRegistration":true}}},"initializationOptions":{"lint":false,"typecheck":false,"ecosystem":false}}})).await.unwrap().unwrap();
        assert!(initialized.get("error").is_none());
        let register = async {
            loop {
                let request = socket.next().await.expect("client registration request");
                if request.method() == "client/registerCapability" {
                    let params = request.params().unwrap().clone();
                    socket
                        .send(tower_lsp::jsonrpc::Response::from_ok(
                            request.id().unwrap().clone(),
                            Value::Null,
                        ))
                        .await
                        .unwrap();
                    break params;
                }
            }
        };
        let (notification, registration) = futures::join!(
            protocol_send(
                &mut service,
                json!({"jsonrpc":"2.0","method":"initialized","params":{}})
            ),
            register
        );
        let scenario = async {
        assert_eq!(notification.unwrap(), None);
        let registration = &registration["registrations"][0];
        assert_eq!(registration["method"], "workspace/didChangeWatchedFiles");
        assert!(registration["registerOptions"]["watchers"].as_array().unwrap().iter().any(|watcher| watcher["globPattern"] == "**/*.{vue,ts,tsx,mts,cts,js,jsx,mjs,cjs}"));
        let source = "<template><div /></template>\n<style>.app { color:red }</style>";
        let a = file(root.path(), "packages/a/src/App.vue", "<template />").unwrap();
        let b = file(root.path(), "packages/b/src/App.vue", "<template />").unwrap();
        for uri in [&a, &b] {
            assert_eq!(protocol_send(&mut service, json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"vue","version":7,"text":source}}})).await.unwrap(), None);
        }
        let hover = |id: i32, uri: &Url| json!({"jsonrpc":"2.0","id":id,"method":"textDocument/hover","params":{"textDocument":{"uri":uri},"position":{"line":1,"character":15}}});
        let before_a = protocol_send(&mut service, hover(2, &a)).await.unwrap().unwrap();
        assert_eq!(before_a, json!({"jsonrpc":"2.0","id":2,"result":null}));
        let before_b = protocol_send(&mut service, hover(3, &b)).await.unwrap().unwrap();
        assert!(before_b["result"].is_object());
        let sibling = service.inner().state.document_project_state(&b).unwrap();
        std::fs::write(
            helper.to_file_path().unwrap(),
            "export default { languageServer: { hover: true } };",
        )
        .unwrap();
        assert_eq!(protocol_send(&mut service, json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{"changes":[{"uri":helper,"type":2}]}})).await.unwrap(), None);
        let after_a = protocol_send(&mut service, hover(4, &a)).await.unwrap().unwrap();
        assert_eq!(
            after_a,
            json!({"jsonrpc":"2.0","id":4,"result":before_b["result"]})
        );
        let after_b = protocol_send(&mut service, hover(5, &b)).await.unwrap().unwrap();
        assert_eq!(
            after_b,
            json!({"jsonrpc":"2.0","id":5,"result":before_b["result"]})
        );
        assert!(Arc::ptr_eq(
            &sibling,
            &service.inner().state.document_project_state(&b).unwrap()
        ));
        assert_eq!(
            service.inner().state.documents.text(&a).as_deref(),
            Some(source)
        );
        assert_eq!(service.inner().state.documents.version(&a), Some(7));
        };
        // Keep the client reading while notifications and requests are sent.
        let client = async {
            while let Some(request) = socket.next().await {
                if let Some(id) = request.id() {
                    socket.send(tower_lsp::jsonrpc::Response::from_ok(id.clone(), Value::Null)).await.unwrap();
                }
            }
        };
        futures::pin_mut!(scenario, client);
        if let futures::future::Either::Right(_) = futures::future::select(scenario, client).await {
            panic!("client disconnected before the watcher scenario completed");
        }
    })).expect("registered watcher protocol completes within 30 seconds");
}

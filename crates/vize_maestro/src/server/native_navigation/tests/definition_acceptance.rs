//! One experimental same-file Artifact definition frontdoor, with whole bytes.
#![expect(
    clippy::disallowed_types,
    reason = "the acceptance client retains the same real server/navigation Arc"
)]

mod fixtures;
mod wire;

use super::{MaestroServer, block_on};
use crate::source_project::navigation::NativeNavigationProject;
use fixtures::{AUTHORED, CHANGED, CROSS_FILE, OTHER, OTHER_SOURCE, URI};
use serde_json::{Value, json};
use std::sync::Arc;
use tower_lsp::{LspService, Server};

#[derive(Clone, Copy)]
enum Case {
    Success,
    Disabled,
    Stale,
    CrossFile,
}

impl Case {
    fn name(self) -> &'static str {
        match self {
            Self::Success => "same-file-success-and-unknown",
            Self::Disabled => "native-registration-disabled",
            Self::Stale => "native-ready-before-edit",
            Self::CrossFile => "cross-file-component-unadmitted",
        }
    }
}

struct Observation {
    case: Case,
    tape: Value,
    client: Vec<Value>,
    server: Vec<Value>,
    driver: Option<Value>,
}

async fn drive(
    case: Case,
    peer: wire::Peer,
    navigation: Arc<NativeNavigationProject<'static>>,
) -> Option<Value> {
    peer.send(fixtures::initialize());
    let initialize = peer.response(1).await?;
    if initialize.get("error").is_some() {
        return Some(json!({"initialize":initialize}));
    }
    peer.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
    if matches!(case, Case::CrossFile) {
        peer.send(fixtures::open(OTHER, OTHER_SOURCE, 1));
        peer.ready(OTHER, 1).await?;
    }
    let source = if matches!(case, Case::CrossFile) {
        CROSS_FILE
    } else {
        AUTHORED
    };
    peer.send(fixtures::open(URI, source, 1));
    let readiness = peer.ready(URI, 1).await?;
    let mut details = json!({"initialize":initialize,"readiness":readiness});
    let shutdown = match case {
        Case::Success => {
            peer.send(fixtures::definition(2, 2, 11));
            peer.response(2).await?;
            peer.send(fixtures::definition(3, 4, 14));
            peer.response(3).await?;
            peer.send(json!({"jsonrpc":"2.0","id":4,
                "method":"vize/unknownArtifactDefinition","params":{}}));
            peer.response(4).await?;
            peer.send(json!({"jsonrpc":"2.0","method":"textDocument/didClose",
                "params":{"textDocument":{"uri":URI}}}));
            peer.send(fixtures::definition(5, 4, 14));
            peer.response(5).await?;
            6
        }
        Case::Disabled => {
            peer.send(fixtures::definition(2, 4, 14));
            peer.response(2).await?;
            peer.send(json!({"jsonrpc":"2.0","method":"textDocument/didClose",
                "params":{"textDocument":{"uri":URI}}}));
            3
        }
        Case::Stale => {
            peer.send(fixtures::definition(2, 4, 14));
            peer.response(2).await?;
            let (entered, release) = navigation
                .pause_next_definition_publication_for_test()
                .ok()?;
            peer.send(fixtures::definition(3, 4, 14));
            let actual_ready = entered.await.ok()?;
            details["heldNativeReady"] = match actual_ready {
                Ok(value) => json!({"result":value}),
                Err(error) => json!({"actualRefusalDebug":vize_l0::cstr!("{error:?}")}),
            };
            peer.send(json!({"jsonrpc":"2.0","method":"textDocument/didChange",
                "params":{"textDocument":{"uri":URI,"version":2},
                "contentChanges":[{"text":CHANGED}]}}));
            peer.ready(URI, 2).await?;
            peer.send(fixtures::definition(4, 1, 13));
            peer.response(4).await?;
            details["releasedNativeReady"] = json!(release.send(()).is_ok());
            peer.response(3).await?;
            peer.send(json!({"jsonrpc":"2.0","method":"textDocument/didClose",
                "params":{"textDocument":{"uri":URI}}}));
            5
        }
        Case::CrossFile => {
            peer.send(fixtures::definition(2, 11, 4));
            peer.response(2).await?;
            for uri in [URI, OTHER] {
                peer.send(json!({"jsonrpc":"2.0","method":"textDocument/didClose",
                    "params":{"textDocument":{"uri":uri}}}));
            }
            3
        }
    };
    peer.send(json!({"jsonrpc":"2.0","id":shutdown,"method":"shutdown"}));
    peer.response(shutdown).await?;
    peer.send(json!({"jsonrpc":"2.0","method":"exit"}));
    Some(details)
}

fn observe(case: Case) -> Observation {
    let (service, socket) = if matches!(case, Case::Disabled) {
        // Real Maestro/Tower dispatch with the native custom registration absent.
        // No replacement response or legacy definition wrapper is installed.
        LspService::new(MaestroServer::new)
    } else {
        crate::server::build_lsp_service()
    };
    let navigation = Arc::clone(service.inner().navigation.as_ref().unwrap());
    let (peer, input, output) = wire::pipes(case.name());
    let stopped = peer.clone();
    let client = peer.clone();
    let (service, exit) = crate::server::with_exit_signal(service);
    let server = async move {
        let transport = Server::new(input, output, socket).serve(service);
        let _ = futures::future::select(Box::pin(transport), Box::pin(exit)).await;
        stopped.stopped();
    };
    let driver = async move {
        let observed = drive(case, client.clone(), navigation).await;
        client.finish();
        observed
    };
    let completed = block_on(crate::runtime::timeout(
        std::time::Duration::from_secs(30),
        futures::future::join(driver, server),
    ));
    let deadline_elapsed = completed.is_err();
    let driver = completed.ok().and_then(|(driver, ())| driver);
    let tape = peer.tape();
    // Preserve the complete/partial byte record before decode or any verdict.
    println!("VIZE_ARTIFACT_DEFINITION_TAPE {}", tape);
    println!(
        "VIZE_ARTIFACT_DEFINITION_COMPLETION {}",
        json!({"case":case.name(),"deadlineSeconds":30,"deadlineElapsed":deadline_elapsed})
    );
    let executable = std::env::current_exe().unwrap();
    let image = std::fs::read(&executable).unwrap();
    println!(
        "VIZE_ARTIFACT_DEFINITION_CONTEXT {}",
        json!({
            "case":case.name(),"featureExperimentalSourceNavigation":cfg!(feature="experimental-source-navigation"),
            "source":option_env!("GITHUB_SHA"),"testProcessPid":std::process::id(),
            "testExecutable":{"path":executable,"bytes":image.len(),"sha256":fixtures::hash(&image)},
            "compiledSources":fixtures::compiled_sources(),"inputs":fixtures::inputs(),
            "driver":driver,"deadlineSeconds":30,"deadlineElapsed":deadline_elapsed,
            "transport":"in-process actual tower-lsp framed codec",
            "defaultShippingAdmission":false,"normalDifferentialNativeAdmission":false,
            "crossFileComponentAuthority":false
        })
    );
    assert!(!deadline_elapsed, "whole observation deadline");
    let client: Vec<u8> = serde_json::from_value(tape["clientConsumed"].clone()).unwrap();
    let server: Vec<u8> = serde_json::from_value(tape["serverWritten"].clone()).unwrap();
    Observation {
        case,
        client: wire::decode(&client, true).unwrap(),
        server: wire::decode(&server, true).unwrap(),
        tape,
        driver,
    }
}

fn response(observation: &Observation, id: i32) -> Value {
    let replies = observation
        .server
        .iter()
        .filter(|value| value["id"] == id)
        .collect::<Vec<_>>();
    assert_eq!(replies.len(), 1, "one whole actual response per request");
    replies[0].clone()
}

fn validate_transport(observation: &Observation, reply_ids: &[i32], methods: &[&str]) {
    assert_eq!(
        observation.tape["clientOffered"],
        observation.tape["clientConsumed"]
    );
    assert_eq!(observation.tape["unreadBytes"], 0);
    assert_eq!(observation.tape["transportStopped"], true);
    assert!(observation.driver.is_some());
    assert_eq!(
        observation
            .client
            .iter()
            .map(|v| v["method"].as_str().unwrap())
            .collect::<Vec<_>>(),
        methods
    );
    assert_eq!(
        observation
            .server
            .iter()
            .filter_map(|v| v["id"].as_i64())
            .collect::<Vec<_>>(),
        reply_ids
            .iter()
            .map(|id| i64::from(*id))
            .collect::<Vec<_>>()
    );
    assert_eq!(observation.client[0], fixtures::initialize());
    let initialized = response(observation, 1);
    assert!(initialized.get("error").is_none());
    assert!(initialized["result"]["capabilities"].is_object());
    assert_eq!(
        initialized["result"]["serverInfo"],
        json!({"name":"vize-maestro","version":vize_carton::VERSION})
    );
    let notifications = match observation.case {
        Case::Success | Case::Disabled => vec![
            fixtures::initialized_log(),
            fixtures::diagnostics(URI, Some(1)),
            fixtures::diagnostics(URI, None),
        ],
        Case::Stale => vec![
            fixtures::initialized_log(),
            fixtures::diagnostics(URI, Some(1)),
            fixtures::diagnostics(URI, Some(2)),
            fixtures::diagnostics(URI, None),
        ],
        Case::CrossFile => vec![
            fixtures::initialized_log(),
            fixtures::diagnostics(OTHER, Some(1)),
            fixtures::diagnostics(URI, Some(1)),
            fixtures::diagnostics(URI, None),
            fixtures::diagnostics(OTHER, None),
        ],
    };
    assert_eq!(
        observation
            .server
            .iter()
            .filter(|v| v.get("method").is_some())
            .cloned()
            .collect::<Vec<_>>(),
        notifications,
        "whole ordered notifications, including their complete URI/version fields"
    );
    let shutdown = *reply_ids.last().unwrap();
    assert_eq!(
        response(observation, shutdown),
        fixtures::result(shutdown, Value::Null)
    );
    assert_eq!(
        observation.client.last(),
        Some(&json!({"jsonrpc":"2.0","method":"exit"}))
    );
}

#[test]
fn same_file_artifact_definition_retains_complete_native_wire_and_unknown_refusal() {
    let observed = observe(Case::Success);
    validate_transport(
        &observed,
        &[1, 2, 3, 4, 5, 6],
        &[
            "initialize",
            "initialized",
            "textDocument/didOpen",
            "vize/nativeDefinition",
            "vize/nativeDefinition",
            "vize/unknownArtifactDefinition",
            "textDocument/didClose",
            "vize/nativeDefinition",
            "shutdown",
            "exit",
        ],
    );
    for id in [2, 3] {
        assert_eq!(
            response(&observed, id),
            fixtures::result(id, fixtures::target(1, 6, 8))
        );
    }
    assert_eq!(
        response(&observed, 4),
        fixtures::method_not_found(4, "vize/unknownArtifactDefinition")
    );
    assert_eq!(
        response(&observed, 5),
        fixtures::error(5, -32602, "Native document unavailable")
    );
}

#[test]
fn native_definition_registration_disabled_cannot_borrow_standard_maestro_success() {
    let observed = observe(Case::Disabled);
    validate_transport(
        &observed,
        &[1, 2, 3],
        &[
            "initialize",
            "initialized",
            "textDocument/didOpen",
            "vize/nativeDefinition",
            "textDocument/didClose",
            "shutdown",
            "exit",
        ],
    );
    assert_eq!(
        response(&observed, 2),
        fixtures::method_not_found(2, "vize/nativeDefinition")
    );
}

#[test]
fn actual_ready_native_definition_is_superseded_before_publication_after_wire_edit() {
    let observed = observe(Case::Stale);
    validate_transport(
        &observed,
        &[1, 2, 4, 3, 5],
        &[
            "initialize",
            "initialized",
            "textDocument/didOpen",
            "vize/nativeDefinition",
            "vize/nativeDefinition",
            "textDocument/didChange",
            "vize/nativeDefinition",
            "textDocument/didClose",
            "shutdown",
            "exit",
        ],
    );
    assert_eq!(
        response(&observed, 2),
        fixtures::result(2, fixtures::target(1, 6, 8))
    );
    assert_eq!(
        observed.driver.as_ref().unwrap()["heldNativeReady"],
        json!({"result":fixtures::target(1, 6, 8)})
    );
    assert_eq!(
        observed.driver.as_ref().unwrap()["releasedNativeReady"],
        true
    );
    assert_eq!(
        response(&observed, 3),
        fixtures::error(3, -32801, "Native source superseded")
    );
    assert_eq!(
        response(&observed, 4),
        fixtures::result(4, fixtures::target(0, 27, 31))
    );
}

#[test]
fn cross_file_registered_component_keeps_whole_genuine_native_refusal() {
    let observed = observe(Case::CrossFile);
    validate_transport(
        &observed,
        &[1, 2, 3],
        &[
            "initialize",
            "initialized",
            "textDocument/didOpen",
            "textDocument/didOpen",
            "vize/nativeDefinition",
            "textDocument/didClose",
            "textDocument/didClose",
            "shutdown",
            "exit",
        ],
    );
    assert_eq!(
        response(&observed, 2),
        fixtures::error(2, -32010, "Native original SFC observation refused")
    );
}

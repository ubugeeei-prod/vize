//! Scripted process controls exercise the receiver, never the native provider.

use std::{
    panic::AssertUnwindSafe,
    process::{Command, Stdio},
};

use serde_json::{Value, json};

use super::super::super::LspProcess;
use super::Fixture;

const TEXT: &str = "<template />\n";

fn fixture(mode: &str) -> Fixture {
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join("src")).unwrap();
    std::fs::write(project.path().join("src/App.vue"), TEXT).unwrap();
    let program = project.path().join("confirmation-peer.cjs");
    std::fs::write(
        &program,
        include_str!("../../../programs/observe-lsp-save-confirmation.cjs"),
    )
    .unwrap();
    let mut command = Command::new("node");
    command.arg(program).arg(mode);
    let lsp = LspProcess::spawn_command(command, Stdio::piped());
    let runtime = project.path().join("no-native-provider-in-this-control");
    Fixture {
        project,
        lsp,
        id: 2,
        publications: Default::default(),
        mismatches: Vec::new(),
        publication_sequences: Vec::new(),
        runtime,
        runtime_distribution: None,
    }
}

fn publish(fixture: &mut Fixture) -> Value {
    super::publish(fixture, "src/App.vue", TEXT, 2, false, json!([]), json!([]))
}

fn reason(panic: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = panic.downcast_ref::<String>() {
        return message.clone();
    }
    panic.downcast_ref::<&str>().unwrap().to_string()
}

#[test]
fn coalesced_answer_requires_and_records_the_explicit_save_confirmation() {
    let mut fixture = fixture("coalesced");
    assert_eq!(publish(&mut fixture), json!([]));
    let sequence = &fixture.publication_sequences[0];
    assert_eq!(
        sequence["saveConfirmationAfterFirstPublication"],
        json!({
            "jsonrpc":"2.0", "method":"textDocument/didSave",
            "params":{"textDocument":{"uri":fixture.uri("src/App.vue")}}
        })
    );
    assert_eq!(sequence["actual"], sequence["expected"]);
    fixture.shutdown();
}

#[test]
fn missing_or_foreign_confirmation_cannot_turn_one_empty_prompt_into_success() {
    for mode in ["missing", "wrong-uri", "wrong-version"] {
        let mut fixture = fixture(mode);
        let panic =
            std::panic::catch_unwind(AssertUnwindSafe(|| publish(&mut fixture))).unwrap_err();
        let message = reason(panic);
        let (failure, transport) = message.split_once("\nLSP process status: ").unwrap();
        let (rejection, _) = failure.split_once("; seen: ").unwrap();
        assert_eq!(
            rejection,
            format!(
                "timed out waiting for LSP message: {}",
                std::sync::mpsc::RecvTimeoutError::Disconnected
            ),
            "{mode}: the controlled peer closed without a matching completion"
        );
        let (_, stderr) = transport.split_once("\nLSP stderr:\n").unwrap();
        assert_eq!(
            stderr, "confirmation-peer: saved\n",
            "the scripted save phase was reached"
        );
        let packets = fixture.lsp.published_diagnostics();
        assert_eq!(packets.len(), if mode == "missing" { 1 } else { 2 });
        assert_eq!(fixture.mismatches.len(), usize::from(mode != "missing"));
        if mode == "wrong-uri" {
            assert_eq!(packets[1]["params"]["uri"], "file:///foreign/App.vue");
        }
        if mode == "wrong-version" {
            assert_eq!(packets[1]["params"]["version"], 3);
        }
        assert!(
            fixture.publication_sequences.is_empty(),
            "no complete sequence was accepted"
        );
    }
}

#[test]
fn a_wrong_whole_payload_rejects_even_with_two_matching_uri_version_packets() {
    let mut fixture = fixture("wrong-payload");
    let panic = std::panic::catch_unwind(AssertUnwindSafe(|| publish(&mut fixture))).unwrap_err();
    assert_eq!(
        reason(panic).lines().next(),
        Some(
            "assertion `left == right` failed: whole publication sequence with its authored completion confirmation"
        )
    );
    assert_eq!(fixture.mismatches.len(), 1);
    assert_eq!(
        fixture.mismatches[0]["actual"]["params"]["diagnostics"][0]["message"],
        "controlled wrong diagnostic"
    );
}

#[test]
fn late_duplicates_are_retained_and_late_whole_payload_changes_still_reject() {
    for mode in ["late-duplicate", "late-wrong"] {
        let mut fixture = fixture(mode);
        assert_eq!(publish(&mut fixture), json!([]));
        fixture.request("src/App.vue", "test/drain", Value::Null, json!({}));
        assert_eq!(fixture.lsp.published_diagnostics().len(), 3);
        if mode == "late-duplicate" {
            assert!(fixture.mismatches.is_empty());
            fixture.shutdown();
        } else {
            assert_eq!(fixture.mismatches.len(), 1);
            assert_eq!(
                fixture.mismatches[0]["actual"]["params"]["diagnostics"][0]["message"],
                "controlled late diagnostic"
            );
            let panic =
                std::panic::catch_unwind(AssertUnwindSafe(|| fixture.shutdown())).unwrap_err();
            let message = reason(panic);
            let (rejection, _) = message.split_once(": ").unwrap();
            assert_eq!(rejection, "all whole publication mismatches");
        }
    }
}

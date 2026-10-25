//! ACK-to-EOF capability bound to the complete native image and initialize result.
//!
//! Data contract retained from the independently owned #9239 shutdown repair;
//! no SDK/package migration or process-observer dependency is needed here.

use corsa_lsp::LspClient;
use lsp_types::request::Shutdown;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

mod completion;

const LINUX_X64: &str = "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4";
const DARWIN_ARM64: &str = "a82f731365ad69d5c4c15f5e18fba4584bf3b7b839960172a76c3462b5114bf2";

pub(super) fn eof_capability(executable: &Path, initialized: &Value) -> bool {
    if !known_initialize(initialized) {
        return false;
    }
    let Ok(digest) = File::open(executable).and_then(image_digest) else {
        return false;
    };
    let digest: vize_l0::String = digest
        .iter()
        .map(|byte| vize_l0::cstr!("{byte:02x}"))
        .collect();
    known_binary(digest.as_str())
}

fn image_digest(mut image: impl Read) -> io::Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        match image.read(&mut buffer) {
            Ok(0) => return Ok(hasher.finalize().into()),
            Ok(count) => hasher.update(buffer.get(..count).ok_or(io::ErrorKind::InvalidData)?),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
}

pub(super) async fn close(client: &LspClient, shutdown_to_eof: bool) -> corsa_lsp::Result<()> {
    if !shutdown_to_eof {
        return client.graceful_close().await;
    }
    // EditorLspSession is a non-cloneable owner: shutdown, discard and Drop
    // converge on its closed latch, and only shutdown sends this request. SDK
    // clones are private responders/overlays and cannot initiate another one.
    // A fresh subscription therefore has no earlier Shutdown to mistake for
    // this one. The SDK does not expose numeric request IDs; this is temporal
    // single-owner correlation, not an independent numeric-ID proof.
    let events = client.subscribe();
    // Native queues its null ACK before the dispatcher's final Info log. EOF
    // immediately after ACK can cancel that log's queue write into stderr.
    // Drain that producer completion before EOF, without filtering any stream.
    let acknowledged = client.request_without_params::<Shutdown>().await;
    let completed = acknowledged.and_then(|()| completion::drain(&events));
    // Always close/reap, preserving the first protocol/completion failure.
    let closed = client.close().await;
    completed.and(closed)
}

fn known_initialize(initialized: &Value) -> bool {
    *initialized == known_contract()
}

fn known_contract() -> Value {
    serde_json::json!({
        "capabilities": {
            "_vs_onAutoInsertProvider": {
                "_vs_triggerCharacters": [
                    ">"
                ]
            },
            "_vs_referencesProvider": true,
            "callHierarchyProvider": true,
            "codeActionProvider": {
                "codeActionKinds": [
                    "quickfix",
                    "source.organizeImports",
                    "source.removeUnusedImports",
                    "source.sortImports",
                    "source.fixAll"
                ]
            },
            "codeLensProvider": {
                "resolveProvider": true
            },
            "completionProvider": {
                "completionItem": {
                    "labelDetailsSupport": true
                },
                "resolveProvider": true,
                "triggerCharacters": [
                    ".",
                    "\"",
                    "'",
                    "`",
                    "/",
                    "@",
                    "<",
                    "#",
                    " ",
                    "*"
                ]
            },
            "definitionProvider": true,
            "diagnosticProvider": {
                "identifier": "typescript",
                "interFileDependencies": true,
                "workspaceDiagnostics": false
            },
            "documentFormattingProvider": true,
            "documentHighlightProvider": true,
            "documentOnTypeFormattingProvider": {
                "firstTriggerCharacter": "{",
                "moreTriggerCharacter": [
                    "}",
                    ";",
                    "\n"
                ]
            },
            "documentRangeFormattingProvider": true,
            "documentSymbolProvider": true,
            "experimental": {
                "customMultiDocumentHighlightProvider": true,
                "customSourceDefinitionProvider": true
            },
            "foldingRangeProvider": true,
            "hoverProvider": true,
            "implementationProvider": true,
            "inlayHintProvider": true,
            "linkedEditingRangeProvider": true,
            "positionEncoding": "utf-16",
            "referencesProvider": true,
            "renameProvider": {
                "prepareProvider": true
            },
            "selectionRangeProvider": true,
            "semanticTokensProvider": {
                "full": true,
                "legend": {
                    "tokenModifiers": [],
                    "tokenTypes": []
                },
                "range": true
            },
            "signatureHelpProvider": {
                "retriggerCharacters": [
                    ")"
                ],
                "triggerCharacters": [
                    "(",
                    ",",
                    "<"
                ]
            },
            "textDocumentSync": {
                "change": 2,
                "openClose": true,
                "save": true
            },
            "typeDefinitionProvider": true,
            "workspace": {
                "fileOperations": {
                    "willRename": {
                        "filters": [
                            {
                                "pattern": {
                                    "glob": "**/*.{ts,tsx,js,jsx,cts,cjs,mts,mjs,json}"
                                },
                                "scheme": "file"
                            }
                        ]
                    }
                }
            },
            "workspaceSymbolProvider": true
        },
        "serverInfo": {
            "name": "typescript-go",
            "version": "7.0.2"
        }
    })
}

fn known_binary(digest: &str) -> bool {
    matches!(digest, LINUX_X64 | DARWIN_ARM64)
}

#[cfg(test)]
mod tests;

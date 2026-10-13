//! Native 7.0.2 shutdown capability, bound to actual binary and initialize bytes.

use corsa_lsp::process::ProcessObserver;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

const LINUX_X64: &str = "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4";
const DARWIN_ARM64: &str = "a82f731365ad69d5c4c15f5e18fba4584bf3b7b839960172a76c3462b5114bf2";

#[cfg(target_os = "linux")]
mod owned_image;

pub(super) fn eof_capability(
    executable: &Path,
    initialized: &Value,
    observer: &ProcessObserver,
) -> bool {
    if !known_initialize(initialized) {
        return false;
    }
    if let Ok(bytes) = fs::read(executable) {
        let digest: vize_l0::String = Sha256::digest(bytes)
            .iter()
            .map(|byte| vize_l0::cstr!("{byte:02x}"))
            .collect();
        if known_binary(digest.as_str()) {
            return true;
        }
    }
    #[cfg(target_os = "linux")]
    return owned_image::eof_capability(observer);
    #[cfg(not(target_os = "linux"))]
    {
        let _ = observer;
        false
    }
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
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn full_native_binary_and_exact_initialize_identity_are_both_required() {
        let initialized = known_contract();
        assert!(known_initialize(&initialized));
        assert!(known_binary(LINUX_X64));
        assert!(known_binary(DARWIN_ARM64));
        for value in [
            json!(null),
            json!({}),
            json!({"serverInfo":{"name":"typescript-go"}}),
            json!({"serverInfo":{"name":"other","version":"7.0.2"}}),
            json!({"serverInfo":{"name":"typescript-go","version":"7.0.3"}}),
            json!({"serverInfo":{"name":"typescript-go","version":7.02}}),
        ] {
            assert!(!known_initialize(&value));
        }
        let mut changed = initialized.clone();
        changed["capabilities"]["positionEncoding"] = json!("utf-8");
        assert!(!known_initialize(&changed));
        let mut changed = initialized.clone();
        changed["capabilities"]
            .as_object_mut()
            .unwrap()
            .remove("diagnosticProvider");
        assert!(!known_initialize(&changed));
        let temp = tempfile::tempdir().unwrap();
        let fake = temp.path().join("native");
        let observer = ProcessObserver::default();
        fs::write(&fake, b"Version 7.0.2\n").unwrap();
        assert!(!eof_capability(&fake, &initialized, &observer));
        fs::write(&fake, b"Version 7.0.2\n\0\xff").unwrap();
        assert!(!eof_capability(&fake, &initialized, &observer));
        assert!(!eof_capability(
            &temp.path().join("absent"),
            &initialized,
            &observer
        ));
        for digest in ["", "4f2de678", &"f".repeat(64), &format!("{LINUX_X64}00")] {
            assert!(!known_binary(digest));
        }
    }
}

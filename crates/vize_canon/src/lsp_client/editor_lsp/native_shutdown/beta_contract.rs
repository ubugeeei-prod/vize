//! Entire Initialize result bound independently to the two observed beta images.
//! Linux: Actions 40229044048; Darwin: #10741 live-image and complete-wire proof.
use serde_json::Value;

pub(super) fn initialize() -> Value {
    serde_json::json!({
        "capabilities": {
            "positionEncoding": "utf-16",
            "textDocumentSync": {
                "openClose": true,
                "change": 2,
                "save": true
            },
            "completionProvider": {
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
                ],
                "resolveProvider": true,
                "completionItem": {
                    "labelDetailsSupport": true
                }
            },
            "hoverProvider": true,
            "signatureHelpProvider": {
                "triggerCharacters": [
                    "(",
                    ",",
                    "<"
                ],
                "retriggerCharacters": [
                    ")"
                ]
            },
            "definitionProvider": true,
            "typeDefinitionProvider": true,
            "implementationProvider": true,
            "referencesProvider": true,
            "documentHighlightProvider": true,
            "documentSymbolProvider": true,
            "codeActionProvider": {
                "codeActionKinds": [
                    "quickfix",
                    "source.organizeImports.ts",
                    "source.removeUnusedImports.ts",
                    "source.sortImports.ts",
                    "source.fixAll.ts"
                ]
            },
            "codeLensProvider": {
                "resolveProvider": true
            },
            "workspaceSymbolProvider": true,
            "documentFormattingProvider": true,
            "documentRangeFormattingProvider": true,
            "documentOnTypeFormattingProvider": {
                "firstTriggerCharacter": "{",
                "moreTriggerCharacter": [
                    "}",
                    ";",
                    "\n"
                ]
            },
            "renameProvider": {
                "prepareProvider": true
            },
            "foldingRangeProvider": true,
            "selectionRangeProvider": true,
            "callHierarchyProvider": true,
            "linkedEditingRangeProvider": true,
            "semanticTokensProvider": {
                "legend": {
                    "tokenTypes": [],
                    "tokenModifiers": []
                },
                "range": true,
                "full": true
            },
            "inlayHintProvider": true,
            "diagnosticProvider": {
                "identifier": "typescript",
                "interFileDependencies": true,
                "workspaceDiagnostics": false
            },
            "workspace": {
                "fileOperations": {
                    "willRename": {
                        "filters": [
                            {
                                "scheme": "file",
                                "pattern": {
                                    "glob": "**/*.{ts,tsx,js,jsx,cts,cjs,mts,mjs,json}"
                                }
                            }
                        ]
                    }
                }
            },
            "experimental": {
                "customSourceDefinitionProvider": true,
                "customMultiDocumentHighlightProvider": true
            },
            "_vs_onAutoInsertProvider": {
                "_vs_triggerCharacters": [
                    ">"
                ]
            },
            "_vs_referencesProvider": true
        },
        "serverInfo": {
            "name": "typescript",
            "version": "7.1.0-beta"
        }
    })
}

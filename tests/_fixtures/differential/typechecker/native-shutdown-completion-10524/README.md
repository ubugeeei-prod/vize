# Native shutdown completion (#10524)

These frozen envelopes exercise the exact native provider's final dispatch
notification. They are protocol controls, not authored-language or whole-product
acceptance claims. The Rust parser test compares every complete result and error;
the owned wire peer separately records full request, ACK, notification and EOF
packets, including rejection and process-cleanup controls.

The native Linux executable is SHA-256
`4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4`.
The npm 7.0.2 package records upstream revision
`2bd066d87f5bafd315be9f40889d0a60b9e58e0b`. At that revision,
[request dispatch](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/lsp/server.go)
queues the Shutdown result before its final Info notification. The
[logger](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/lsp/logger.go)
falls back to stderr if cancellation prevents that notification's queue write.

Production admission still requires both the complete executable hash and the
complete Initialize result. `EditorLspSession` cannot be cloned; its client is
private. Only `shutdown` invokes the native helper, and all retirement paths
converge on `finish_close`'s closed latch. `discard` and spawn cleanup close the
transport without requesting Shutdown. Private responder and overlay clones do
not request Shutdown, and the pinned SDK has no protocol-sending Drop. Thus a
fresh pre-request subscription has no earlier Shutdown on this owned connection.

The SDK does not expose its numeric request ID. Correlation is explicitly the
fresh subscription and the single owned Shutdown, not independent numeric-ID
binding. Numeric fields are validated only as the exact provider's positive
integer grammar. An arbitrary valid numeric value is not a rejection control.
Every other log remains observable, and all failure paths still close/reap the
owned process. The completion wait uses the SDK's existing two-second shutdown
bound without sleeps, retries or stderr filtering.

The unchanged original handoff test failed at source
`970f984e82fdbec4aec080e8c02b3c486824379b` in run `40165338616`, job
`121085184870`, with complete stderr
`handled method 'shutdown' (7) in 6.071µs\n`.
A paired one-shot replay in run `40167059473` passed at frozen baseline
`7fa14a7a2bfccf9b1af924d23f5f3a660d8b3904` and candidate
`80dbb438d36174de014826df58d21df138dc3b99`, each with empty complete stderr.
Both compiled unchanged test executables were byte-identical, SHA-256
`8cdc2ff011da01f49ce51fbce239a1a7d71f712cc70bf5b69bfe1ee720ea5c12`.
This isolates the lifecycle problem from the unrelated batching change; a later
pass does not invalidate the retained failure.

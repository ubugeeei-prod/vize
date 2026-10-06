# Response-backed editor readiness without semantic diagnostic sweeps

Issue: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698).
[Paired decision](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6012837401).

The maintainer reports approximately ten-second hover and slow initial loading
on published 0.435.0. The complete existing public 400-SFC/134-TS graph reproduces
13.289 seconds for an immediate first script hover, followed by 54–60 ms repeated
hover responses. Those are local observations, not a same-worker source comparison
or a general performance guarantee.

The reusable native editor session currently requests full document diagnostics
for every dirty mirrored URI to acknowledge notification installation. It also
uses that request to drain each group of 128 notifications. The response payload
is ignored, but the native provider still performs semantic diagnostic work.

Use response-backed `textDocument/documentSymbol` requests for these readiness
acknowledgements. Retain every dirty identity, stable ordering, the close/query
or stable-live-document topology barrier, the 128-notification drain, the existing
16-request concurrency bound, and failure-before-ready state updates. Preserve
all existing worker ownership, deadlines, cancellation and native-empty behavior.
Actual diagnostic collection continues through the unchanged diagnostic method.

The pinned SDK 7.0.2 source revision is
[`2bd066d87f5bafd315be9f40889d0a60b9e58e0b`](https://github.com/microsoft/typescript-go/tree/2bd066d87f5bafd315be9f40889d0a60b9e58e0b).
Its [server handlers](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/lsp/server.go)
process preceding notifications and acquire the same language-service snapshot
before dispatching either request. The
[symbol provider](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/ls/symbols.go)
traverses the installed source AST without acquiring the semantic checker.
Failed or unsupported responses must still refuse readiness.

The existing Actions pair builds the actual common ancestor and successor on one
worker with the same compiler recipe, provider graph, original sources and config.
Add four immediate cold requests before its unchanged initial-type completion,
ten-second idle, priming sweep, twenty warm requests and complete invalidation,
cancellation, close/reopen, cross-root and native-retirement controls. Retain all
whole envelopes, payloads, timestamps, descendant CPU/RSS and binary/source receipts.
The installed-public replay keeps its original request sequence.

TODO: qualify the exact source pair, original full authored Markdown/diagnostics,
held-hover and worker lifetime laws, full native/non-native checks and protected
instruction ceilings. Publish measured results only after actual execution;
no gain, 10x, Program-build elimination, or installed-release credit is claimed.
Broader batch-performance and warm-input-copy work remain separate.

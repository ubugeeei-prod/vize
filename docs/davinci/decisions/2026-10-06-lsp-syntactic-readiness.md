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

The first exact e608 source pair used development-inheriting profile.ci and
observed first script hover 19,721.983 → 14,322.078 ms, with warm hover near45 ms.
All78 complete measured envelopes and all18 complete ordered notifications
matched; the aggregate failed its historical93-frame server count (actual98).
Current didChange publishes prompt and complete versioned answers for all five
awaited edits. Independently authored whole publication vectors retain both
warnings, URI/version/range/message fields, duplicate publications and ordering;
93 client frames and80 responses remain required. Historical public sequences
keep their separate binding. The original failed aggregate is not accepted.

Main-thread observations retain approximately12.05 CPU seconds on both sides.
Native editor CPU falls15.30 → 3.98 seconds, but this is not adequate hover latency
or a shipping-profile gain. Add opt-in numeric-only phase begin/end/count/status
observations for existing preparation and native readiness operations, without
clocks when disabled, additional queries/stages or payload changes. The same
source pair now requires actual release-profile builds, exact Cargo profiles,
fresh required artifacts and byte-exact staged ELF/source/recipe custody.
Default source-launch CI bindings stay strict; the release recipe is explicit.

Close regressive Draft#8038 after its own measured failures, preserving branch
and evidence and open#7698. No bulk source or timing is imported here. Delivered
snapshot-source correctness b034 remains separate. TODO: qualify shipping cold,
repeated and during-diagnostic hover plus all whole invalidation/lifetime/full
and protected gates; the remaining14-second observation is unacceptable.

The shipping pair also retains one additional complete script hover after the
actual `collect_corsa_diagnostics` producer marker and before terminal initial
diagnostics; it must equal the whole first-hover answer. All74 original requests
and four cold requests remain, with94 client frames,81 complete responses and
all18 source-authored notifications for current feedback. Passive stderr/native
phase counters distinguish actual collection from pre-lock worker scheduling.
Draft#8038 was actually closed at09:25:17Z; its factual paired closure is
[recorded on#7698](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6013334183).

[The phase/publication/shipping decision](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6013356884)
pairs this same-PR successor; fresh exact execution remains pending.

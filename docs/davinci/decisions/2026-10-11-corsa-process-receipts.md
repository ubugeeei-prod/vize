# Opt-in Corsa process receipts

## Decision and scope

Prepare a local transport capability for #6879 using only the authenticated
published `corsa_core` and `corsa_lsp` 1.14.0 crates. Preserve all original
features, dependencies, manifests and public shutdown `Result<()>` signatures.
Existing unobserved commands still inherit stderr. No protocol messages, native
flags, diagnostic policy or SDK stderr filtering change.

An opted-in command passes a spawn-specific observation directly into the owning
child guard. A shared observer retains the actual PID, every stderr byte,
`ExitStatus`, EOF completeness, process/read errors, timeout, kill request, reap,
completion and elapsed time. Hash/equality use observer identity, never mutable
contents. Guard completion verifies the observation belongs to the same PID;
a mismatch cannot become a completed receipt. The guard exists before transport
construction, so a transport-construction error also drops the owning guard.

`vendor/corsa-published-custody.json` records all 95 original archive members,
the original archive SHA256 values and each changed/new file. Exactly four
published files change: core's `src/lib.rs`/`src/process.rs` and LSP's
`src/lib.rs`/`src/lsp/client.rs`. The new core helper has 211 lines. All other 91
original files are byte-identical, including normalized Cargo manifests, locked
upstream dependencies, original tests, documentation and `.cargo_vcs_info.json`.
Both archives pin revision `34bb731bdcaf5d4c154596b0c32912cbffd60b18` in
[corsa-bind](https://github.com/ubugeeei-prod/corsa-bind/tree/34bb731bdcaf5d4c154596b0c32912cbffd60b18).
The MIT license text omitted by the published archives is retained from that
[exact revision](https://raw.githubusercontent.com/ubugeeei-prod/corsa-bind/34bb731bdcaf5d4c154596b0c32912cbffd60b18/LICENSE).
No upstream changes are made. Root Cargo.lock removes only the two substituted
registry source/checksum pairs; package versions and all other lock bytes stay
unchanged.

## Consumer API

The existing LSP process module exposes the opt-in types without a new direct
Vize dependency or feature:

```rust
use corsa_lsp::process::{ProcessObserver, ProcessReceipt};

let observer = ProcessObserver::default();
spawn.command = spawn.command.with_process_observer(observer.clone());
let client = corsa_lsp::LspClient::spawn(spawn).await?;
let protocol_close = client.graceful_close().await;
let receipts: Vec<ProcessReceipt> = observer.receipts();
```

The SDK does not classify stderr or turn a native nonzero status into its old
public close result. The caller must inspect both `protocol_close` and the
completed receipt. Raw bytes stay raw; an adapter needing a text error must
preserve non-UTF8 bytes, for example by hex encoding, rather than replacing them.

## Reproducible laws and retained native results

`tests/_fixtures/corsa-process-receipt/verify-vendor.mjs` checks every original
member and its narrowly declared patch. `run.mjs` authenticates the actual
native executable by canonical path, complete SHA256, byte size and successful
`--version`, then runs the isolated SDK laws with whole raw stdout/stderr/status
retained. Set `VIZE_CORSA_PATH` to the native 7.0.2 executable and run:

```sh
node tests/_fixtures/corsa-process-receipt/verify-vendor.mjs
node tests/_fixtures/corsa-process-receipt/run.mjs
```

The OS-child laws retain exact NUL/non-UTF8 stderr and exit 17, unexpected stderr
with exit 0, the expected EOF line plus an unexpected complete suffix, deadline
kill/reap with actual signal status, drop kill/reap, repeated shutdown and the
unchanged default unobserved spawn. None of these bodies are filtered.

The configured native laws use the genuine SDK LSP owner, attached API pipe,
strict selected tsconfig, complete root/source-file names and full diagnostics.
Every case retains all 83 source names, the sole authored Leaf.ts, source/config
bytes, selected compiler options, semantic request/response bodies, native owner
PID, complete raw stderr, terminal status and close timing. Broken/repaired
source retains TS2304 while removing exactly TS2322. Actual local native 7.0.2
SHA256 `a82f731365ad69d5c4c15f5e18fba4584bf3b7b839960172a76c3462b5114bf2`
produced these four controls:

| Control                                         | Diagnostics | SDK close result        | Actual native result                    |
| ----------------------------------------------- | ----------- | ----------------------- | --------------------------------------- |
| Configured broken                               | 2322, 2304  | graceful close succeeds | exit 0; entire 34-byte exit EOF line    |
| Configured repaired                             | 2304        | graceful close succeeds | exit 0; same entire EOF line            |
| Shutdown/exit with input deliberately kept open | 2322, 2304  | close succeeds          | exit 1; entire context-canceled line    |
| Native built-in watcher                         | 2322, 2304  | reader error            | exit 2; entire 2,997-byte nil-map panic |

All four were reaped with EOF-complete stderr and no deadline/kill. Close success
alone therefore does not prove native exit success. The built-in watcher control
intentionally differs from Canon's dynamic watcher capability; it proves panic
custody, not a failure of Canon's normal watcher setup. The harness requires a
real panic control within its bounded repetitions and retains every attempt.
The semantic event log is not raw stdio framing. Earlier raw framed protocol
controls remain separate evidence; successful SDK `graceful_close()` establishes
its acknowledgment/params-less sequence through the pinned implementation.

## Limits and remaining work

A descendant keeping an inherited stderr pipe open can delay EOF. Draining is
bounded after process retirement; a missed deadline returns an error and an
incomplete receipt, while its reader continues retaining bytes until EOF. No
incomplete capture can authorize successful classification. Non-UTF8, extra
bytes and actual native failures remain available to the consumer.

This change only supplies the transport capability. Canon still needs explicit
closure of the actual configured diagnosing client before accepting a completed
batch result; drop cleanup cannot propagate its failure. An adapter may classify
only the entire exact EOF body after acknowledged standard graceful close,
completed EOF capture with no process/read errors, actual exit 0, no timeout/kill
and verified reap. Unexpected bytes, nonzero exit, panic, missing receipts and
incomplete capture must remain observable. That integration, exact-source
Actions, protected full gates, measured paired benchmarks and released/public
acceptance remain required; this local prototype provides none of those credits.
The parent owns the paired #6879 decision comment and integration delivery.

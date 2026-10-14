# Native TypeScript shutdown custody

This records the read-only shutdown investigation for
[#6879](https://github.com/ubugeeei-prod/vize/issues/6879) and the syntax
completion qualification in #8614. The required whole-program semantic
handoff and native teardown are separate operations. Correct diagnostics do
not prove a healthy native process exit.

## Authenticated runtime and sources

The local experiment executed the canonical installed
`@typescript/typescript-darwin-arm64@7.0.2/lib/tsc`, 23,653,616 bytes, SHA256
`a82f731365ad69d5c4c15f5e18fba4584bf3b7b839960172a76c3462b5114bf2`.
Its version response is `Version 7.0.2`; its package manifest identifies native
source `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`.

The inspected Corsa 1.14.0 process, API, LSP and JSON-RPC sources match their
published crate archive bytes. All four archive hashes match Cargo.lock.
The frozen benchmark mechanism was traced from Git source
`d55b8238f89bab4418a1a961bd8db1c84911cb42`, including its actual executor,
diagnostic transport selection, explicit project attachment and teardown.
This is not an execution of the retained Linux benchmark binary on Darwin.

## Observed protocol outcomes

Each native session retains whole framed stdin/stdout, whole stderr, actual
executable identity, argv, PID, full inputs, exit status, forced-kill result
and monotonic reap timing. Configured controls use the diagnosing LSP's own
`custom/initializeAPISession` pipe, open and observe the same explicit config,
release their snapshots and close the attachment after reaping its owner.

| Actual transport and close                                     | Outcome                                                       |
| -------------------------------------------------------------- | ------------------------------------------------------------- |
| API snapshot release, stdin EOF                                | Native exit 0; stderr empty                                   |
| API `exit` or `shutdown` request                               | Unknown method, RPC error -32603                              |
| API exit notification without EOF                              | Does not stop; deadline kills and reaps                       |
| Configured LSP shutdown acknowledgment, exit notification, EOF | Three controls exit 0; each retains the exact `exit: EOF` log |
| Configured LSP shutdown acknowledgment, exit, stdin kept open  | Three controls exit 1 with `context canceled`                 |

All six configured controls completed the same full diagnostic response:
error TS2322, error TS2304 and suggestion TS6133, with `strict: true` and all
83 native source-file names observed through the same owning process. The
successful standard closes reaped in approximately 1.5–4.7 ms after close;
these are protocol measurements, not a typechecker speed claim.

A separate builtin-watcher control, with dynamic registration disabled,
exited 2 with a retained nil-map panic during native watcher shutdown despite
complete diagnostics. Canon advertises dynamic registration; that control
does not establish a current Canon watcher failure. It establishes why a
teardown allowance cannot rely only on completed diagnostic output.

The retained frozen escaped-keyword CLI sample returns TS1260 and TS2304
with the generic CLI-to-session warning and the same EOF line. Its exact
source selects whole-program completion after parser-aborted CLI output,
then selects the diagnosing LSP when the older aggregate/file API endpoints
are unsupported. The API configuration receipt is attached to that LSP
owner; it does not identify a separate API process as diagnostic authority.

## Cause and decision

The pinned native [LSP server](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/lsp/server.go)
uses EOF as its exit handler's terminal result and logs it before outer
termination handling. Its [logger](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/lsp/logger.go)
normally sends structured log notifications, then falls back to stderr when
the outgoing queue's context has already been cancelled. The exit/EOF race
therefore explains the expected log after a correct standard close.

There is no supported file-log flag in the pinned
[native command](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/cmd/tsgo/lsp.go).
Actual `--logFile`, `--logVerbosity` and `--logToFile` probes exit 2 with
unknown flags. Initialization log verbosity exists, but Off suppresses all
native logged errors and is not an acceptable correction.

Corsa 1.14.0 hard-codes inherited stderr, exposes no capture hook, and its
private process guard discards exit status, including after timeout killing
and reaping. Its successful close result cannot prove native exit 0. Keep
the normal API EOF close and the standard acknowledged LSP shutdown/exit
sequence; do not suppress native stderr or change the benchmark verdict.

TODO: add an opt-in dependency receipt that retains all raw native stderr,
actual exit status, timeout/kill result and protocol acknowledgment. Drain
stderr concurrently. An exact expected EOF lifecycle classification requires
completed semantic responses, shutdown acknowledgment, transmitted exit,
native status 0 and successful reap; every other stderr or process failure
remains observable. A native source correction is an alternative. Neither
dependency nor native source was modified by this investigation.

The executor may separately distinguish required semantic completion from
runtime degradation while preserving the full error and real failure
warnings. This grants no native teardown, benchmark, original App baseline,
installed acceptance or release credit. Full evidence remains in
`/tmp/vize-native-api-close-20261011/`, including the reproducible scripts,
whole per-session receipts, source hashes and report; preserve that directory.

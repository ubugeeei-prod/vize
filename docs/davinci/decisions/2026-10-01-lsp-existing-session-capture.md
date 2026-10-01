# Retained LSP session observations — #6883

Paired with the [#6883 decision comment](https://github.com/ubugeeei-prod/vize/issues/6883#issuecomment-5931315699).

## Decision

Keep the existing stdio regression sessions and their authored assertions. When
`VIZE_LSP_REQUIRE_SOURCE_BUILD=1`, attach a passive observer to that same process
and preserve its complete transport under `target/differential/lsp-sessions/`.
The pinned checkout has 102 direct `LspSession` construction sites in 75 tooling
test files, plus shared wrappers. These are construction-site counts, not runtime
session totals or a fix-history denominator.

The observer reuses the launcher's existing source-receipt validation and actual
`--version` probe. It starts no additional CLI, version probe or Cargo build.
Ordinary local launches keep their existing candidate order and do not create
source observations. Source-built launches still reject missing, stale or corrupt
receipts before any fallback.

## Artifact contract

Every observed session receives a unique process/session directory containing
`client.bin`, `server.bin`, `stderr.bin` and `observation.json`. Client bytes mean
the existing client's attempted stdin writes, including a batched request and
cancellation; they do not assert that a closed pipe delivered every attempt.
Server and stderr bytes come directly from the process streams, without a JSON
projection or Unicode conversion.

Metadata retains the source and binary identities, original build receipt, raw
version probe, caller source paths/line positions/hashes, exact stream hashes
and byte counts, and process exit/signal/error. The shared response adapter's
strict frame decoder indexes the complete raw client/server streams and marks
malformed or truncated framing explicitly. Framing success is not response parity.

Capture retains at most 16 MiB total per session. Observed counts continue after
the bound, and each shortened stream is marked truncated. Initial metadata says
`awaiting-process-close`; process close writes the final bytes and receipt,
including failed or killed executions. An abrupt job termination may leave the
initial incomplete record. The original runtime assertions remain authoritative.

The existing differential evidence upload already retains this recursive
directory. No workflow, build stage, test deferral or performance ceiling changes
are needed. The observer buffers bounded bytes during the session and performs
artifact writes and frame indexing at construction/close, outside request writes.

## Reconciliation and limits

These artifacts are pending observations, separate from the shared
`vize.differential.result` comparison report. They provide complete response and
state traffic for the existing executions; they do not freeze a current output
as a historical expected response. Workspace dependency bytes are explicitly
not captured. Caller hashes identify candidate source witnesses and do not prove
the original input, setup, request or provider closure is equivalent.

Every observation records zero whole fixes closed, native handled and native
equivalent. Original input/dependency matching, full historical requirements,
superseded behavior and current expected-response acceptance remain required
before any artifact can enter the fix-history corpus. #6883 stays open.

The source audit also corrects the `37d8e8e` diagnostic-version requirement:
`215ca5706` (#6260) intentionally permits an unversioned initial synchronous lint
publication before terminal versioned diagnostics. The retained test preserves
the unavailable-Corsa setup and the terminal-version requirement. An absolute
every-publication version assertion would contradict current public behavior.

## Verification

Local laws use a synthetic source-receipted echo subprocess and explicit frame
fixtures. They prove capture integrity, raw Unicode, failed exit retention,
bounded truncation, pending closure, caller source hashes and forged-source
rejection. They are not production LSP observations. Strict TypeScript and
selected formatting/lint checks pass. Fresh exact-head Actions, protected queue
execution and actual merge remain unverified for this change.

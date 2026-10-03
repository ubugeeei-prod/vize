# Native Maestro original Vue navigation retention

Tracked in [#6871](https://github.com/ubugeeei-prod/vize/issues/6871#issuecomment-5968682672)
and [#6872](https://github.com/ubugeeei-prod/vize/issues/6872#issuecomment-5968682930).
This extends the actually merged [Program/File worker](./2026-10-03-maestro-retained-file-navigation.md).

## Original owners and queries

The opt-in `vize/nativeDefinition` and `vize/nativeReferences` endpoints accept
the original editor-open Vue source. A dedicated worker invokes genuine
`lower_sfc_native` once per physical source snapshot and actual configuration
profile. The allocator, whole original `NativeSfcObservation`, descriptor,
selected scripts, Component, retained expressions/decoder maps and completed
File are ordinary thread-local stack owners throughout the existing mailbox
loop. Borrowed query work ends before the SFC observation drops; the allocator
and immutable snapshot outlive every native owner. Native values never need
Send/Sync, a self-reference, lifetime cast, unsafe code or shared AST allocation.

Only the private authentic `NativeSfcObservation::admitted` view admits a File.
Original SFC producer refusal retains the whole observation for repeated
requests. Script positions use the shared original File position API. Template
positions use each actual retained embed's original File resolution table,
same-owner binding and exact `JsExpr::authored_span` decoder/wrapper projection.
The consumer checks original AST, source and authored span identity and refuses
overlapping position observations. References scan existing original script
records and expression occurrences, preserving original binding identity,
setup shadowing, local import aliases, half-open UTF-8 names and complete entity
ranges before converting to LSP UTF-16. It adds no parser, AST walk, semantic
index, pipeline stage, legacy generator or level serialization. Its scans are
linear; no unmeasured indexing or performance improvement is claimed.

## Actual profile and publication

Vue requires the genuine `ServerState` host, not invented settings for a bare
document store. Source language is authoritative; standalone HTML is refused.
The existing server's SFC-role policy uses standard Vue. The copied profile
records actual configured Vue version, raw dialect override, legacy flag and
patterned-template flag. Version/dialect and genuine producer admission remain
distinct: an unchanged profile grants no native authority. Patterned templates
and legacy mode with a Vue 3 profile are explicit configuration refusals; other
versions pass to the original descriptor and retain its actual refusal.
The bounded preview uses the existing ordinary surface options, without
claiming admission for other experimental template options.

Physical snapshot and profile must both match a cached worker. The actual host
source check remains under the cache mutex before worker insertion. Final
authentic `ProjectQuery` publication checks source/version/cancellation and
compares the current actual Vue settings to the worker's profile. A mismatch
returns content-modified and retires the changed cached entry after publication,
outside the document guard. Configuration reads never re-enter DocumentStore;
there is no cache lock or document guard across an await. Retire checks current
configuration while holding the cache mutex, preserving a newly refreshed entry.

Existing cancellation, prompt didChange notification, source replacement,
close/reopen and non-blocking retirement apply to Vue. Sixteen actual live
worker threads and sixteen mailbox commands remain the resource bounds. Slots
release only after all original native owners drop, including unwind; no
silent eviction or per-request reparse is introduced.

## Validation and remaining work

The new laws cover repeated original descriptor/File/Program/embed identity,
one SFC producer invocation, Unicode/CRLF, exact entity and escaped identifier
ranges, reversed script order, setup shadowing, local aliases, template calls,
source holes, opaque styles, original sticky refusals, actual Vue version and
patterned-template configuration, final configuration/source freshness,
cancellation, close, mailbox saturation and genuine live-owner capacity.
Complete real service RPC envelopes cover original Vue definitions/references,
change/close/reopen, refusal and client cancellation.

The affected `vize_maestro` PR Rust builder and full differential recipe share
the same real feature-enabled SourceProject/RPC tests, minimal non-default
library check and strict all-targets Clippy action. Source-feature failure fails
the existing required Rust tier. Pure workflow contracts verify both callers
and the required aggregation; these do not substitute for compilation/runtime.
Local formatting and pure workflow contracts are separate from hosted runtime
validation. No local Cargo/npm install or compile/test campaign is claimed.
Exact-head Actions, protected full suites, immutable instruction ceilings and
actual merge are required before this slice is delivered.

This bounded consumer admits only the original producer's supported Vue 3
JS/TS SFC family. Vue dialects/macros/control constructs outside that family,
JSX/TSX, external/workspace navigation, L3/L4 retention, standard endpoint
routing, whole product adoption and complete fix history
[#6883](https://github.com/ubugeeei-prod/vize/issues/6883) remain unfinished.

The first hosted source head `2a2a0738` failed the actual feature compilation in
[Check 37119631818](https://github.com/ubugeeei-prod/vize/actions/runs/37119631818/job/111194076938).
Two response paths omitted propagation of the borrowed binding query's Result;
the dev-only original Program inspection also read an admission handle after
its authentic consumption. The correction propagates the original refusal and
captures only dev inspection data before the existing checked handoff. It adds
no parser call, admission constructor or altered test input/golden. Runtime,
minimal and strict feature checks were not reached at that failed head; it was
never queued. A fresh corrected head requires complete hosted acceptance.

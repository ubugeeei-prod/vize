# Validated warm LSP query surfaces

Issue: [#8008](https://github.com/ubugeeei-prod/vize/issues/8008).
Paired decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8008#issuecomment-5993441802).

The current warm request path validates an AliasContext and then reconstructs the host/dependency query surface, including import walks, materialized source maps and document packets. Overlapping fingerprint and project-member validation also reads the same disk inputs repeatedly within one lookup. This is a source-level explanation to test, not an accepted timing or CPU result.

Prepare a bounded immutable query surface inside that existing AliasContext revision. Reuse it only after the unchanged host/overlay/options/request-membership fingerprint and strong closed-file/configuration/project-member validation succeeds. Use one temporary InputStampCache only inside each individual cache lookup; discard it before any build/materialization and before the next lookup. Preserve same-mtime/same-length content and missing/config/package route checks. Do not use received watcher generations as proof that external files are unchanged.

The normal materialized project, virtual-document synchronization, native request/readiness/recovery, cancellation and stale-response guards remain in place. This caches generated input data, not native responses. Full public owned packets still clone, and distinct closed disk inputs still receive strong content validation. The existing eight-context bound remains; no new stage, IPC, process or persistent filesystem-observation cache is introduced.

Retain the complete original issue body and exact generator, including all 400 SFCs and 134 TS modules for the reported large case. The package/config files materialize options described by the reporter; they are not presented as literal files supplied in the report. Whole cached-versus-uncached host/dependency/materialized/map/document packets and closed dependency/config same-mtime/same-length edits, unsaved overlays, close/reopen, source options, requested membership, cross-root identities and fresh mirror-session ownership are source laws. Existing native cancellation/recovery/refusal guards and their complete laws remain unchanged.

TODO: reuse the existing Canon scaling Actions compare job for source-qualified before/head builds on one worker, identical dependencies and the original 400 generator/workspace, five rounds of the four reported requests in each warm process, whole raw client/server/results/exit packets and Linux process-tree CPU observations. Actual dependency versions must be disclosed separately from the report. No local native build, private benchmark, speedup, native-default/history completion or release acceptance is claimed. Fresh source Actions, the paired current-source observation, protected full suites/all unchanged instruction ceilings, actual reporter-credited merge and coordinated publication remain required.

The original report and generator SHA-256 values are respectively
`247d435038eb8c53f317dc6a5fdcdd216a19fa9bc58d155e355b5ed2c3eb4a58` and
`4d0e14d66ce6e7b0b5d2e32a51c39bbe985283aa94934616b4da0ff587d34649`.
The reported dependency/platform/timing observations remain historical, with
no transfer to current source execution or CPU measurements.

The [bounded source review checkpoint](https://github.com/ubugeeei-prod/vize/issues/8008#issuecomment-5993558094)
clears production4d3c only. Compare canonical physical temporary roots in the
complete cross-root membership law, including macOS `/var` symlink spelling.
The fixture names its current Rust source law; the original400 paired whole
wire/native-process/currentCPU observer remains pending. Session config PATH
equality is not whole native configuration execution, and a fresh mirror
session is not an actually executed new native process. Source checks remain
mandatory on the corrected head, with no earlier runtime or timing transfer.

[First source-gate repair](https://github.com/ubugeeei-prod/vize/issues/8008#issuecomment-5993762899)
retains actual Check37303985181 failures: production expect() was rejected before
Rust tests, and tooling4 found the moved consumer inventory stale. Stable
OnceLock::get_or_init follows the same fallible preparation and validated
revision, preserving the concurrent winner without panic or waiver. The existing
generator changes only the moved import row and context test offset. Every
original law/input and native guard remains. Current source tests and original400
whole-wire/CPU acceptance remain pending; no infrastructure cause is invented.

## Original400 same-worker qualification

Paired decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8008#issuecomment-5994315127).

The existing Canon scaling workflow retains its historical manual rows and
adds one automatic PR pair inside the same comparison job. The original
unchanged generator produces all 400 SFCs and 134 TS modules at the same
absolute workspace path for both sources. The driver rejects unrelated
production changes from the genuine common ancestor, requires identical
locks/toolchain and the existing successful source-build receipts, and uses
the same `cargo build --profile ci -p vize` recipe and physical locked Vue/native
runtime. The reported Vue 3.5.41 remains historical; the actual resolved locked
version and binary hashes are recorded without changing the original inputs.

After the actual initial native diagnostics and the retained ten-second idle,
a complete priming sweep is recorded separately. Every one of the reported
five rounds of four hover/definition/completion requests is retained. Complete
public response and request packets are compared, including resolve data;
only physically checked private 0700 session-root spellings are replaced
through a per-side bijection. Raw stdio bytes, framing, source receipts,
server/native PID and birth identities, inclusive process-tree CPU and request
wall observations remain available. No public field or failed case is filtered.

Controls retain whole inputs and responses for an unsaved host change and its
inverse, same-mtime/same-length closed dependency and strict-config edits and
their inverses, cancellation and the next request, close refusal/reopen, an
independent original-generator 20-SFC root and the original root afterward,
and retirement/recovery of only physically verified native descendants. The
changed dependency must resolve to `Ref<string>` and then `Ref<number>`; the
config probe must change from `null` to `any` and back. Current-version native
completion is observed before requests after edits; all control outcomes are
aggregated before a terminal failure gate. These are strict obligations, not
accepted observations yet. Existing source checks at 322de6b passed, but no
original400 speed, CPU, native recovery or protected/release credit is claimed
until the newly configured exact source pair executes and its full artifacts
are authenticated. No additional production request, pipeline stage, IPC or
instruction ceiling change is introduced.

## Whole protocol and owned lifecycle correction

Paired decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8008#issuecomment-5994544164).

The first bounded original400 helper review found three test-custody gaps,
without a production-cache finding. Put session/sampler construction under
owned cleanup and retain construction, shutdown and capture failures. Recheck
captured PID and birth against current descendants and the physical native
executable immediately before any test retirement signal; a reused PID is a
distinct life. Check every retained native identity after actual shutdown.

Bind each measured client ID to exactly one complete original request and
response in the existing raw frame decoder. The passive test-client observer
retains the whole response before unchanged dispatch, including unknown
fields and the distinction between absent and null error data. Reject duplicate
response IDs and reconcile the full observed response vector against captured
frames. The pair compares these complete envelopes after only the same
physically checked private-root bijection. This correction does not add any
production/native protocol or response filtering. Fresh exact-source paired
execution remains required; neither source review nor an earlier unit-test
pass grants original400 timing, CPU, recovery or release credit.

## Actual-main source replay

Paired decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8008#issuecomment-5994672343).

Genuinely replay the six owned source commits onto actual main
`a6dfe45aeb9bd4281168b52e94391da59a3ac7ad`, retaining every source
author/email/date/full body and reporter footer and all 26 owned noncanonical
blobs. Removing only the owned clause reproduces the complete incoming
350-line canonical record. The first newly configured pair must build this
actual common source baseline, including incoming backend changes, rather
than borrow original400 evidence from 66a9. Earlier source Check37305000331
and its authenticated four JUnit artifacts prove 16,131 unique passing tests
at historical hosted source5630f3ec, including the four whole source laws;
they do not qualify this new head or any original400 native/wire/CPU outcome.
Fresh exact source Actions and the first paired execution remain required.

## First actual pair and CPU localization

Paired decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8008#issuecomment-5995283667).

Actual run37311997959 compiled common baseline a6dfe45a and successor
48272963 with identical CI recipe, locks, physical workspace and pinned native
runtime. Official artifact11346239589 is 4,983,594 bytes with SHA256
67af9aead8dd3b6f97ec291ebd89158f9f9785663a14fc0d24e5713c0edaa722;
all 767 members and CRCs are authenticated. Both processes exited cleanly and
retained all 74 envelopes and raw wire. Baseline failures are empty; the
successor's first unsaved-host script hover, ID26, returned null in 0.599 ms.
The other 73 complete envelopes match after the checked private-root bijection,
including every original warm request. This whole oracle remains red.

All 20 warm wall-time medians are 2612.376 ms before and 2637.232 ms after.
Sampled Vize CPU medians are 2.615 s and 2.630 s; live native CPU medians are
zero. These failed-pair observations show no measured gain. Both sources log
native 12-second timeouts. A later collected-diagnostics marker acknowledges
collection/publication, not successful native completion; abandoned work can
still be draining. The source path permits a fast refusal, while the particular
null's correlated draining/error cause remains unproven. Keep the original
null/typed/invalidation/refusal expectations and all recorded outcomes strict.

Add read-only per-thread CPU ticks and process-I/O observations to the same
existing pair outside each original request's timed wall/CPU window. Retain
raw stat, thread name and I/O bytes plus physical executable, PID and birth
identity. Recheck the observed process life and executable; native lifecycle
misses retain their error instead of inventing zero work. The measured parent
identity must remain exact. No immutable baseline patch, extra IPC, retry,
sleep, payload filter, production behavior or pipeline stage is added. These
observations localize thread work, not source statements or Program counts.
Linux field authority is the [kernel proc documentation](https://www.kernel.org/doc/html/v6.7/filesystems/proc.html).
Fresh whole execution remains required; #8035 stays Draft and offqueue under
the finite first v0.433 publication hold.

## Equal-binary source-build custody correction

Paired decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8008#issuecomment-5995685205).

The preceding compiled-successor attribution is superseded by the actual
build and launch evidence. First run37311997959 compiled Canon, Maestro and
the CLI from the baseline worktree in a 52.64-second build. The successor
command took 0.13 seconds without a compilation line. Both source receipts
and actual launch captures contain identical binary SHA256
41a78da3c12da5c189a26a0fcaf28442287fb70a3d0e1f2e321a7ea7e9952041.
The shared target is concrete; the exact fingerprint cause remains unknown.
The original 74 envelopes, null, timeouts and CPU/wall observations remain
historical actual execution, but cannot establish a new-cache effect or its
compiled-successor performance. Ordinary Check source execution is separate.

Repair only the two existing automatic pair build steps. Identical package
clean under the existing CI profile removes Canon/Maestro/CLI artifacts on
each side while retaining other unchanged dependencies. Capture complete
Cargo output and process outcomes. Require all four library/binary artifacts
to show executed rustc (`fresh=false`), actual-side manifest/root source paths,
exact native/default features, identical profiles and successful build-finished.
Bind frozen and successor launch files to the captured CLI filename/hash and
successful unchanged receipt. The input allowlist also rejects npm drift,
including the CLI's embedded schema; reused dependency artifacts receive no
fresh-compilation claim. The [Cargo artifact contract](https://doc.rust-lang.org/cargo/reference/external-tools.html#artifact-messages)
and [package clean scope](https://doc.rust-lang.org/cargo/commands/cargo-clean.html)
define those checks.

Preserve the original generator and every complete packet, inverse, cancellation,
refusal, recovery and timeout gate. No retry, profile/ceiling change, extra
job or baseline source patch is added. Fresh actual compilation plus the
whole same-worker execution remain required; the thread/I/O observation is
source-only pending, and the finite first-v0.433 admission hold remains.

## Current native-baseline replay

Paired decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8008#issuecomment-5995975121).

Genuinely replay the nine existing commits onto actually merged main
92e036de6f8dc651440e7c9a2f415a3c7fa69890, including #7857. All source
authors/email/dates/full bodies and reporter footers remain exact. No incoming
main change touches an owned Canon production file. The replay preserves all
27 noncanonical blobs other than the shared inventory; that inventory retains
both the complete incoming row delta and the exact owned moved-source/offset
delta. Preserve every incoming canonical byte at 350 lines and append this
narrow history in a separate meaningful docs-only commit.

The next same-worker comparison builds this current common native baseline
and its successor, including identical merged configuration/publication
behavior. Strong Cargo fresh-artifact and launch-hash proof must precede
the unchanged complete original400 oracle. Earlier equal-binary observations
and source JUnit outcomes remain historical, with no execution or performance
credit transferred. Existing #8035 stays Draft/offqueue under the finite cut
hold; current source peer and automatic Actions remain mandatory.

# Native Maestro File navigation retention

Tracked in [#6871](https://github.com/ubugeeei-prod/vize/issues/6871) and
[#6872](https://github.com/ubugeeei-prod/vize/issues/6872).

## Decision

The existing opt-in JS/TS navigation endpoint keeps one genuine L1 syntax and
L2 File alive per physical immutable source snapshot. Its requests use the
[shared original File position API](./2026-10-03-file-position-queries.md).
The previous private declaration/reference response index is removed.

Each worker thread receives an owned `Arc<SourceSnapshot>`, constructs its
allocator, original `NativeSyntax` and checked `File` as ordinary local values,
then serves requests throughout their scopes. The File drops before syntax and
allocator. Native objects never cross a thread boundary or become shared
`Arc` data. No lifetime cast, unsafe code, self-reference, leak, additional parse
or level serialization is introduced.

The project cache holds physical snapshot keys and sendable command handles.
An older revision cannot replace a newer cache entry. Supported repeated
definition/reference requests call the same retained File and original targets;
there is no per-request parsing or copied semantic index. Source/profile/parser
and genuine File completion refusals remain explicit without a legacy fallback.
Only owned locations or refusal data cross the channel.

Worker admission checks actual current host identity while holding the worker
cache mutex. A mutation that has already notified an empty cache cannot admit
an old worker afterward; a later mutation must wait for that same cache mutex
before its notification retires the new entry. The document read guard ends
before spawning, and notification never holds a document guard while taking
the worker cache lock.

## Cancellation and resource bounds

`ProjectQuery::run` and its final guarded publication remain authoritative for
current source, version and cancellation. No mutex or document guard survives
the response await. Dropping a request drops its oneshot receiver; queued
cancelled work skips projection. One request's cancellation preserves other
requests on the same current snapshot.

Actual host changes retire superseded workers outside the cache lock, while a
fresh entry captured before notification survives. Retirement sets an atomic
flag and attempts a wake command, even when outstanding handle clones exist.
Workers drop queued requests and original owners after their current synchronous
segment. Production Drop never joins. A parser or File walk already executing
is cooperative and cannot be forcibly preempted. The existing didChange
notification moves before diagnostic awaits, immediately after the actual
document mutation.

There are at most sixteen live worker threads per project and sixteen queued
commands per worker. A live slot is released only after the original native
owners exit, including unwind. Saturation returns distinct capacity/busy
refusals through the preview JSON-RPC endpoint, with no blocking send or silent
eviction/reparse. Transient capacity/spawn failures are retryable. A terminated
worker remains an explicit unavailable refusal until host replacement.

## Verification and remaining work

All fifty-four SourceProject laws, including twenty native navigation laws,
and six complete production JSON-RPC laws pass locally. They cover live
original Program/File observations and actual parse
calls across repeated requests, UTF-8/UTF-16/CRLF boundaries, shadowed and imported
bindings, original refusals, cancellation, close/reopen, old-insertion races,
bounded request/worker capacity, unwind and native-owner/snapshot release.
Channel barriers and bounded exit waits provide deterministic ordering.
Strict all-targets feature Clippy passes. Exact-head Actions and the protected
queue remain required; the existing differential recipe executes these feature
laws, the minimal library check and strict feature Clippy in the queue.

The minimal non-native library feature compiles. Whole libtest without default
features exposes inherited unconditional formatter/native state tests; it is
not reported as a passing test campaign. The executed navigation/wire laws use
the normal default features plus `experimental-source-navigation`.

This establishes retained L1/L2 ordinary JS/TS navigation. Vue, JSX/TSX,
workspace/external symbols, L3/L4 retention, broader language admission,
instruction acceptance and complete Maestro fix history
[#6883](https://github.com/ubugeeei-prod/vize/issues/6883) remain unfinished.
The existing standard request routing is independent of this opt-in endpoint.

# Pinned mounted runtime bundle reuse

Issue: [#6863](https://github.com/ubugeeei-prod/vize/issues/6863).

The four PR runs 36819142814, 36819143330, 36819031970 and 36819152224
completed their required report in 7m45s, 8m33s, 7m48s and 6m59s.
The Rust build/archive followed by its workers is the critical path.
Its Transition parity case takes 89.93–90.84s and starts 27 independent
Node processes: nine scenarios against official, native and retained code.
Every process rebuilds the same pinned Vue runtime bundle.

## Provider contract

Reuse only immutable bundled source bytes. A complete, atomically published
record binds the input identity, resolved module list and bundle content hash.
Never cache a DOM, imported module, Vue scheduler, compiled test render,
AST, trace, golden output or test result. Every existing subprocess and its
fresh data-URL import remains necessary.

Input collection follows actual installed package resolution, including
transitive dependencies, optional/peer presence and all installed package
file contents. It also fingerprints workspace pins, harness implementation,
exact bundler configuration, runtime mode, actual working directory, Node and
platform context, and relevant build environment. The real build must prove
its modules belong to the collected input set and produce one self-contained
module without external imports or assets.

Reuse is deliberately bypassed for env files, custom native bindings, forced
WASI, WebContainer or Node import/require/loader hooks. Those contexts retain
the original build behavior rather than claiming a complete bounded identity.
Development and production use distinct identities; the production behavior
introduced by #7308 is retained.

Input changes invalidate reuse. Corrupted, partial, incomplete or wrong-input
records are treated as misses and rebuilt before executable bytes are returned;
cache I/O failure bypasses storage while actual build/runtime failures still fail.
Only regular files may be quarantined: unexpected directories or symlinks are
preserved and bypass storage, and cleanup never recursively removes contents.
A failed build or inputs changed during a build cannot publish a record. Concurrent successful
writers atomically publish complete equal bytes without overwriting a valid
record; differing successful builds for identical inputs fail. Missing module
coverage bypasses both reuse and storage. Synthetic laws exercise these boundaries
and fresh subprocess state.

## Delivery and measurement

The provider is independently reviewable before its harness consumer.
The consumer will replace the existing repeated build with this provider,
keeping every assertion, snapshot, subprocess and runtime mode. Actions will
replay the identical Transition command with reuse disabled, cold and warm,
record bundle preparation separately and require all 27 comparisons each time.
No existing gate, filter, retry, timeout or numerical ceiling is changed.

The two-minute whole-PR target remains unfinished. Rust archive transfer,
duplicated native preparation and browser preparation are separate measured
bottlenecks; this change does not claim to resolve them.

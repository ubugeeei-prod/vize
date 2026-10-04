# Snapshot import resolution sharing (2026-10-04)

Tracking: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698).
Prerequisite: [batch source snapshots](./2026-10-04-typechecker-batch-source-snapshot.md).

## Decision

The batch snapshot shares unresolved module parses, but its import resolver
still keys results by the entire importer filename and holds its map mutex
through filesystem probes. Sibling SFCs repeat the same relative import
resolution while unrelated dependency resolutions serialize on that mutex.
The prerequisite's default-max CLI measurement did not demonstrate a gain;
its imported corpus worker augmentation grew despite the one-thread gain.
This follow-up requires new same-run complete CLI evidence before acceptance.

Share snapshot resolution cells by canonical importer directory and exact
specifier. The existing compatibility resolver depends on that directory
for relative paths, `@/` source aliases and nearest package ancestry. It has
no importer mode or condition input. Its existing `.mts`/`.cts` behavior is
preserved; Canon's contextual package route resolver is unchanged. Parentless
paths and a filename literally named `node_modules` use a distinct full-file
cache scope, retaining their previous context and snapshot hit/miss caching.

Each cell captures a positive result or a miss once with `OnceLock`. Release
the map mutex before probing or waiting on a cell, so independent imports
can resolve concurrently. New snapshots refresh missing files and dependency
edits. Overlay and disk extension precedence, JS substitutions and package
exports and the compatibility resolver's returned paths remain unchanged.

The public resolver normalizes the supplied importer on every call, retaining
current-directory and symlink freshness for saved or new unsaved roots. This
includes every type-world edge, exactly as before. There is no new canonical
path cache or trusted-path shortcut. Root text and normal/setup scope
still come from the current caller. Cycle handling and the existing per-world
512-module admission check remain unchanged.

## Required evidence

- Sibling Vue, `.mts` and `.cts` importers share one resolution while distinct
  package scopes and exact specifiers remain separate.
- Overlay JS/JSX/MJS/CJS substitutions and `@/` alias precedence match existing
  behavior; a cached miss or source revision refreshes in a new snapshot.
- Concurrent siblings publish one cell per import. Public saved and unsaved
  roots observe symlink retargeting and relative cwd changes; cwd mutation runs
  in an isolated test process.
- Symlink workspace package targets match the compatibility resolver's exact
  returned paths. Full exported dependency facts and module identities match
  sibling consumers and fresh snapshots; special file contexts retain cached
  relative/absolute misses until a new revision.
- Existing root-buffer, private binding, normal/setup, TSX, cycle, overlay,
  fresh-batch and warmed admission-limit tests continue to pass.
- The unchanged paired CLI harness builds exact full Stack base/head CLIs and
  retains complete diagnostic and program signatures for generated500 and
  shared-barrel500 in max and one-thread modes. Cold launches, warmed process
  pairs, raw vectors, executable/runtime hashes and separate profiles remain
  required. A successful parity job alone does not reject a timing regression.

Performance acceptance, protected native Stack delivery, real-project and
persistent-session edit evidence, and the full-command 10x goal remain pending.
No benchmark budget or product output is relaxed by this change.

Further canonicalization reuse is deferred: the compatibility resolver can
return its original path when canonicalization fails, so a private trusted
path shortcut requires separate complete path and world-identity proof.

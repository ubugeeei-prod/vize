# Batch typechecker source snapshots (2026-10-04)

Tracking: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698).
Prerequisite: [immutable dependency module facts](./2026-10-04-typechecker-snapshot-performance.md).

## Decision

Share one `TypeSourceSnapshot` through the existing private
`RuntimePropResolveCache` for each `VirtualProject::register_paths` call.
The cache already lives on that call's stack and is borrowed by its parallel
workers. It is dropped after registration, so a later scan captures dependency
edits and missing-file resolutions again. Single-file registration keeps its
fresh local cache. Concurrent projects never share a snapshot.

The SFC Croquis merge always constructs and installs a `ResolvedTypeWorld`
before resolving props. Since scoped props were selected in `389fd87223`,
its compatibility collector, normal-script flat-name collection, context
analysis and fallback props branch do not contribute to the selected result.
The world builder consumes only the current context source, supplied normal
script, syntax and snapshot. Remove that unused work within the private merge;
leave both public imported-type collectors and their respective traversal
policies unchanged. The normal/setup root script is parsed from its current
descriptor for every caller, including TSX. Dependency facts are unresolved
until each world's own clone resolves its imports.

The world's existing 512-module admission policy remains in the world
builder, before cache lookup. A snapshot warmed by smaller worlds must not
allow a larger world to admit dependencies beyond that policy. This change
does not switch between the disk compatibility collector's unbounded traversal
and the snapshot compatibility collector's bounded traversal.

## Required evidence

The source Actions test batch and single-file virtual output and diagnostics,
split normal/setup blocks, TSX syntax, a shared dependency edit across fresh
registrations, concurrent projects, and warmed-cache admission beyond 512
modules. Existing scoped-world and source-snapshot laws remain required.

The dependent PR also builds the actual CLI at its exact common ancestor and
head, deriving the full Stack baseline from its common ancestor with `main`
so the parent's first-use module cloning costs are included. Both executables
run against the same generated 500-SFC corpus and a
shared imported-barrel corpus. Each side/thread mode must retain full
normalized diagnostic fingerprints on every run and pass the existing
minimal and corpus-scale planted diagnostic gates before timing is published.
First-launch samples and warmed fresh-process samples are distinct; filesystem
caches are not forcibly evicted. Two untimed warmups precede nine alternating
pairs in max and one-thread modes. Profiling uses a separate process after all
timed samples. Executable hashes, exact SHAs, runtime versions, corpus bytes,
raw samples and diagnostic records accompany the report.

The historical full-command max target is 42.55 ms, derived from the 425.5 ms
baseline in [the prerequisite record](./2026-10-04-typechecker-snapshot-performance.md).
A gain in the imported graph alone does not establish that target
or native Davinci product completion. #7698 remains open until the pinned
full-command 10x criterion, real-project and persistent-session edit evidence
are met. Publication and merge are pending until exact-head Actions and the
protected native Stack queue actually finish.

## Rejected occurrence-cache experiment

[PR #7705](https://github.com/ubugeeei-prod/vize/pull/7705) was closed unqueued
after its local whole-CLI experiment showed no demonstrated benefit. The
[complete experiment record](./2026-10-04-check-import-occurrence-cache.md)
retains its paired samples, binary hashes and actual profile. The
[issue receipt](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-5975755104)
records exact base `da66dc241cb7e6ad25fc52fbb4a1b2c1effec9e5` and candidate
`a2f8356f860ffbb5c822c3001dfbe784d24dca0e`, identically configured `ci-opt`
builds in the same physical tree, Corsa 7.0.2 and Vue 3.6.0-beta.10.
Fifty alternating fresh-process pairs retained complete 502-file JSON parity
in all 100 processes. The median was 787.954 to 789.758 ms (+0.23%);
paired mean base-minus-candidate was -2.274 ms, standard deviation 54.185 ms
and standard error 7.663 ms. A separate profile observed 502 scans and 502
reuses, which does not demonstrate an end-to-end gain. This local corpus is
separate from the pinned 500-SFC Actions acceptance workload.

The binary hashes were
`b052a9ed3fb5ff62d1e119a7a3f5a31154318add939aa5b6c287b869e4ac7385` (base)
and `a80ea53c9bff3f3582828a8d960cb33d0d388c1ee18e91f27308a9455ffb649d`
(candidate). The rejected cache code is not part of this batch slice.

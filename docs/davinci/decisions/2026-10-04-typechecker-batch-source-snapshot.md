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
head. Both executables run against the same generated 500-SFC corpus and a
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
A gain against this PR's immediate parent alone does not establish that target
or native Davinci product completion. #7698 remains open until the pinned
full-command 10x criterion, real-project and persistent-session edit evidence
are met. Publication and merge are pending until exact-head Actions and the
protected native Stack queue actually finish.

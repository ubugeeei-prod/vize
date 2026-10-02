# Project path selection ownership

Issue: [#6834](https://github.com/ubugeeei-prod/vize/issues/6834).
Prerequisite: [configuration host ownership](./2026-10-01-config-host-owner.md).

Carton owns `ProjectModel`, the snapshot of invocation root, selected config
source and TypeScript project paths. Its existing module and three tests
move unchanged in a move-only commit. The CLI already consumes Carton's
config facade; the two Maestro project-selection calls use Carton's model.
L0 keeps the exact `TypeCheckerConfig` effective type consumed by the model
and has no forwarding project API or reverse host dependency.

The selected config directory retains precedence over the invocation root
for configured relative tsconfig paths. Explicit CLI selection continues to
use the invocation root and override config selection. Absolute paths and
unrooted relative paths retain their identity. No filesystem lookup,
compiler/checker route switch, additional pipeline stage or serialization
is introduced; existing output, errors and numeric ceilings remain unchanged.

The source storage gate recognizes only `ProjectModel::new` at its two
existing Maestro callers. It continues to reject Carton storage, effective
type imports, other constructors and similarly named APIs. Mutation laws
cover these boundaries and both slash and Windows source paths.
The source-only consumer inventory is refreshed without changing fixtures.

Replay with `vp node tools/support/levels/move-project-host.ts moves`, commit
the pure move, then run `integrate` and `check`. The published host-import
gate companion must accompany replay. Source/target collisions and missing
host configuration ownership fail before moving; repeated phases are clean.
The earlier config-host replay recognizes this exact later host extension
without deleting it or permitting an unrelated Carton module shape.

This slice is prepared privately while the config Stack is in the queue.
It is rebased or replayed onto fresh `main` only after that prerequisite
actually merges, then reviewed before publication and validated on Actions before entering
its own protected queue validation. #6834 stays open: path matching,
effective-type framework placement, i18n, profiler and full platform
isolation remain unfinished.

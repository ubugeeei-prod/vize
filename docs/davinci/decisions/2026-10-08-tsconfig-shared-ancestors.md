# Preserve shared tsconfig ancestors

Owning issue: [#3984](https://github.com/ubugeeei-prod/vize/issues/3984).

The CLI input-selection and declaration-option readers used a permanent visited
set for an entire `extends` operation. When two entries inherited the same base,
the later entry received no base values. Its inherited selection, exclusions,
`allowJs`, declaration directory, output directory and map setting could therefore
lose to an earlier entry's own overrides. Canon's compiler-option reader already
keeps the active ancestry separate from completed values; these CLI readers must
follow the same inheritance law.

Both readers now use operation-local active ancestry and completed-value
memoization. A completed acyclic config is merged into every later occurrence,
with its original path anchors and explicit field-presence flags. Each unique
acyclic config is read and parsed once per operation. A cycle still yields the
reader's existing default result, and every result depending on that cut remains
uncached because it depends on the active ancestry. Read failures clear active
state and propagate through the existing outer fallback. Missing extends targets,
malformed readable JSON, empty arrays, local false overrides and explicit CLI
output-directory precedence keep their existing behavior. Silent legacy cycle
termination differs from stock TypeScript's cycle error and is not parity credit.

The complete corpus includes both sibling configs, their shared base, selected
and excluded TS/JS roots, an authored Vue root, clean/broken/repaired source, the
full official TypeScript 6.0.3 config/declaration/map oracle, and a separate
consumer containing only copied declaration outputs. The old source-extracted
loader disagrees with the external config in six fields. Local helper laws cover
an acyclic diamond of depth 24, ancestry-sensitive cycles and failed reads. These
bounded local witnesses establish neither the complete CLI nor native acceptance.
Fresh exact-head Actions must execute the whole CLI/program/diagnostic results,
emitted bodies/maps and declaration-only Vue/TS consumer, retaining the complete
existing project/reference/packed-consumer corpus. The protected queue must then
execute the original full runtime and immutable instruction gates before actual
merge; installed-consumer replay remains a separate release requirement.

The adjacent `compilerOptions.types` reader still concatenates inherited arrays
and suppresses shared ancestors. A separate external RED corpus must fix its
last-declared-value semantics, including an explicitly empty array, before it can
claim effective project authority. This change does not replace a legacy product,
close #3984, qualify complete project/build/watch/invalidation/LSP/Content Mapper
parity, establish n8n JSONC transport authority, or demonstrate the 10x performance
target. Upstream TypeScript and n8n remain read-only.

## Required native CLI coverage

The first protected candidate at `9df572dbd17b56ae16e8b9d61ea886df8cf91969`
executed the first public CLI call in each new law. Both expected exit statuses
matched, but the harness then rejected the existing normal progress messages on
stderr before parsing the complete JSON. Neither law established its full
selection, declaration/map or independent-consumer contract in that candidate.
The original raw progress and failed worker receipts remain historical evidence;
there is no protected rerun or acceptance credit from that failure.

The new harness now uses the existing public `--quiet` option while retaining
its complete input, JSON, diagnostic, declaration, map, copied-package and exit
expectations. Production output is unchanged. The corpus module moves to the
standalone `tsconfig_diamond` Cargo test target in a move-only commit, followed by
separate wiring with the original helper literals and removal of only its own
former parent-module link. Existing declaration targets and fixtures stay intact.

The existing required-native source CLI qualifier adds only
`--test tsconfig_diamond` to its current argv. Every prior target and flag, the
same job and stage, and its original 40-minute limit remain. The new target must
execute both complete public laws and all eight original CLI invocations under
the required native runtime before source readiness, followed by the full fresh
protected suites and immutable instruction gates before actual merge. An ordinary
PR worker's disable-TSGO early return remains zero native-body acceptance. This
coverage change does not qualify the separate types-array successor or close the
full #3984 project, declaration, build/watch, LSP, parity or performance roadmap.

## Authored public JSON path envelope

The fresh source-native run for `ffc8df2e37613ce77eeacbb02754ce9b318f8754`
reached the first complete JSON packet in each new law. The authored diagnostics,
reported files, program roots and file counts matched, but the newly anticipated
path spelling did not match the existing public reporting contract. Neither law
completed, and the six later CLI invocations remained unexecuted.

The compiler-options snapshot explicitly makes every path-bearing option
absolute before public JSON reporting. Declaration reporting strips the project
root while preserving the literal config anchor, so the unchanged authored
`configs/shared.json` destination `../types-base` is reported as
`configs/../types-base/...`. The five original declaration tests establish no
alternative inherited-path spelling contract.

The harness now binds only the expected `rootDir`, `outDir` and `declarationDir`
to its independently known project root and derives each expected declaration
path from the preserved authored config anchor. No actual response supplies an
expected value, and no field is removed or filtered. The frozen seven corpus
files and every original expected literal remain byte exact. An independent
compiled projection/inverse law recovers the complete original expected DTOs for
both inheritance orders and the declaration report, preserves unknown fields,
and keeps wrong-sibling paths unequal. Production reporting stays byte exact.

Fresh source-native Actions must still execute both complete laws and all eight
original CLI invocations, followed by the full protected suites before actual
merge. This correction does not close the remaining #3984 project, declaration,
build/watch, LSP, parity or performance requirements.

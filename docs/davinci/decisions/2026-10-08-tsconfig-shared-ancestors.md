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

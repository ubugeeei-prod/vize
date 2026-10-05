# Local snapshot replacement (#7914)

The original `LazyInput.vue` stores a plain snapshot in `lastEmitted`, then
replaces that local binding inside `commit`. An identifier assignment never
writes through its old value. Do not encode identifier assignments as snapshot
member mutations. Retain the first extraction finding. Replacing the binding
with a non-snapshot value clears its old provenance, so later copies/composable
reads do not claim that the new value still comes from `props.modelValue`.

Preserve member writes, mutating calls, updates, assignment from another snapshot
and the existing loss enum/diagnostic encoding. A known shadowing parameter/local
must not invalidate the recorded outer binding: compare existing authored binding
identity with the visible scope binding. This uses the current walk and origin
map, without an extra parse or new tracking allocation. Precise control-flow
joins and all nested-scope provenance are not established by this slice; broader
semantic tracking remains unfinished and must retain separate evidence.

The original full SFC uses a raw corpus carrier, keeping all source bytes, source
SHA-256 and issue-body SHA-256. Full ordered loss vectors retain the initial
snapshot, later-copy negatives, original member-write positives and a shadowing
parameter control. Source Actions and protected full fixture/Rust/instruction
count gates, actual signed merge/issue closure and publication are required.
This is a legacy correction, without Davinci native-stage or speed credit.

Hosted Check `37258805916` inherited the parent filter-control and source-census
failures. Its genuine successor rebases onto both repaired parent heads, preserves
the complete loss vectors and production correction, and replays the canonical
record without dropping either parent decision. The official census generator
reports no additional owned rows at this layer. Fresh source qualification is
required; the failed head stays historical and unqueued.

An existing-PR source-qualified successor adds public CLI observation for the
complete original #7913/#7911/#7914 sources, without changing production bytes.
The existing tooling jobs use the exact built CLI receipt/hash/revision and actual
pinned native TypeScript 7.0.2 executable, retaining its version/binary identity
and the Vue type-provider version/hash. Strict-reactivity configuration runs both
with and without `--type-aware`. Compare complete per-file JSON arrays, counts,
messages, severity, help and authored ranges, preserving the initial snapshot
positive. Persist full inputs/configuration/stdout/stderr/exits and expected/actual
reports under `target/differential` before comparisons; the existing always-upload
step retains them on failure. Native CLI execution and fresh Actions acceptance
remain pending until the successor's actual hosted run passes. No new PR or
private #7898 production/corpus is included in this published Stack.

Bounded source peer review finds a concrete evaluation-order error in the first
candidate: clearing `last` before walking the RHS of `last = useFeature(last)`
silences the genuine snapshot argument loss. Defer only plain-assignment origin
replacement until after the existing RHS walk. Reactive-binding reassignment
recording and member-assignment writeback-root ownership keep their original
order. The authored full `RightHandSide.vue` carrier and complete ordered vectors
retain the initial snapshot and RHS call-boundary finding; later reads of the
replaced binding are clean. Sequence RHS and self-alias controls retain the same
boundary positive. The actual public CLI compares the complete two-finding JSON
in both strict-reactivity modes. Preserve all original reporter bytes and the
parent rules/caps. The observer-only `c881e220ac` stays historical; this semantic
repair requires its own immutable source head, peer review and fresh Actions.

The reviewed `b3c173621d` correction and every original/control source, whole
result law and CLI observer are byte-identical after replay onto genuine parent
`6149fd4a01604968c583b29216f8d5f114d51866`, rooted in actual main `a2712e7896`.
Preserve all incoming canonical decisions and regenerate the exact current census.
Historical run acceptance does not transfer to the replay: require fresh source
Actions/native Stack membership and actual protected-prefix merges, then release.

Hosted source Check `37262456064` rejects the observer's unawaited Node test
registration (`typescript/no-floating-promises`), with all 7,590 files correctly
formatted. Await the registration at the existing module boundary; preserve all
observed inputs/assertions, source semantics and the repository zero-warning
budget. Raw job `111612351056` is retained before the correction. This test-only
successor requires fresh Actions; it grants no native execution or acceptance
credit from the rejected run.

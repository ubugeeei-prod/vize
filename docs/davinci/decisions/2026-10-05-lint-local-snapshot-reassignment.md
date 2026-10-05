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

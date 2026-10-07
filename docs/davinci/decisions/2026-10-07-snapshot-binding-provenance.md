### 2026-10-07: lexical snapshot provenance (#7987)

Plain reactive-value origins follow lexical scopes. A snapshot declared inside
one callback cannot make a same-named fresh local in another function a snapshot.
Parameters, catch bindings and block declarations shadow outer origins; closures
still inherit the origin of captured values, and leaving a shadowed scope restores
the outer origin. Replacement clears the visible binding's provenance.

The parser keeps its public debug representation and records scoped origins only
as private analysis facts. A legacy corpus preserves real snapshot-mutation
controls alongside fresh array-copy, object, parameter and catch controls.

Protected queue candidates `bb7eb146` (#8169) and `31d77abc` (#8175) failed the
unchanged Croquis instruction ceilings. The parent measured 164082 against
164060 for compile-small; the child measured 165631 against 164060, 169910
against 168657 for hoist-off and 170525 against 169896 for hoist-on. All three
original observations agreed. Both candidates were removed from the queue.

The reviewed repair retains the universal root already built by
`ScriptParseResult::default()` and reserves its existing scope store to 16
slots, instead of building and discarding another copy of the same standard
global bindings. Script setup and plain scripts retain their original root
contents, scope IDs, callbacks and diagnostic settings. A complete original
constructor/transition control accompanies the change; the original nine
lexical-snapshot fixtures and fifteen live-ref fixtures remain byte-exact.
No provenance algorithm, source input, harness, ceiling or pipeline stage is
changed. Fresh source/native/Corsa and protected whole-output/instruction
qualification are required; no new measured result or actual merge is claimed.

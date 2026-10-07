### 2026-10-07: lexical snapshot provenance (#7987)

Plain reactive-value origins follow lexical scopes. A snapshot declared inside
one callback cannot make a same-named fresh local in another function a snapshot.
Parameters, catch bindings and block declarations shadow outer origins; closures
still inherit the origin of captured values, and leaving a shadowed scope restores
the outer origin. Replacement clears the visible binding's provenance.

The parser keeps its public debug representation and records scoped origins only
as private analysis facts. A legacy corpus preserves real snapshot-mutation
controls alongside fresh array-copy, object, parameter and catch controls.

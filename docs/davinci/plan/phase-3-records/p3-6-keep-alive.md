# P3-6 - Checked KeepAlive generation (2026-09-26)

The native payload retains `ComponentKind::KeepAlive`. Admission accepts one
regular or dynamically selected component in implicit default content, with
only `include`, `exclude` and `max` props. Computed expressions retain their
S2 AST. Keyed/conditional children, multiple roots, explicit slot carriers,
unknown props, spreads and component events remain retained-lane selections.

## Shared runtime correction

Mounted comparison exposed an existing retained-generator failure:
`withVaporCtx` is absent from the pinned published runtime. KeepAlive now uses
an `extend`-annotated slot function and a direct dynamic child receives the
runtime's `SLOT_ROOT` flag (4). Both native and retained generation use this
contract. Nested ordinary component slots restore the surrounding context.
The two existing KeepAlive fixture expectations change for this correction;
Suspense has a separate [async lifecycle contract](p3-6-suspense.md).

## Evidence

- Three sources under both prefix settings produce byte-equal native/retained
  code and templates. A checked-payload mutation changes the include filter
  independently of the source supplied to emission.
- Two exact code/map snapshots match every decoded segment with the retained
  lane for static and reactive props and dynamically selected children.
- TS-33 runs the native lane, retained lane and pinned official compiler
  against the same published runtime. Five policies cover unrestricted
  caching, `max=1` eviction, include/exclude filters, and reactive prop getters.
  Cached components retain their DOM identity when reactivated; evicted or
  excluded components receive new identities. Props and emitted events stay
  current across switches; unmount removes the active subtree.
- The dedicated zero-walk/expression-reparse floor and allocation ceilings
  remain unchanged. Fixture coverage stays at 742/743 with only tracked #1161.

These proofs do not close P3-6 or the remaining built-in contracts.

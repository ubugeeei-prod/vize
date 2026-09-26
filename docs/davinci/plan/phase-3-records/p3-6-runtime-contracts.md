# P3-6 — Runtime extension contracts

These bounded production contracts extend the native route and correct the
retained runtime route. Each appendix records its accepted shapes, mutation
witnesses, code/maps, parse/walk floor and mounted official-runtime comparisons.
P3-6's complete surface and performance acceptance remain open.

- [Computed DOM prop names](p3-6-computed-dom-props.md)
- [Computed DOM event names](p3-6-computed-events.md)
- [Computed component prop and event names](p3-6-component-names.md)
- [Computed component model argument names](p3-6-model-names.md)
- [Computed slot outlet prop names](p3-6-slot-props.md)
- [Flat select models and raw option values](p3-6-select-models.md)
- [Static events in once subtrees](p3-6-once-events.md)
- [Teleport targets and cleanup](p3-6-teleport.md)
- [KeepAlive cache lifetimes](p3-6-keep-alive.md)
- [Asynchronous Suspense lifetimes](p3-6-suspense.md)
- [Conditional and looped slot content](p3-6-structural-slots.md)
- [Retained custom directive payloads](p3-6-custom-directives.md)

Computed prop names on Teleport, KeepAlive and Suspense remain unproved. Their checked
schemas reject such names even when the raw JavaScript happens to spell a known
prop. Ordinary component computed names have their own wider contract.

Contract: [P3-6](p3-6.md).

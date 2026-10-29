# P3-6 — Computed DOM event names (2026-09-26)

`@[name]` and `v-on:[name]` now retain the S2 expression AST in the checked
S3 payload, including member, conditional and call expressions. The shared
emitter registers them through `onBinding` in a render effect, preserving
listener options and modifier guards. Static listeners retain their existing
delegation policy. Computed component names have a separate
[contract](p3-6-component-names.md). In a plain-element `v-once` subtree the
computed name is evaluated only at mount, so `onBinding` is emitted outside a
render effect; the listener's handler still reads the current render context.

The exact code and decoded source-map snapshots cover references, indexed
members, conditional names and calls. Computed names and reference handlers
now retain their authored anchors in both lowering lanes. A graph payload
mutation proves generation uses the checked name rather than source text.
The single-recorder floor test covers both prefix settings with zero legacy
walks and expression reparses; the existing seven allocation ceilings remain
unchanged.

Four TS-33 runtime scenarios compare exact DOM, event and node-identity traces
with the pinned official compiler: four name expression forms remove stale
listeners and avoid duplicate registration, `.once.capture.passive` resets
only after a name change, and `.enter.stop.prevent` continues guarding keys
after rebinding. The once-subtree scenario proves that changing the computed
name does not move the mounted listener, while its callback observes the new
context value. Every scenario checks unmount. This extends the bounded
production route; P3-6's full surface and performance acceptance remain open.

Contract: [P3-6](p3-6.md).

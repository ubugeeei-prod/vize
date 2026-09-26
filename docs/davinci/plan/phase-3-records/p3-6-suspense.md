# P3-6 - Checked asynchronous Suspense (2026-09-26)

The native payload retains `ComponentKind::Suspense`. Admission accepts one
implicit default root, or explicit `#default` and optional `#fallback` templates
with one element or regular component root each. `timeout` and unmodified
`pending`, `fallback` and `resolve` listeners retain their S2 expression trees.
Other props, computed names, spreads, scoped/dynamic slots, multiple roots,
nested boundary roots and mixed implicit content with named fallback select
the explicit `Legacy(Component)` route. These shapes remain outside the native
acceptance claim.

## Published runtime contract

Vue 3.6.0-rc.9's official compiler imports the renderer primitive `Suspense`
and calls `createComponent(Suspense, props, ordinaryVaporSlots)`. Both Vize
lanes now emit that contract. The previous retained output resolved a user
component named `Suspense` and imported `withVaporCtx`, which rc.9 does not
export. Inner regular components are resolved in their owning component's
context before deferred slots execute.

As with the official compiler output, mounting this boundary in a Vapor app
requires the published interop bridge:

```js
import { createVaporApp, vaporInteropPlugin } from "vue";
createVaporApp(Root).use(vaporInteropPlugin).mount("#app");
```

The bridge enables async Vapor setup and adapts the renderer primitive while
preserving its pending boundary. The mounted proof installs this same plugin
for all three lanes; generated output does not install application plugins.

## Evidence

- Three sources under both prefix settings produce equal native/retained
  code. A checked payload mutation changes `timeout` independently of the
  source used during generation. Unproved shapes keep an explicit legacy
  selection.
- Root and nested conditional boundaries have exact generated-code and
  decoded source-map snapshots, equal on native and retained lanes.
- The isolated `davinci_suspense_parity` binary compares native, retained and
  official compiler output under the same pinned published runtime. A real
  controlled async `setup()` Promise keeps fallback visible until resolution;
  fallback text updates retain its DOM identity, resolution replaces it with
  the child, and resolved props and emitted events stay current. Root and tail
  identities stay stable.
- Removing a pending boundary and unmounting the app while pending dispose
  the child exactly once. Resolving setup after either operation does not
  mount or resurrect DOM. Removing a resolved boundary and final app unmount
  also dispose the child and leave no nodes behind. Runtime warnings/errors
  are asserted empty.
- Native parent and child compilation have zero legacy walks. Retained
  compilation stays in its own test binary so process-global walk probes in
  native-only TS-33 binaries remain isolated.

The runner awaits the controlled setup continuation and Vue's scheduler; it
uses no timing sleeps. These async lifecycle proofs do not close the complete
P3-6 surface or nested boundary, hydration, error/rejection and timeout-delay
semantics.

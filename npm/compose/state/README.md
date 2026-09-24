# @vizejs/state

> Experimental: the reducer, persistence, and generated metadata contract may change during the
> Vize alpha line.

This package is currently workspace-only and has not been published to npm. The example below
requires the Vize repository workspace.

`@vizejs/state` provides typed reducer-store foundations for Vize applications. It is SSR-safe:
stores keep state per instance, snapshots are serializable, and persistence hydration validates the
model key, `.vue` source, version, expiry, and optional migration.

```vue
<script setup lang="ts">
import { createStateStore, defineStateModel } from "@vizejs/state";

const counterModel = defineStateModel({
  key: "counter.panel",
  source: "./CounterPanel.vue",
  version: 1,
  initialState: { count: 0 },
  reducer(state, action: { readonly type: "increment"; readonly by: number }) {
    return { count: state.count + action.by };
  },
});

const counter = createStateStore(counterModel);
counter.dispatch({ type: "increment", by: 1 });
counter.undo();
</script>

<template>
  <output>{{ counter.state.count }}</output>
</template>
```

Server renderers can call `serializeStateSnapshot(store)` and restore the client with
`hydrateStateStore(model, snapshot)`. App-facing model examples are authored as `.vue` SFCs so
generators can keep ownership metadata attached to real source files.

Headless components can use `createControllableState()` for controlled props or uncontrolled local
defaults. Devtools can call `createStateDiagnosticsManifest(models)` during development; production
manifests deliberately return an empty model list so command names, `.vue` sources, and metadata are
not emitted into production diagnostics payloads.
